//! `ssh_config` parsing, limited to what changes identity selection.
//!
//! This mirrors `readconf.c process_config_line_depth()` + the `Host` /
//! `Match` / `Include` handling around it, and deliberately implements only
//! the directives that affect *which keys get tried*:
//!
//! * `IdentityFile` / `CertificateFile`
//! * `IdentitiesOnly`
//! * `Hostname` / `User` / `HostKeyAlias` / `ProxyJump` — not because they
//!   select keys, but because they are the values `%h`, `%r`, `%k` and `%j`
//!   expand to.
//! * `Host` / `Match` block scoping
//! * `Include`
//!
//! Everything else (ProxyCommand, LocalForward, …) is silently ignored — this
//! crate never opens a connection, it only decides which paths to try.
//!
//! Where OpenSSH `fatal()`s and refuses to connect, this crate warns and
//! carries on; the caller logs the warnings.

use std::path::Path;

use crate::expand::{percent_expand, tilde_expand, ExpandContext};
use crate::glob::{glob, match_pattern};

/// `readconf.c:2617 READCONF_MAX_DEPTH`.
const READCONF_MAX_DEPTH: usize = 16;
/// `ssh.h:28 SSH_MAX_IDENTITY_FILES`.
const SSH_MAX_IDENTITY_FILES: usize = 100;
/// `ssh.h:22 SSH_MAX_CERTIFICATE_FILES`.
const SSH_MAX_CERTIFICATE_FILES: usize = 100;

/// A path exactly as written in the config (before `~`/`%` expansion).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RawPath {
    pub raw: String,
    /// `identity_file_userprovided[]` — `readconf.c:1410` passes
    /// `flags & SSHCONF_USERCONF`, so this is "the file it was read from is a
    /// *user* configuration file", which covers `~/.ssh/config`, everything it
    /// includes, and `-i` on the command line; entries from
    /// `/etc/ssh/ssh_config` and the built-in defaults are clear. OpenSSH logs
    /// `no such identity` at a different level depending on it.
    pub user_provided: bool,
}

/// The string options that percent expansion draws from, plus the identity
/// lists.
#[derive(Debug, Default)]
pub(crate) struct RawConfig {
    pub identity_files: Vec<RawPath>,
    pub certificate_files: Vec<RawPath>,
    /// `None` when the config never mentioned `IdentitiesOnly`.
    pub identities_only: Option<bool>,
    /// `Hostname` (`readconf.c:1493`), the source of `%h`.
    pub hostname: Option<String>,
    /// `User` (`readconf.c:1439`), the source of `%r`.
    pub user: Option<String>,
    /// `HostKeyAlias` (`readconf.c:1501`), the source of `%k`.
    pub host_key_alias: Option<String>,
    /// `ProxyJump` (`readconf.c:1543`), the source of `%j`. Kept verbatim,
    /// including the literal `none`, as `parse_jump()` does.
    pub jump: Option<String>,
    pub warnings: Vec<String>,
}

/// One top-level configuration file to read.
pub(crate) struct ConfigFile<'a> {
    pub path: &'a Path,
    /// `SSHCONF_USERCONF` (`ssh.c:576`/`585`): sets `userprovided` on the
    /// paths it contributes and anchors relative `Include`s at `~/.ssh`.
    pub userconf: bool,
    /// `SSHCONF_CHECKPERM` (`ssh.c:585`, `readconf.c:2124`): OpenSSH refuses
    /// to read a file this could be edited by others.
    pub check_perm: bool,
}

/// Read `files` in order for the target described by `ctx`, accumulating into
/// one `RawConfig` exactly like OpenSSH feeds every file the same `Options`.
pub(crate) fn parse(files: &[ConfigFile<'_>], ctx: &ExpandContext<'_>) -> RawConfig {
    let mut parser = Parser {
        ctx,
        out: RawConfig::default(),
        active: true,
        userconf: true,
    };

    for file in files {
        parser.userconf = file.userconf;
        // read_config_file() (readconf.c:2610-2615) starts every top-level
        // file with a fresh `int active = 1`.
        parser.active = true;
        parser.parse_top_level(file);
    }

    parser.out
}

struct Parser<'a> {
    ctx: &'a ExpandContext<'a>,
    out: RawConfig,
    /// The `activep` flag of readconf.c: options are applied only while it
    /// is set.
    active: bool,
    /// The `SSHCONF_USERCONF` flag of the file currently being read.
    userconf: bool,
}

/// Which string option a `parse_string` line writes to.
#[derive(Clone, Copy)]
enum StringOption {
    Hostname,
    User,
    HostKeyAlias,
    Jump,
}

impl<'a> Parser<'a> {
    fn warn(&mut self, message: String) {
        self.out.warnings.push(message);
    }

