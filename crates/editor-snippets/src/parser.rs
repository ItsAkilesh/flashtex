//! The snippet scanner.
//!
//! # UTF-8 safety
//!
//! The scanner never slices at an arbitrary byte offset. It tracks a
//! cursor (`self.pos`) that only ever advances:
//! - one byte at a time, but *only* over bytes that are not one of the
//!   ASCII structural characters (`$`, `\`, `}`); and
//! - across a structural character it has explicitly matched, which is
//!   always exactly one byte (all of `$ \ { } :` are single-byte ASCII).
//!
//! Because UTF-8 continuation bytes (`0x80..=0xBF`) can never equal an
//! ASCII byte, none of `$ \ { } :` can ever appear as a continuation byte
//! of a multi-byte character. That means every position where the cursor
//! stops - to slice a text run, to look ahead, or to report an error
//! offset - is guaranteed to be a `char` boundary, for any valid UTF-8
//! input, without needing a runtime `is_char_boundary` check anywhere in
//! this file. Multi-byte text (accented Latin, CJK, emoji, ...) that isn't
//! itself one of those five ASCII characters is simply copied through
//! whole inside a text run and never inspected byte-by-byte.

use crate::Snippet;
use crate::error::SnippetError;
use crate::limits::{MAX_INPUT_BYTES, MAX_NESTING_DEPTH, MAX_PLACEHOLDER_INDEX};
use crate::model::Segment;

/// Parses `source` into a [`Snippet`]. See [`Snippet::parse`] for the
/// supported syntax.
pub(crate) fn parse(source: &str) -> Result<Snippet, SnippetError> {
    if source.len() > MAX_INPUT_BYTES {
        return Err(SnippetError::InputTooLarge { len: source.len() });
    }
    let mut parser = Parser {
        src: source,
        bytes: source.as_bytes(),
        pos: 0,
    };
    let segments = parser.parse_segments(0, false)?;
    Ok(Snippet { segments })
}

