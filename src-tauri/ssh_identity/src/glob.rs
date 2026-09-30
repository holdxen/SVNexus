//! Pattern matching that mirrors OpenSSH.
//!
//! Two different matchers exist in `openssh-portable`, and they are not
//! interchangeable:
//!
//! * [`match_pattern`] is `match.c:82 match_pattern()` — used for the
//!   patterns of a `Host` line. It understands `*` and `?` only, is
//!   case sensitive and has **no** `[...]` character classes.
//! * [`glob`] is a stand-in for `glob(3)` with `GLOB_TILDE` semantics,
//!   used for the argument of `Include`. It additionally understands
//!   `[...]`, skips dot-files unless the pattern starts with `.`, and
//!   only matches paths that actually exist.

use std::path::{Path, PathBuf};

/// OpenSSH `match_pattern()` (match.c:82): `*` and `?` only, case-sensitive,
/// no character classes.
pub fn match_pattern(text: &str, pattern: &str) -> bool {
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    match_inner(&p, &t, false)
}

fn match_inner(p: &[char], t: &[char], classes: bool) -> bool {
    if p.is_empty() {
        return t.is_empty();
    }
    match p[0] {
        '*' => (0..=t.len()).any(|i| match_inner(&p[1..], &t[i..], classes)),
        '?' => !t.is_empty() && match_inner(&p[1..], &t[1..], classes),
        '[' if classes => match_class(p, t),
        c => !t.is_empty() && t[0] == c && match_inner(&p[1..], &t[1..], classes),
    }
}

/// Match one `[...]` character class starting at `p[0] == '['`.
fn match_class(p: &[char], t: &[char]) -> bool {
    if t.is_empty() {
        return false;
    }
    let mut i = 1;
    let negated = matches!(p.get(i), Some('!') | Some('^'));
    if negated {
        i += 1;
    }
    let start = i;
    // `]` directly after `[` or `[!` is a literal, not the terminator.
    let mut first = true;
    while i < p.len() {
        if p[i] == ']' && !first {
            break;
        }
        first = false;
        i += 1;
    }
    if i >= p.len() {
        // Unterminated class: POSIX treats the bracket as a literal.
        return t[0] == '[' && match_inner(&p[1..], &t[1..], true);
    }
    let inner = &p[start..i];
    let mut hit = false;
    let mut k = 0;
    while k < inner.len() {
        if k + 2 < inner.len() && inner[k + 1] == '-' {
            if t[0] >= inner[k] && t[0] <= inner[k + 2] {
                hit = true;
                break;
            }
            k += 3;
        } else {
            if t[0] == inner[k] {
                hit = true;
                break;
            }
            k += 1;
        }
    }
    if hit == negated {
        return false;
    }
    match_inner(&p[i + 1..], &t[1..], true)
}

fn has_magic(s: &str) -> bool {
    s.contains(['*', '?', '['])
}

/// `glob(3)`-ish expansion: returns every existing path matching `pattern`.
///
/// Patterns without wildcards resolve to a single path only when that path
/// exists — which is what makes a missing `Include` target a non-event
/// (`GLOB_NOMATCH`, readconf.c).
pub fn glob(pattern: &str) -> Vec<PathBuf> {
    if !has_magic(pattern) {
        let p = PathBuf::from(pattern);
        return if p.exists() { vec![p] } else { Vec::new() };
    }

    let comps: Vec<String> = Path::new(pattern)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    let (base, rest) = match comps.first() {
        Some(first) if first == "/" => (PathBuf::from("/"), comps[1..].to_vec()),
        _ => (PathBuf::from("."), comps),
    };

    let mut out = Vec::new();
    expand(&base, &rest, &mut out);
    out.sort();
    out.dedup();
    out
}

fn expand(base: &Path, comps: &[String], out: &mut Vec<PathBuf>) {
    let Some((first, rest)) = comps.split_first() else {
        out.push(base.to_path_buf());
        return;
    };

    if !has_magic(first) {
        let next = base.join(first);
        if rest.is_empty() {
            if next.exists() {
                out.push(next);
            }
        } else if next.is_dir() {
            expand(&next, rest, out);
        }
        return;
    }

    let Ok(dir) = std::fs::read_dir(base) else {
        return;
    };
    let mut names: Vec<PathBuf> = dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            // glob(3) never matches leading dots unless the pattern has one.
            if name.starts_with('.') && !first.starts_with('.') {
                return false;
            }
            let pat: Vec<char> = first.chars().collect();
            let nam: Vec<char> = name.chars().collect();
            match_inner(&pat, &nam, true)
        })
        .collect();
    names.sort();

    for name in names {
        if rest.is_empty() {
            out.push(name);
        } else if name.is_dir() {
            expand(&name, rest, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_util::TempDir;

    #[test]
    fn match_pattern_star_and_question() {
        assert!(match_pattern("github.com", "*.com"));
        assert!(match_pattern("github.com", "git*.com"));
        assert!(match_pattern("github.com", "g?thub.com"));
        assert!(!match_pattern("github.com", "gi??hub.com"));
        assert!(!match_pattern("github.com", "gitlab.com"));
        assert!(!match_pattern("github.com", "*.github.com"));
        assert!(!match_pattern("github.com", ""));
        assert!(match_pattern("", ""));
    }

    #[test]
    fn match_pattern_is_case_sensitive_and_has_no_classes() {
        // match.c match_pattern() neither lowercases nor handles '[...]'.
        assert!(!match_pattern("GitHub.com", "github.com"));
        assert!(!match_pattern("a", "[ab]"));
        // A comma in a Host pattern is a literal, not a list separator:
        // readconf.c oHost passes each whitespace-delimited argument
        // straight to match_pattern().
        assert!(!match_pattern("foo", "foo,bar"));
    }

    #[test]
    fn glob_without_magic_only_matches_existing_paths() {
        let dir = TempDir::new("glob");
        dir.file("a.conf", "");
        assert_eq!(glob(&dir.path.join("a.conf").display().to_string()).len(), 1);
        assert!(glob(&dir.path.join("missing.conf").display().to_string()).is_empty());
    }

    #[test]
    fn glob_star_and_hidden_files() {
        let dir = TempDir::new("glob2");
        dir.file("conf.d/one.conf", "");
        dir.file("conf.d/two.conf", "");
        dir.file("conf.d/.hidden.conf", "");

        let pattern = dir.path.join("conf.d/*.conf").display().to_string();
        let mut got = glob(&pattern);
        got.sort();
        assert_eq!(got.len(), 2, "{got:?}");
        assert!(got[0].to_string_lossy().ends_with("one.conf"));

        // Hidden files need an explicit dot in the pattern.
        let hidden = dir.path.join("conf.d/.hidden.conf").display().to_string();
        let pattern = dir.path.join("conf.d/.*").display().to_string();
        assert_eq!(glob(&pattern).len(), 1, "{hidden}");
    }

    #[test]
    fn glob_char_class() {
        let dir = TempDir::new("glob3");
        dir.file("id1", "");
        dir.file("id2", "");
        dir.file("idx", "");

        let pattern = dir.path.join("id[12]").display().to_string();
        assert_eq!(glob(&pattern).len(), 2);

        let pattern = dir.path.join("id[!12]").display().to_string();
        assert_eq!(glob(&pattern).len(), 1);
    }
}