    /// Open the top-level `file`; a file that cannot be opened is not an
    /// error, it just means "no options" (`read_config_file_depth(): fopen
    /// failure returns 0`).
    fn parse_top_level(&mut self, file: &ConfigFile<'_>) {
        if file.check_perm {
            if let Some(problem) = self.permission_problem(file.path) {
                self.warn(problem);
                return;
            }
        }
        match std::fs::read_to_string(file.path) {
            Ok(content) => self.parse_content(&content, file.path, 0, false),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => self.warn(format!("can't read {}: {e}", file.path.display())),
        }
    }

    fn parse_content(&mut self, content: &str, file: &Path, depth: usize, never_match: bool) {
        for (index, line) in content.lines().enumerate() {
            let lineno = index + 1;
            let line = trim_line(line);
            let Some((keyword, rest)) = split_keyword(line) else {
                continue;
            };
            if keyword.is_empty() || keyword.starts_with('#') {
                continue;
            }
            // readconf.c:1197 `invalid quotes`: an unterminated quote makes
            // OpenSSH abandon the whole file, we skip just this line.
            let args = match argv_split(rest) {
                Some(args) => args,
                None => {
                    let message = format!("{}:{lineno}: invalid quotes", file.display());
                    self.warn(message);
                    continue;
                }
            };
            self.parse_directive(keyword, &args, file, lineno, depth, never_match);
        }
    }

    fn parse_directive(
        &mut self,
        keyword: &str,
        args: &[String],
        file: &Path,
        lineno: usize,
        depth: usize,
        never_match: bool,
    ) {
        // readconf.c:1184 `lowercase(keyword)` then parse_token(): every
        // keyword, Include included, is case insensitive.
        match keyword.to_ascii_lowercase().as_str() {
            "host" => self.parse_host(args, file, lineno, never_match),
            "match" => self.parse_match(args, file, lineno, never_match),
            "identityfile" => self.parse_path(args, file, lineno, false),
            "certificatefile" => self.parse_path(args, file, lineno, true),
            "identitiesonly" => self.parse_identities_only(args, file, lineno),
            "include" => self.parse_include(args, file, lineno, depth),
            "hostname" => self.parse_string(args, file, lineno, StringOption::Hostname),
            "user" => self.parse_string(args, file, lineno, StringOption::User),
            "hostkeyalias" => self.parse_string(args, file, lineno, StringOption::HostKeyAlias),
            "proxyjump" => self.parse_string(args, file, lineno, StringOption::Jump),
            _ => {}
        }
    }

    /// readconf.c `case oHost:`.
    fn parse_host(&mut self, args: &[String], file: &Path, lineno: usize, never_match: bool) {
        if args.is_empty() {
            self.active = false;
            self.warn(format!(
                "{}:{lineno}: Host without a pattern",
                file.display()
            ));
            return;
        }
        // `*activep = 0` (readconf.c:1829) first, and only a positive match
        // re-sets it.
        self.active = false;
        if never_match {
            // SSHCONF_NEVERMATCH is tested inside the argument loop
            // (readconf.c:1838-1841), so the flag keeps activep clear.
            return;
        }
        for arg in args {
            if arg.is_empty() {
                self.active = false;
                self.warn(format!("{}:{lineno}: empty Host pattern", file.display()));
                return;
            }
            let (negated, pattern) = match arg.strip_prefix('!') {
                Some(rest) => (true, rest),
                None => (false, arg.as_str()),
            };
            // `Host` patterns match the host as given (`host_arg`), not the
            // `Hostname` value — that is what makes host aliases work.
            if match_pattern(self.ctx.host, pattern) {
                if negated {
                    // A negated match vetoes the whole line immediately
                    // (readconf.c:1846-1852 `*activep = 0; break`).
                    self.active = false;
                    return;
                }
                self.active = true;
            }
        }
    }

    /// readconf.c `case oMatch:` via `match_cfg_line()`.
    ///
    /// Only the `all` condition is understood here; every other condition
    /// (`host`, `user`, `exec`, `final`, …) is reported as a warning and
    /// the block is **deactivated**. Note the consequence: if the only
    /// `IdentityFile` lines in the config sit inside such a block, this crate
    /// sees no identity files at all and `fill_default_options()` adds the
    /// built-in list, which OpenSSH would not have done. Skipping the block
    /// is still the safer of the two — evaluating `Match exec` would mean
    /// running a command from a config file we do not fully trust.
    fn parse_match(&mut self, args: &[String], file: &Path, lineno: usize, never_match: bool) {
        // match.c: `all` must appear alone; `!all` is allowed and means the
        // opposite. readconf.c:1879 `*activep = (flags & NEVERMATCH) ? 0 : value`.
        if args.len() == 1 && args[0].eq_ignore_ascii_case("!all") {
            self.active = false;
            return;
        }
        if args.len() == 1 && args[0].eq_ignore_ascii_case("all") {
            self.active = !never_match;
            return;
        }
        let shown = args.join(" ");
        self.active = false;
        self.warn(format!(
            "{}:{lineno}: unsupported Match condition, block skipped: {shown}",
            file.display()
        ));
    }

