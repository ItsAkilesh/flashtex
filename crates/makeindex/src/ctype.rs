//! Character classification used by makeindex's `TOLOWER`/`TOUPPER`
//! macros (`mkind.h` lines 234-237).
//!
//! makeindex never calls `setlocale(LC_ALL, "")`.  Sorting (`sortid.c`) and
//! the heading of the very first entry (`genind.c` `make_entry`, n == 0)
//! therefore run in the "C" ctype.  `genind.c` `new_entry` (lines 219-220)
//! switches `LC_CTYPE` to the environment locale before computing the first
//! letter of a group and printing its heading, so every later heading uses
//! the environment ctype.  On glibc and in `LC_ALL=C` that is plain ASCII;
//! on macOS `en_US.UTF-8` single bytes 0x80..0xFF are classified as their
//! Latin-1 code points (verified against MacTeX makeindex 2.18, see
//! `tests/fixtures/locale-*`).

/// Which `LC_CTYPE` table `genind.c` `new_entry` sees.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CtypeLocale {
    /// "C"/POSIX, glibc UTF-8 locales: only ASCII letters have case.
    #[default]
    C,
    /// macOS libc UTF-8 locales: bytes behave like Latin-1 code points.
    DarwinUtf8,
}

impl CtypeLocale {
    pub fn is_upper(self, c: u8) -> bool {
        match self {
            CtypeLocale::C => c.is_ascii_uppercase(),
            CtypeLocale::DarwinUtf8 => {
                c.is_ascii_uppercase() || ((0xC0..=0xDE).contains(&c) && c != 0xD7)
            }
        }
    }

    fn raw_tolower(self, c: u8) -> u8 {
        if self.is_upper(c) { c + 0x20 } else { c }
    }

    fn raw_toupper(self, c: u8) -> u8 {
        match self {
            CtypeLocale::C => c.to_ascii_uppercase(),
            CtypeLocale::DarwinUtf8 => match c {
                b'a'..=b'z' => c - 0x20,
                0xE0..=0xFE if c != 0xF7 => c - 0x20,
                // U+00FF -> U+0178, truncated to unsigned char.
                0xFF => 0x78,
                // U+00B5 -> U+039C, truncated to unsigned char.
                0xB5 => 0x9C,
                _ => c,
            },
        }
    }

    /// `TOLOWER(C)`: `isupper(C) ? tolower(C) : C`.
    pub fn tolower(self, c: u8) -> u8 {
        if self.is_upper(c) { self.raw_tolower(c) } else { c }
    }

    /// `TOUPPER(C)`: `isupper(C) ? C : toupper(C)`.
    pub fn toupper(self, c: u8) -> u8 {
        if self.is_upper(c) { c } else { self.raw_toupper(c) }
    }
}
