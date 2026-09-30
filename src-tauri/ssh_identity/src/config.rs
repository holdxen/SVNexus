//! `~/.ssh/config` parsing, limited to what changes identity selection.
//!
//! This mirrors `readconf.c process_config_line_depth()` + the `Host` /
//! `Match` / `Include` handling around it, and deliberately implements only
//! the directives that affect *which keys get tried*:
//!
//! * `IdentityFile` / `CertificateFile`
//! * `IdentitiesOnly`
//! * `Host` / `Match` block scoping
//! * `Include`
//!
//! Everything else (ProxyJump, LocalForward, …) is silently ignored — this
//! crate never opens a connection, it only decides which paths to try.

use std::path::Path;

use crate::expand::{percent_expand, tilde_expand, ExpandContext};
use crate::glob::{glob, match_pattern};

/// `readconf.c:2617 READCONF_MAX_DEPTH`.
const READCONF_MAX_DEPTH: usize = 16;
/// `ssh.h:28 SSH_MAX_IDENTITY_FILES`.
const SSH_MAX_IDENTITY_FILES: usize = 100;

/// A path exactly as written in the config (before `~`/`%` expansion).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RawPath {
    pub raw: String,
    /// `identity_file_userprovided[]` — set for config lines, clear for the
    /// built-in defaults. OpenSSH logs `no such identity` at a different
    /// level depending on it.
    pub user_provided: bool,
}

#[derive(Debug, Default)]
pub(crate) struct RawConfig {
    pub identity_files: Vec<RawPath>,
    pub certificate_files: Vec<RawPath>,
    /// `None` when the config never mentioned `IdentitiesOnly`.
    pub identities_only: Option<bool>,
    pub warnings: Vec<String>,
}

/// Parse `path` (which may not exist) for the target described by `ctx`.
pub(crate) fn parse_config(path: &Path, ctx: &ExpandContext<'_>) -> RawConfig {
    let mut parser = Parser {
        ctx,
        out: RawConfig::default(),
        active: true,
    };

    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        // read_config_file_depth(): a config file that cannot be opened is
        // not an error, it just means "no options".
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return parser.out,
        Err(e) => {
            parser.out.warnings.push(format!("can't read {}: {e}", path.display()));
            return parser.out;
        }
    };

    parser.parse_content(&content, path, 0, false);
    parser.out
}

struct Parser<'a> {
    ctx: &'a ExpandContext<'a>,
    out: RawConfig,
    /// The `activep` flag of readconf.c: options are applied only while it
    /// is set.
    active: bool,
}

impl<'a> Parser<'a> {
    fn warn(&mut self, message: String) {
        self.out.warnings.push(message);
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
        match keyword.to_ascii_lowercase().as_str() {
            "host" => self.parse_host(args, file, lineno, never_match),
            "match" => self.parse_match(args, file, lineno, never_match),
            "identityfile" => self.parse_path(args, file, lineno, false),
            "certificatefile" => self.parse_path(args, file, lineno, true),
            "identitiesonly" => self.parse_identities_only(args, file, lineno),
            "include" => self.parse_include(args, file, lineno, depth),
            _ => {}
        }
    }