    /// readconf.c `case oIdentityFile:` / `case oCertificateFile:` — the
    /// argument is stored verbatim, `ssh.c:2402` expands it later.
    fn parse_path(&mut self, args: &[String], file: &Path, lineno: usize, certificate: bool) {
        let Some(arg) = args.first() else {
            self.warn(format!("{}:{lineno}: missing argument", file.display()));
            return;
        };
        if !self.active {
            return;
        }
        let entry = RawPath {
            raw: arg.clone(),
            user_provided: self.userconf,
        };
        let (list, what, max) = if certificate {
            (
                &mut self.out.certificate_files,
                "certificate",
                SSH_MAX_CERTIFICATE_FILES,
            )
        } else {
            (
                &mut self.out.identity_files,
                "identity",
                SSH_MAX_IDENTITY_FILES,
            )
        };
        if !push_dedup(list, entry, max) {
            self.warn(format!(
                "{}:{lineno}: too many {what} files specified (max {max})",
                file.display()
            ));
        }
    }

    /// readconf.c `case oIdentitiesOnly:` → `goto parse_flag`.
    fn parse_identities_only(&mut self, args: &[String], file: &Path, lineno: usize) {
        let Some(arg) = args.first() else {
            self.warn(format!("{}:{lineno}: missing argument", file.display()));
            return;
        };
        // readconf.c:1274 parses the value first and reports an unsupported
        // one whether or not this block is active; only the assignment below
        // is gated (readconf.c:1277 `if (*activep && *intptr == -1)`).
        let Some(value) = parse_flag(arg) else {
            self.warn(format!(
                "{}:{lineno}: unsupported option \"IdentitiesOnly {arg}\"",
                file.display()
            ));
            return;
        };
        if self.active && self.out.identities_only.is_none() {
            self.out.identities_only = Some(value);
        }
    }

    /// readconf.c `parse_string:` (line 1441):
    /// `if (*activep && *charptr == NULL) *charptr = xstrdup(arg)`.
    fn parse_string(&mut self, args: &[String], file: &Path, lineno: usize, option: StringOption) {
        let Some(arg) = args.first() else {
            self.warn(format!("{}:{lineno}: missing argument", file.display()));
            return;
        };
        if !self.active {
            return;
        }
        let value = Some(arg.clone());
        let slot = match option {
            StringOption::Hostname => &mut self.out.hostname,
            StringOption::User => &mut self.out.user,
            StringOption::HostKeyAlias => &mut self.out.host_key_alias,
            StringOption::Jump => &mut self.out.jump,
        };
        if slot.is_none() {
            *slot = value;
        }
    }

    /// readconf.c `case oInclude:`.
    fn parse_include(&mut self, args: &[String], file: &Path, lineno: usize, depth: usize) {
        let Some(arg) = args.first() else {
            self.warn(format!("{}:{lineno}: missing argument", file.display()));
            return;
        };

        // readconf.c:2077 expand_match_exec_or_include_path(): `%` tokens and
        // `${ENV}` first. A failure warns and `continue`s (readconf.c:2081),
        // which is what the crate does too — unlike OpenSSH's other
        // expansions, this one is not fatal.
        //
        // Deviation: `%h`, `%k` and `%j` take their values from options the
        // config is still in the middle of declaring; OpenSSH uses whatever
        // has been collected so far, we use the pre-parse values.
        let percented = match percent_expand(arg, self.ctx) {
            Ok(expanded) => expanded,
            Err(e) => {
                self.warn(format!("{}:{lineno}: include {arg}: {e}", file.display()));
                return;
            }
        };

        // readconf.c:2091 tests `*p == '~'` on that still un-tilde-expanded
        // result — `~` is a user-config-only privilege. It has to happen
        // before tilde_expand(), or the marker is already gone.
        if percented.starts_with('~') && !self.userconf {
            self.warn(format!(
                "{}:{lineno}: bad include path {percented}.",
                file.display()
            ));
            return;
        }
        let expanded = tilde_expand(&percented, self.ctx.home);

        // readconf.c:2096-2100: a relative path is anchored at ~/.ssh (user
        // configurations) or SSHDIR (system ones).
        let pattern = if Path::new(&expanded).is_absolute() || expanded.starts_with('~') {
            expanded
        } else {
            let base = if self.userconf {
                self.ctx.home.join(".ssh")
            } else {
                Path::new(SSHDIR).to_path_buf()
            };
            base.join(expanded).to_string_lossy().into_owned()
        };

        // GLOB_NOMATCH is not an error (readconf.c:2105-2111): a missing
        // include is simply nothing.
        for matched in glob(&pattern) {
            self.parse_included_file(&matched, depth + 1);
        }
    }