struct Parser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    /// Parses a run of segments. `depth` is the current placeholder-default
    /// nesting depth (see [`MAX_NESTING_DEPTH`]). When `in_default` is
    /// true, an unescaped `}` ends the run (and is left unconsumed, for the
    /// caller to match against the placeholder's opening `${`); at the top
    /// level `in_default` is false and a stray `}` is just literal text.
    fn parse_segments(
        &mut self,
        depth: usize,
        in_default: bool,
    ) -> Result<Vec<Segment>, SnippetError> {
        let mut out = Vec::new();
        while let Some(&b) = self.bytes.get(self.pos) {
            if in_default && b == b'}' {
                break;
            }
            match b {
                b'\\' => self.parse_escape(&mut out)?,
                b'$' => self.parse_dollar(depth, &mut out)?,
                _ => self.parse_text_run(in_default, &mut out),
            }
        }
        Ok(out)
    }

    /// Consumes the maximal run of bytes that are not a structural
    /// character, and appends it as (or onto) a `Text` segment. `start` and
    /// `self.pos` afterwards both land on `char` boundaries: see the
    /// module-level UTF-8 safety note.
    fn parse_text_run(&mut self, in_default: bool, out: &mut Vec<Segment>) {
        let start = self.pos;
        while let Some(&b) = self.bytes.get(self.pos) {
            if b == b'$' || b == b'\\' || (in_default && b == b'}') {
                break;
            }
            self.pos += 1;
        }
        if self.pos > start {
            push_text(out, &self.src[start..self.pos]);
        }
    }

    fn parse_escape(&mut self, out: &mut Vec<Segment>) -> Result<(), SnippetError> {
        let offset = self.pos;
        self.pos += 1; // consume '\'
        let Some(&b) = self.bytes.get(self.pos) else {
            return Err(SnippetError::UnterminatedEscape { offset });
        };
        match b {
            b'$' | b'}' | b'\\' => {
                push_text(out, &self.src[self.pos..self.pos + 1]);
                self.pos += 1;
                Ok(())
            }
            _ => {
                // `self.pos` is a char boundary (see module docs), so this
                // decodes the full escaped character even if it is
                // multi-byte, for a precise error message.
                let found = self.src[self.pos..]
                    .chars()
                    .next()
                    .expect("non-empty slice");
                Err(SnippetError::InvalidEscape { offset, found })
            }
        }
    }

    fn parse_dollar(&mut self, depth: usize, out: &mut Vec<Segment>) -> Result<(), SnippetError> {
        let dollar_offset = self.pos;
        self.pos += 1; // consume '$'
        match self.bytes.get(self.pos) {
            Some(b) if b.is_ascii_digit() => {
                let index = self.parse_index(dollar_offset)?;
                out.push(Segment::Placeholder {
                    index,
                    default: None,
                });
                Ok(())
            }
            Some(b'{') => {
                self.pos += 1; // consume '{'
                let index = self.parse_index(dollar_offset)?;
                match self.bytes.get(self.pos) {
                    Some(b'}') => {
                        self.pos += 1;
                        out.push(Segment::Placeholder {
                            index,
                            default: None,
                        });
                        Ok(())
                    }
                    Some(b':') => {
                        self.pos += 1; // consume ':'
                        let new_depth = depth + 1;
                        if new_depth > MAX_NESTING_DEPTH {
                            return Err(SnippetError::NestingTooDeep {
                                offset: dollar_offset,
                            });
                        }
                        let default_segments = self.parse_segments(new_depth, true)?;
                        match self.bytes.get(self.pos) {
                            Some(b'}') => {
                                self.pos += 1;
                                out.push(Segment::Placeholder {
                                    index,
                                    default: Some(default_segments),
                                });
                                Ok(())
                            }
                            _ => Err(SnippetError::UnterminatedPlaceholder {
                                offset: dollar_offset,
                            }),
                        }
                    }
                    _ => Err(SnippetError::UnterminatedPlaceholder {
                        offset: dollar_offset,
                    }),
                }
            }
            _ => {
                // A lone '$' not followed by a digit or '{' is literal text.
                push_text(out, "$");
                Ok(())
            }
        }
    }

    /// Parses one or more ASCII decimal digits at the current position into
    /// a placeholder index, enforcing [`MAX_PLACEHOLDER_INDEX`]. Always
    /// leaves `self.pos` just past the digits it consumed, so parsing state
    /// stays consistent even when it returns an error.
    fn parse_index(&mut self, dollar_offset: usize) -> Result<u32, SnippetError> {
        let start = self.pos;
        // Cap how many digits we will collect before we know the index is
        // already too large; ten decimal digits safely exceeds u32::MAX,
        // so this bound never rejects a value that would otherwise fit.
        const MAX_DIGITS: usize = 10;
        let mut too_many_digits = false;
        while matches!(self.bytes.get(self.pos), Some(b) if b.is_ascii_digit()) {
            self.pos += 1;
            if self.pos - start > MAX_DIGITS {
                too_many_digits = true;
            }
        }
        if self.pos == start {
            return Err(SnippetError::InvalidPlaceholderIndex { offset: self.pos });
        }
        if too_many_digits {
            return Err(SnippetError::PlaceholderIndexTooLarge {
                offset: dollar_offset,
            });
        }
        let digits = &self.src[start..self.pos];
        match digits.parse::<u64>() {
            Ok(value) if value <= MAX_PLACEHOLDER_INDEX as u64 => Ok(value as u32),
            _ => Err(SnippetError::PlaceholderIndexTooLarge {
                offset: dollar_offset,
            }),
        }
    }
}

fn push_text(out: &mut Vec<Segment>, s: &str) {
    if let Some(Segment::Text(last)) = out.last_mut() {
        last.push_str(s);
    } else {
        out.push(Segment::Text(s.to_string()));
    }
}