    /// readconf.c `case oHost:`.
    fn parse_host(&mut self, args: &[String], file: &Path, lineno: usize, never_match: bool) {
        if args.is_empty() {
            self.active = false;
            self.warn(format!("{}:{lineno}: Host without a pattern", file.display()));
            return;
        }
        // activep is cleared first and only re-set by a positive match.
        self.active = false;
        if never_match {
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
            if match_pattern(self.ctx.host, pattern) {
                if negated {
                    // A negated match vetoes the whole line immediately.
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
    /// the block is **deactivated**. Refusing to apply options whose
    /// condition we cannot evaluate is the fail-safe choice: fewer
    /// candidate keys are tried, never wrong ones.
    fn parse_match(&mut self, args: &[String], file: &Path, lineno: usize, never_match: bool) {
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

    /// readconf.c `case oIdentityFile:` / `case oCertificateFile:`.
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
            user_provided: true,
        };
        let (list, what) = if certificate {
            (&mut self.out.certificate_files, "certificate")
        } else {
            (&mut self.out.identity_files, "identity")
        };
        if !push_dedup(list, entry) {
            self.warn(format!(
                "{}:{lineno}: too many {what} files specified (max {SSH_MAX_IDENTITY_FILES})",
                file.display()
            ));
        }
    }

    /// readconf.c `case oIdentitiesOnly:` → `parse_flag`, first value wins.
    fn parse_identities_only(&mut self, args: &[String], file: &Path, lineno: usize) {
        let Some(arg) = args.first() else {
            self.warn(format!("{}:{lineno}: missing argument", file.display()));
            return;
        };
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

    /// readconf.c `case oInclude:`.
    fn parse_include(&mut self, args: &[String], file: &Path, lineno: usize, depth: usize) {
        let Some(arg) = args.first() else {
            self.warn(format!("{}:{lineno}: missing argument", file.display()));
            return;
        };

        // %tokens and ${ENV} first, then a leading `~`.
        let expanded = match percent_expand(arg, self.ctx) {
            Ok(expanded) => tilde_expand(&expanded, self.ctx.home),
            Err(e) => {
                self.warn(format!("{}:{lineno}: include {arg}: {e}", file.display()));
                return;
            }
        };
        // Relative paths live in ~/.ssh for user configuration files.
        let pattern = if Path::new(&expanded).is_absolute() || expanded.starts_with('~') {
            expanded
        } else {
            self.ctx
                .home
                .join(".ssh")
                .join(expanded)
                .to_string_lossy()
                .into_owned()
        };

        // GLOB_NOMATCH is not an error: a missing include is simply nothing.
        for matched in glob(&pattern) {
            self.parse_included_file(&matched, depth + 1);
        }
    }

    fn parse_included_file(&mut self, path: &Path, depth: usize) {
        let saved = self.active;
        if depth > READCONF_MAX_DEPTH {
            self.warn(format!(
                "Too many recursive configuration includes: {}",
                path.display()
            ));
            return;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
            Err(e) => {
                self.warn(format!("Can't open included config {}: {e}", path.display()));
                return;
            }
        };

        // readconf.c: an Include inside an inactive block is parsed with
        // SSHCONF_NEVERMATCH (nothing may match), and activep is restored
        // afterwards so that Host/Match inside the include cannot clobber
        // the state of the containing file.
        self.parse_content(&content, path, depth, !saved);
        self.active = saved;
    }
}

fn push_dedup(list: &mut Vec<RawPath>, entry: RawPath) -> bool {
    // add_identity_file(): duplicate raw path + same origin is ignored.
    if list
        .iter()
        .any(|known| known.raw == entry.raw && known.user_provided == entry.user_provided)
    {
        return true;
    }
    if list.len() >= SSH_MAX_IDENTITY_FILES {
        return false;
    }
    list.push(entry);
    true
}

fn parse_flag(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" => Some(true),
        "no" | "false" | "off" => Some(false),
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

    #[test]
    fn missing_config_is_not_an_error() {
        let dir = TempDir::new("missing_config");
        let out = parse_config(&dir.path("nope.config"), &ctx(&dir.path, "example.com"));
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
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
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
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "good.example.com"));
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/example", "~/.ssh/any"]);

        // The negated pattern vetoes the first block, but `Host
        // bad.example.com` matches on its own — `~/.ssh/example` must stay
        // out while `~/.ssh/bad` gets in.
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "bad.example.com"));
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/bad", "~/.ssh/any"]);
    }