    fn parse_included_file(&mut self, path: &Path, depth: usize) {
        let saved = self.active;
        // readconf.c:2628 `depth < 0 || depth > READCONF_MAX_DEPTH` is a
        // fatal(); here it skips the one file.
        if depth > READCONF_MAX_DEPTH {
            self.warn(format!(
                "Too many recursive configuration includes: {}",
                path.display()
            ));
            return;
        }
        // readconf.c:2123-2125 always adds SSHCONF_CHECKPERM to includes.
        if let Some(problem) = self.permission_problem(path) {
            self.warn(problem);
            return;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
            Err(e) => {
                self.warn(format!(
                    "Can't open included config {}: {e}",
                    path.display()
                ));
                return;
            }
        };

        // readconf.c: an Include inside an inactive block is parsed with
        // SSHCONF_NEVERMATCH (nothing may match), and activep is restored
        // after every included file — readconf.c:2140, "don't let Match in
        // includes clobber the containing file's Match state".
        self.parse_content(&content, path, depth, !saved);
        self.active = saved;
    }

    /// readconf.c:2636-2644 (SSHCONF_CHECKPERM): the file must be owned by
    /// root or by us and must not be group/world writable. OpenSSH `fatal()`s;
    /// we skip the file and warn.
    ///
    /// The owner half needs our own uid, which a dependency-free crate cannot
    /// ask for — it is checked only when the caller injected one. The mode
    /// half, which is the part that makes the file editable by others, is
    /// always checked.
    fn permission_problem(&self, path: &Path) -> Option<String> {
        let detail = owner_problem(path, self.ctx.uid)?;
        Some(format!(
            "Bad owner or permissions on {}{detail}",
            path.display()
        ))
    }
}

/// `pathnames.h:18 SSHDIR` on unix; OpenSSH's Windows build uses
/// `c:/programdata/ssh`.
#[cfg(unix)]
pub(crate) const SSHDIR: &str = "/etc/ssh";
#[cfg(not(unix))]
pub(crate) const SSHDIR: &str = r"C:\ProgramData\ssh";

/// `pathnames.h:38 _PATH_HOST_CONFIG_FILE` (`SSHDIR "/ssh_config"`), the
/// system wide file `ssh.c:589` reads after the user's.
#[cfg(unix)]
pub(crate) const SSH_SYSTEM_CONFIG: &str = "/etc/ssh/ssh_config";
#[cfg(not(unix))]
pub(crate) const SSH_SYSTEM_CONFIG: &str = r"C:\ProgramData\ssh\ssh_config";

/// `Some(reason)` when `path` fails the CHECKPERM test.
fn owner_problem(path: &Path, uid: Option<u32>) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // readconf.c:2637 fstats whatever it opened and applies the same test
        // to it, directories included.
        let meta = path.metadata().ok()?;
        let owner = meta.uid();
        let owned_by_us = match uid {
            // uid 0 is accepted: readconf.c:2642 `sb.st_uid != 0 && sb.st_uid != getuid()`.
            Some(mine) => owner == 0 || owner == mine,
            None => true,
        };
        if !owned_by_us {
            return Some(format!(" (owned by {owner})"));
        }
        if meta.mode() & 0o22 != 0 {
            return Some(" (writable by group or others)".to_string());
        }
        None
    }
    #[cfg(not(unix))]
    {
        let _ = (path, uid);
        None
    }
}

/// `add_identity_file()` (readconf.c:492-500): a raw path already present with
/// the same origin is ignored and does not consume a slot.
fn push_dedup(list: &mut Vec<RawPath>, entry: RawPath, max: usize) -> bool {
    if list
        .iter()
        .any(|known| known.raw == entry.raw && known.user_provided == entry.user_provided)
    {
        return true;
    }
    if list.len() >= max {
        return false;
    }
    list.push(entry);
    true
}

/// readconf.c:994-1000 `multistate_flag` — those four spellings and no
/// others; `on`/`off` are not accepted by OpenSSH.
fn parse_flag(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" => Some(true),
        "false" | "no" => Some(false),
        _ => None,
    }
}

/// readconf.c strips trailing whitespace but keeps the newline — we keep no
/// newline at all since we iterate over `lines()`.
fn trim_line(line: &str) -> &str {
    line.trim_end_matches(|c: char| c.is_whitespace() || c == '\x0C')
}

/// `strdelim()` for the leading keyword: the keyword ends at whitespace,
/// a quote or `=`, after which whitespace and at most one `=` are skipped.
fn split_keyword(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_start();
    if line.is_empty() {
        return Some(("", ""));
    }
    if let Some(rest) = line.strip_prefix('"') {
        let end = rest.find('"')?;
        let keyword = &rest[..end];
        let after = &rest[end + 1..];
        return Some((keyword, skip_after_keyword(after)));
    }
    let end = line
        .find(|c: char| c.is_whitespace() || c == '"' || c == '=')
        .unwrap_or(line.len());
    let (keyword, rest) = line.split_at(end);
    Some((keyword, skip_after_keyword(rest)))
}

fn skip_after_keyword(rest: &str) -> &str {
    let rest = rest.trim_start_matches(|c: char| c.is_whitespace());
    match rest.strip_prefix('=') {
        Some(rest) => rest.trim_start_matches(|c: char| c.is_whitespace()),
        None => rest,
    }
}

