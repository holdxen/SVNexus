use crate::parser::parse_line;
use crate::types::{HostEntry, ParseError};
use crate::writer::{check_salt, WriteError, WriteStyle};

/// One line of a known_hosts file, preserving information that a bare
/// `parse()` call discards (comments, blank lines, invalid lines).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocLine {
    /// A blank / whitespace-only line (rendered as an empty line).
    Blank,
    /// A `#` comment line, stored verbatim (without the newline).
    Comment(String),
    /// A successfully parsed entry.
    Entry(HostEntry),
    /// A line that failed to parse; `ParseError::line` holds the original
    /// text so it can be written back unchanged (OpenSSH keeps invalid
    /// lines too).
    Invalid(ParseError),
}

/// A whole known_hosts file as an ordered list of lines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub lines: Vec<DocLine>,
}

/// Parse an entire known_hosts file into a line-preserving [`Document`].
///
/// Unlike [`parse`](crate::parse), comments, blank lines and unparseable
/// lines are kept, so a `WriteStyle::Preserve` render is byte-exact
/// (except that entry lines are stored in trimmed form and a final
/// newline is always emitted).
pub fn parse_document(content: &str) -> Document {
    let mut lines = Vec::new();
    for (idx, raw) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            lines.push(DocLine::Blank);
        } else if trimmed.starts_with('#') {
            lines.push(DocLine::Comment(raw.to_string()));
        } else {
            match parse_line(trimmed, line_num) {
                Ok(mut entry) => {
                    entry.raw_line = raw.to_string();
                    lines.push(DocLine::Entry(entry));
                }
                Err(mut err) => {
                    err.line = raw.to_string();
                    lines.push(DocLine::Invalid(err));
                }
            }
        }
    }
    Document { lines }
}

impl Document {
    /// Render the document; every output line is `\n`-terminated.
    pub fn render(&self, style: WriteStyle) -> Result<String, WriteError> {
        self.render_impl(style, None)
    }

    /// Like [`render`](Self::render), but with a caller-supplied salt for
    /// deterministic hashing. Salt must be 20 bytes.
    pub fn render_with_salt(
        &self,
        style: WriteStyle,
        salt: &[u8],
    ) -> Result<String, WriteError> {
        check_salt(salt)?;
        self.render_impl(style, Some(salt))
    }

    fn render_impl(
        &self,
        style: WriteStyle,
        salt: Option<&[u8]>,
    ) -> Result<String, WriteError> {
        let mut out = String::new();
        for line in &self.lines {
            match line {
                DocLine::Blank => out.push('\n'),
                DocLine::Comment(text) => {
                    out.push_str(text);
                    out.push('\n');
                }
                DocLine::Entry(entry) => {
                    let lines = match salt {
                        Some(salt) => entry.to_lines_with_salt(style, salt)?,
                        None => entry.to_lines(style)?,
                    };
                    for text in lines {
                        out.push_str(&text);
                        out.push('\n');
                    }
                }
                DocLine::Invalid(err) => {
                    out.push_str(&err.line);
                    out.push('\n');
                }
            }
        }
        Ok(out)
    }

    /// Entries in file order (comments/blank/invalid lines skipped).
    pub fn entries(&self) -> Vec<&HostEntry> {
        self.lines
            .iter()
            .filter_map(|l| match l {
                DocLine::Entry(e) => Some(e),
                _ => None,
            })
            .collect()
    }
}