    #[test]
    fn host_matching_follows_match_pattern() {
        let dir = TempDir::new("host2");
        dir.file("config", "Host Git*.com\nIdentityFile ~/.ssh/key\n");
        let lower = "github.com";
        // match_pattern() is case sensitive (match.c:82).
        assert!(parse_config(&dir.path("config"), &ctx(&dir.path, lower))
            .identity_files
            .is_empty());
        assert_eq!(
            parse_config(&dir.path("config"), &ctx(&dir.path, "GitHub.com"))
                .identity_files
                .len(),
            1
        );
    }

    #[test]
    fn match_all_activates_and_other_match_is_ignored() {
        let dir = TempDir::new("match");
        dir.file(
            "config",
            "Match all\nIdentityFile ~/.ssh/matched\nMatch host foo\nIdentityFile ~/.ssh/never\n",
        );
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/matched"]);
        assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
        assert!(out.warnings[0].contains("unsupported Match condition"));
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
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
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
        dir.file("config", "IdentityFile \"unterminated\nIdentityFile ~/.ssh/ok\n");
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert_eq!(out.identity_files.len(), 1);
        assert!(out.warnings.iter().any(|w| w.contains("invalid quotes")));
    }

    #[test]
    fn identities_only_first_value_wins() {
        let dir = TempDir::new("io");
        dir.file("config", "IdentitiesOnly yes\nIdentitiesOnly no\n");
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert_eq!(out.identities_only, Some(true));

        dir.file("config", "Host other\nIdentitiesOnly yes\n");
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert_eq!(out.identities_only, None);

        dir.file("config", "IdentitiesOnly maybe\n");
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert_eq!(out.identities_only, None);
        assert!(out.warnings.iter().any(|w| w.contains("unsupported option")));
    }

    #[test]
    fn dedup_keeps_first_occurrence() {
        let dir = TempDir::new("dedup");
        dir.file(
            "config",
            "IdentityFile ~/.ssh/key\nIdentityFile ~/.ssh/key\nIdentityFile ~/.ssh/key.pub\n",
        );
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert_eq!(out.identity_files.len(), 2);
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

        let out = parse_config(&dir.path(".ssh/config"), &ctx(&dir.path, "example.com"));
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
        let out = parse_config(&dir.path(".ssh/config"), &ctx(&dir.path, "example.com"));
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
        dir.file(".ssh/included.conf", "Host other\nIdentityFile ~/.ssh/other\n");
        let out = parse_config(&dir.path(".ssh/config"), &ctx(&dir.path, "example.com"));
        let raw: Vec<&str> = out.identity_files.iter().map(|e| e.raw.as_str()).collect();
        assert_eq!(raw, ["~/.ssh/after_include"], "warnings: {:?}", out.warnings);
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

        let out = parse_config(&dir.path(".ssh/config"), &ctx(&dir.path, "example.com"));
        assert!(
            out.warnings.iter().any(|w| w.contains("Too many recursive")),
            "warnings: {:?}",
            out.warnings
        );
        assert!(out.identity_files.is_empty());
    }

    #[test]
    fn certificate_files_are_collected_separately() {
        let dir = TempDir::new("cert");
        dir.file(
            "config",
            "IdentityFile ~/.ssh/key\nCertificateFile ~/.ssh/key-cert.pub\n",
        );
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert_eq!(out.identity_files.len(), 1);
        assert_eq!(out.certificate_files.len(), 1);
        assert_eq!(out.certificate_files[0].raw, "~/.ssh/key-cert.pub");
    }

    #[test]
    fn unknown_keywords_are_ignored_quietly() {
        let dir = TempDir::new("unknown");
        dir.file(
            "config",
            "ProxyJump bastion\nIdentityFile ~/.ssh/key\nServerAliveInterval 30\n",
        );
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert_eq!(out.identity_files.len(), 1);
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    }

    #[test]
    fn user_provided_flag_is_set_by_config() {
        let dir = TempDir::new("user_provided");
        dir.file("config", "IdentityFile ~/.ssh/key\n");
        let out = parse_config(&dir.path("config"), &ctx(&dir.path, "example.com"));
        assert!(out.identity_files[0].user_provided);
    }
}