/// `argv_split(..., terminate_on_comment = 1)` from misc.c: whitespace
/// separates, `"`/`'` quote, `\` escapes, a `#` at a token start ends the
/// line. `None` means an unterminated quote (`SSH_ERR_INVALID_FORMAT`).
fn argv_split(input: &str) -> Option<Vec<String>> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => {
                i += 1;
                continue;
            }
            '#' => break,
            _ => {}
        }

        let mut quote: Option<char> = None;
        let mut arg = String::new();
        while i < chars.len() {
            let c = chars[i];
            if c == '\\' {
                match chars.get(i + 1).copied() {
                    Some(next)
                        if next == '\''
                            || next == '"'
                            || next == '\\'
                            || (quote.is_none() && next == ' ') =>
                    {
                        arg.push(next);
                        i += 2;
                        continue;
                    }
                    _ => {
                        arg.push(c);
                        i += 1;
                        continue;
                    }
                }
            }
            if quote.is_none() && (c == ' ' || c == '\t') {
                break;
            }
            if quote.is_none() && (c == '"' || c == '\'') {
                quote = Some(c);
                i += 1;
                continue;
            }
            if let Some(open) = quote {
                if c == open {
                    quote = None;
                    i += 1;
                    continue;
                }
            }
            arg.push(c);
            i += 1;
        }
        if quote.is_some() {
            return None;
        }
        out.push(arg);
    }

    Some(out)
}

#[cfg(test)]
pub(crate) mod test_util {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Minimal dependency-free temp directory.
    pub struct TempDir {
        pub path: PathBuf,
    }

    impl TempDir {
        pub fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "ssh_identity_{tag}_{}_{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        /// Write `content` at `relative`, creating parent directories.
        pub fn file(&self, relative: &str, content: &str) -> PathBuf {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create parent dirs");
            }
            std::fs::write(&path, content).expect("write file");
            path
        }

        pub fn path(&self, relative: &str) -> PathBuf {
            self.path.join(relative)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::TempDir;
    use super::*;
    use crate::expand::ExpandContext;

    fn ctx<'a>(home: &'a Path, host: &'a str) -> ExpandContext<'a> {
        ExpandContext::new(home, host, 22)
    }

    /// Parse one top-level file; `userconf` selects the SSHCONF_USERCONF half
    /// of the flags.
    fn parse_one(dir: &TempDir, name: &str, userconf: bool, host: &str) -> RawConfig {
        let path = dir.path(name);
        parse(
            &[ConfigFile {
                path: &path,
                userconf,
                check_perm: false,
            }],
            &ctx(&dir.path, host),
        )
    }

    fn parse_user(dir: &TempDir, name: &str, host: &str) -> RawConfig {
        parse_one(dir, name, true, host)
    }

    fn parse_system(dir: &TempDir, name: &str, host: &str) -> RawConfig {
        parse_one(dir, name, false, host)
    }

    #[test]
    fn missing_config_is_not_an_error() {
        let dir = TempDir::new("missing_config");
        let out = parse_user(&dir, "nope.config", "example.com");
        assert!(out.identity_files.is_empty());
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
        assert_eq!(out.identities_only, None);
    }

    #[test]
    fn global_directives_apply_before_any_host() {
        let dir = TempDir::new("global");
        dir.file(
            "config",
            "IdentityFile ~/.ssh/global_key\nHost other\nIdentityFile ~/.ssh/other_key\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        assert_eq!(out.identity_files.len(), 1);
        assert_eq!(out.identity_files[0].raw, "~/.ssh/global_key");
    }

    #[test]
    fn host_block_scoping_and_negation() {
        let dir = TempDir::new("host");
        dir.file(
            "config",
            "\
Host *.example.com !bad.example.com\n\
IdentityFile ~/.ssh/example\n\
Host bad.example.com\n\
IdentityFile ~/.ssh/bad\n\
Host *\n\
IdentityFile ~/.ssh/any\n\
Host other\n\
IdentityFile ~/.ssh/other\n",
        );
        let out = parse_user(&dir, "config", "good.example.com");
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/example", "~/.ssh/any"]);

        // The negated pattern vetoes the first block, but `Host
        // bad.example.com` matches on its own — `~/.ssh/example` must stay
        // out while `~/.ssh/bad` gets in.
        let out = parse_user(&dir, "config", "bad.example.com");
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/bad", "~/.ssh/any"]);
    }

    #[test]
    fn host_matching_follows_match_pattern() {
        let dir = TempDir::new("host2");
        dir.file("config", "Host Git*.com\nIdentityFile ~/.ssh/key\n");
        // match_pattern() is case sensitive (match.c:82).
        assert!(parse_user(&dir, "config", "github.com")
            .identity_files
            .is_empty());
        assert_eq!(
            parse_user(&dir, "config", "GitHub.com")
                .identity_files
                .len(),
            1
        );
    }

    #[test]
    fn host_patterns_match_the_given_host_not_the_hostname() {
        // readconf.c:1845 matches `host`, which on the first pass is what the
        // user typed; `Hostname` only feeds %h.
        let dir = TempDir::new("host_alias");
        dir.file(
            "config",
            "Host alias\nHostname real.test\nIdentityFile ~/.ssh/aliased\n",
        );
        assert_eq!(parse_user(&dir, "config", "alias").identity_files.len(), 1);
        assert!(parse_user(&dir, "config", "real.test")
            .identity_files
            .is_empty());
    }

    #[test]
    fn match_all_activates_and_other_match_is_ignored() {
        let dir = TempDir::new("match");
        dir.file(
            "config",
            "Match all\nIdentityFile ~/.ssh/matched\nMatch host foo\nIdentityFile ~/.ssh/never\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/matched"]);
        assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
        assert!(out.warnings[0].contains("unsupported Match condition"));
    }

    #[test]
    fn match_negated_all_deactivates() {
        // match_cfg_line(): `all` honours a leading `!` (readconf.c,
        // `result = negate ? 0 : 1`) and needs no warning.
        let dir = TempDir::new("match_not_all");
        dir.file("config", "Match !all\nIdentityFile ~/.ssh/never\n");
        let out = parse_user(&dir, "config", "example.com");
        assert!(out.identity_files.is_empty());
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    }

    #[test]
    fn comments_quotes_and_equals_forms() {
        let dir = TempDir::new("syntax");
        dir.file(
            "config",
            "\
# a full line comment\n\
IdentityFile ~/.ssh/plain\n\
identityfile=~/.ssh/equals\n\
IdentityFile = ~/./spaced\n\
IdentityFile \"~/.ssh/with space\"   # trailing comment\n\
IdentityFile ~/.ssh/quoted # not-a-file\n\
Host \"*\"\n\
IdentityFile \"'single' quoted\"\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(
            raw,
            [
                "~/.ssh/plain",
                "~/.ssh/equals",
                "~/./spaced",
                "~/.ssh/with space",
                "~/.ssh/quoted",
                "'single' quoted",
            ]
        );
    }

    #[test]
    fn unterminated_quote_is_reported_and_line_skipped() {
        let dir = TempDir::new("badquote");
        dir.file(
            "config",
            "IdentityFile \"unterminated\nIdentityFile ~/.ssh/ok\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        assert_eq!(out.identity_files.len(), 1);
        assert!(out.warnings.iter().any(|w| w.contains("invalid quotes")));
    }

    #[test]
    fn identities_only_first_value_wins() {
        let dir = TempDir::new("io");
        dir.file("config", "IdentitiesOnly yes\nIdentitiesOnly no\n");
        assert_eq!(
            parse_user(&dir, "config", "example.com").identities_only,
            Some(true)
        );

        dir.file("config", "Host other\nIdentitiesOnly yes\n");
        assert_eq!(
            parse_user(&dir, "config", "example.com").identities_only,
            None
        );

        dir.file("config", "IdentitiesOnly maybe\n");
        let out = parse_user(&dir, "config", "example.com");
        assert_eq!(out.identities_only, None);
        assert!(out
            .warnings
            .iter()
            .any(|w| w.contains("unsupported option")));
    }

    #[test]
    fn identities_only_accepts_only_the_multistate_flag_spellings() {
        // readconf.c:994-1000 multistate_flag is true/false/yes/no. `on` and
        // `off` are not OpenSSH values and must not enable the option.
        let dir = TempDir::new("io_values");
        dir.file("config", "IdentitiesOnly TRUE\n");
        assert_eq!(
            parse_user(&dir, "config", "example.com").identities_only,
            Some(true)
        );
        dir.file("config", "IdentitiesOnly False\n");
        assert_eq!(
            parse_user(&dir, "config", "example.com").identities_only,
            Some(false)
        );

        for bogus in ["on", "off", "1", "0"] {
            dir.file("config", &format!("IdentitiesOnly {bogus}\n"));
            let out = parse_user(&dir, "config", "example.com");
            assert_eq!(
                out.identities_only, None,
                "{bogus} must not be a flag value"
            );
        }
    }

    #[test]
    fn string_options_take_the_first_active_value() {
        // readconf.c:1441 parse_string: `*activep && *charptr == NULL`.
        let dir = TempDir::new("strings");
        dir.file(
            "config",
            "Host other\nHostname ignored.test\nUser ignored\nHost *\n\
             Hostname first.test\nHostname second.test\nUser alice\nUser bob\n\
             HostKeyAlias alias1\nProxyJump jump1,jump2\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        assert_eq!(out.hostname.as_deref(), Some("first.test"));
        assert_eq!(out.user.as_deref(), Some("alice"));
        assert_eq!(out.host_key_alias.as_deref(), Some("alias1"));
        assert_eq!(out.jump.as_deref(), Some("jump1,jump2"));
    }

    #[test]
    fn identity_and_string_options_ignore_inactive_blocks() {
        let dir = TempDir::new("inactive");
        dir.file(
            "config",
            "Host other\nIdentityFile ~/.ssh/never\nHostname never.test\nIdentitiesOnly yes\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        assert!(out.identity_files.is_empty());
        assert_eq!(out.hostname, None);
        assert_eq!(out.identities_only, None);
    }

    #[test]
    fn dedup_keeps_first_occurrence() {
        let dir = TempDir::new("dedup");
        dir.file(
            "config",
            "IdentityFile ~/.ssh/key\nIdentityFile ~/.ssh/key\nIdentityFile ~/.ssh/key.pub\n",
        );
        assert_eq!(
            parse_user(&dir, "config", "example.com")
                .identity_files
                .len(),
            2
        );
    }

    #[test]
    fn include_relative_to_ssh_dir_and_glob() {
        let dir = TempDir::new("include");
        dir.file(
            ".ssh/config",
            "IdentityFile ~/.ssh/base\nInclude conf.d/*.conf\nInclude missing.conf\n",
        );
        dir.file(".ssh/conf.d/one.conf", "IdentityFile ~/.ssh/one\n");
        dir.file(".ssh/conf.d/two.conf", "IdentityFile ~/.ssh/two\n");
        dir.file(".ssh/conf.d/.skip.conf", "IdentityFile ~/.ssh/skip\n");

        let out = parse_user(&dir, ".ssh/config", "example.com");
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(
            raw,
            ["~/.ssh/base", "~/.ssh/one", "~/.ssh/two"],
            "warnings: {:?}",
            out.warnings
        );
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    }

    #[test]
    fn include_inside_inactive_block_never_matches() {
        let dir = TempDir::new("include_inactive");
        dir.file(
            ".ssh/config",
            "Host other\nInclude included.conf\nHost *\nIdentityFile ~/.ssh/any\n",
        );
        dir.file(
            ".ssh/included.conf",
            "IdentityFile ~/.ssh/global_in_include\nHost example.com\nIdentityFile ~/.ssh/in_include\n",
        );
        let out = parse_user(&dir, ".ssh/config", "example.com");
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/any"], "warnings: {:?}", out.warnings);
    }

    #[test]
    fn include_restores_block_state_of_containing_file() {
        let dir = TempDir::new("include_restore");
        // The include switches to a non-matching Host, but that must not
        // deactivate the block of the outer file.
        dir.file(
            ".ssh/config",
            "Host example.com\nInclude included.conf\nIdentityFile ~/.ssh/after_include\n",
        );
        dir.file(
            ".ssh/included.conf",
            "Host other\nIdentityFile ~/.ssh/other\n",
        );
        let out = parse_user(&dir, ".ssh/config", "example.com");
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(
            raw,
            ["~/.ssh/after_include"],
            "warnings: {:?}",
            out.warnings
        );
    }

    #[test]
    fn include_depth_is_limited() {
        let dir = TempDir::new("include_depth");
        // 21 chained includes: deeper than READCONF_MAX_DEPTH (16).
        dir.file(".ssh/level20.conf", "IdentityFile ~/.ssh/last\n");
        for i in (0..20).rev() {
            dir.file(
                &format!(".ssh/level{i}.conf"),
                &format!("Include level{}.conf\n", i + 1),
            );
        }
        dir.file(".ssh/config", "Include level0.conf\n");

        let out = parse_user(&dir, ".ssh/config", "example.com");
        assert!(
            out.warnings
                .iter()
                .any(|w| w.contains("Too many recursive")),
            "warnings: {:?}",
            out.warnings
        );
        assert!(out.identity_files.is_empty());
    }

    #[test]
    fn system_config_anchors_relative_includes_at_sshdir_and_rejects_tilde() {
        // readconf.c:2091-2100: `~` is a user-config-only privilege, and a
        // relative include is anchored at SSHDIR rather than ~/.ssh.
        let dir = TempDir::new("system_include");
        dir.file(".ssh/config", "IdentityFile ~/.ssh/user_key\n");
        dir.file(
            "ssh_config",
            "Include ~/.ssh/config\nInclude definitely-not-here/*.conf\n",
        );
        let out = parse_system(&dir, "ssh_config", "example.com");
        assert!(out.identity_files.is_empty(), "{:?}", out.identity_files);
        assert!(
            out.warnings.iter().any(|w| w.contains("bad include path")),
            "warnings: {:?}",
            out.warnings
        );
        // GLOB_NOMATCH stays silent, so that is the only warning.
        assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
    }

    #[test]
    fn userprovided_follows_the_file_not_the_line() {
        // readconf.c:1410 `flags & SSHCONF_USERCONF`.
        let dir = TempDir::new("userconf");
        dir.file("user.config", "IdentityFile ~/.ssh/user_key\n");
        dir.file("system.config", "IdentityFile /etc/ssh/host_key\n");
        let user = dir.path("user.config");
        let system = dir.path("system.config");
        let out = parse(
            &[
                ConfigFile {
                    path: &user,
                    userconf: true,
                    check_perm: false,
                },
                ConfigFile {
                    path: &system,
                    userconf: false,
                    check_perm: false,
                },
            ],
            &ctx(&dir.path, "example.com"),
        );
        let flags: Vec<(String, bool)> = out
            .identity_files
            .iter()
            .map(|e| (e.raw.clone(), e.user_provided))
            .collect();
        assert_eq!(
            flags,
            [
                ("~/.ssh/user_key".to_string(), true),
                ("/etc/ssh/host_key".to_string(), false)
            ]
        );
    }

    #[test]
    fn scalars_are_first_wins_across_both_files() {
        // One shared Options struct: the user file is read first, so a
        // Hostname there wins over the system file's, while IdentityFile
        // lines accumulate in read order.
        let dir = TempDir::new("two_files");
        dir.file(
            "user.config",
            "Hostname user.test\nIdentityFile ~/.ssh/user_key\n",
        );
        dir.file(
            "system.config",
            "Hostname system.test\nIdentityFile /etc/ssh/host_key\n",
        );
        let user = dir.path("user.config");
        let system = dir.path("system.config");
        let out = parse(
            &[
                ConfigFile {
                    path: &user,
                    userconf: true,
                    check_perm: false,
                },
                ConfigFile {
                    path: &system,
                    userconf: false,
                    check_perm: false,
                },
            ],
            &ctx(&dir.path, "example.com"),
        );
        assert_eq!(out.hostname.as_deref(), Some("user.test"));
        assert_eq!(out.identity_files.len(), 2);
    }

    #[test]
    fn active_resets_between_top_level_files() {
        // read_config_file() (readconf.c:2610) gives each file its own
        // `int active = 1`, so a trailing non-matching Host in file one must
        // not deactivate file two.
        let dir = TempDir::new("active_reset");
        dir.file("user.config", "Host other\n");
        dir.file("system.config", "IdentityFile /etc/ssh/host_key\n");
        let user = dir.path("user.config");
        let system = dir.path("system.config");
        let out = parse(
            &[
                ConfigFile {
                    path: &user,
                    userconf: true,
                    check_perm: false,
                },
                ConfigFile {
                    path: &system,
                    userconf: false,
                    check_perm: false,
                },
            ],
            &ctx(&dir.path, "example.com"),
        );
        assert_eq!(out.identity_files.len(), 1);
    }

    #[test]
    fn certificate_files_are_collected_separately() {
        let dir = TempDir::new("cert");
        dir.file(
            "config",
            "IdentityFile ~/.ssh/key\nCertificateFile ~/.ssh/key-cert.pub\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        assert_eq!(out.identity_files.len(), 1);
        assert_eq!(out.certificate_files.len(), 1);
        assert_eq!(out.certificate_files[0].raw, "~/.ssh/key-cert.pub");
    }

    #[test]
    fn unknown_keywords_are_ignored_quietly() {
        let dir = TempDir::new("unknown");
        dir.file(
            "config",
            "ProxyCommand ssh -W %h:%p bastion\nIdentityFile ~/.ssh/key\nServerAliveInterval 30\n",
        );
        let out = parse_user(&dir, "config", "example.com");
        assert_eq!(out.identity_files.len(), 1);
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    }

    #[cfg(unix)]
    #[test]
    fn check_perm_skips_a_group_writable_file() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new("perm");
        let path = dir.file("config", "IdentityFile ~/.ssh/key\n");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o664)).unwrap();

        let out = parse(
            &[ConfigFile {
                path: &path,
                userconf: true,
                check_perm: true,
            }],
            &ctx(&dir.path, "example.com"),
        );
        assert!(out.identity_files.is_empty(), "{:?}", out.identity_files);
        assert!(
            out.warnings
                .iter()
                .any(|w| w.contains("Bad owner or permissions")),
            "warnings: {:?}",
            out.warnings
        );

        // Without CHECKPERM the same file is read (ssh.c:576 does not set the
        // flag for a -F configuration).
        let out = parse(
            &[ConfigFile {
                path: &path,
                userconf: true,
                check_perm: false,
            }],
            &ctx(&dir.path, "example.com"),
        );
        assert_eq!(out.identity_files.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn check_perm_skips_an_included_file_with_bad_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new("perm_include");
        dir.file(".ssh/config", "Include included.conf\n");
        let included = dir.file(".ssh/included.conf", "IdentityFile ~/.ssh/hidden\n");
        std::fs::set_permissions(&included, std::fs::Permissions::from_mode(0o666)).unwrap();

        let out = parse_user(&dir, ".ssh/config", "example.com");
        assert!(out.identity_files.is_empty(), "{:?}", out.identity_files);
        assert!(
            out.warnings
                .iter()
                .any(|w| w.contains("Bad owner or permissions")),
            "warnings: {:?}",
            out.warnings
        );
    }

    #[test]
    fn include_inherits_the_userconf_flag_of_its_parent() {
        let dir = TempDir::new("include_flags");
        dir.file(".ssh/config", "Include included.conf\n");
        dir.file(".ssh/included.conf", "IdentityFile ~/.ssh/from_include\n");
        let out = parse_user(&dir, ".ssh/config", "example.com");
        assert!(out.identity_files[0].user_provided);
    }
}
