//! Test support: a small JSON reader for the committed oracle files (the
//! crate has no external dependencies) and the font locator used by tests.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use flashtex_unicode_tex::locate::{DirectoryLocator, MACOS_FONT_DIRS};

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    pub fn get(&self, k: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.get(k),
            _ => None,
        }
    }
    pub fn str(&self) -> &str {
        match self {
            Json::Str(s) => s,
            _ => "",
        }
    }
    pub fn num(&self) -> f64 {
        match self {
            Json::Num(n) => *n,
            _ => f64::NAN,
        }
    }
    pub fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(a) => a,
            _ => &[],
        }
    }
    pub fn obj(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Obj(m) => Some(m),
            _ => None,
        }
    }
    pub fn bool(&self) -> bool {
        matches!(self, Json::Bool(true))
    }
}

pub fn parse(s: &str) -> Json {
    let mut p = Parser {
        b: s.as_bytes(),
        i: 0,
    };
    let v = p.value();
    p.ws();
    assert_eq!(p.i, p.b.len(), "trailing JSON data");
    v
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn value(&mut self) -> Json {
        self.ws();
        match self.b[self.i] {
            b'{' => {
                self.i += 1;
                let mut m = BTreeMap::new();
                loop {
                    self.ws();
                    if self.b[self.i] == b'}' {
                        self.i += 1;
                        break;
                    }
                    let Json::Str(k) = self.value() else {
                        panic!("object key")
                    };
                    self.ws();
                    assert_eq!(self.b[self.i], b':');
                    self.i += 1;
                    let v = self.value();
                    m.insert(k, v);
                    self.ws();
                    if self.b[self.i] == b',' {
                        self.i += 1;
                    }
                }
                Json::Obj(m)
            }
            b'[' => {
                self.i += 1;
                let mut a = Vec::new();
                loop {
                    self.ws();
                    if self.b[self.i] == b']' {
                        self.i += 1;
                        break;
                    }
                    a.push(self.value());
                    self.ws();
                    if self.b[self.i] == b',' {
                        self.i += 1;
                    }
                }
                Json::Arr(a)
            }
            b'"' => {
                self.i += 1;
                let mut out: Vec<u8> = Vec::new();
                loop {
                    let c = self.b[self.i];
                    self.i += 1;
                    match c {
                        b'"' => break,
                        b'\\' => {
                            let e = self.b[self.i];
                            self.i += 1;
                            match e {
                                b'n' => out.push(b'\n'),
                                b't' => out.push(b'\t'),
                                b'r' => out.push(b'\r'),
                                b'b' => out.push(8),
                                b'f' => out.push(12),
                                b'u' => {
                                    let mut cp = self.hex4();
                                    if (0xD800..0xDC00).contains(&cp) && self.b[self.i] == b'\\' {
                                        self.i += 2;
                                        let lo = self.hex4();
                                        cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                    }
                                    let ch = char::from_u32(cp).unwrap_or('\u{FFFD}');
                                    let mut buf = [0u8; 4];
                                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                                }
                                other => out.push(other),
                            }
                        }
                        other => out.push(other),
                    }
                }
                Json::Str(String::from_utf8(out).expect("utf8"))
            }
            b't' => {
                self.i += 4;
                Json::Bool(true)
            }
            b'f' => {
                self.i += 5;
                Json::Bool(false)
            }
            b'n' => {
                self.i += 4;
                Json::Null
            }
            _ => {
                let start = self.i;
                while self.i < self.b.len()
                    && matches!(
                        self.b[self.i],
                        b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
                    )
                {
                    self.i += 1;
                }
                Json::Num(
                    std::str::from_utf8(&self.b[start..self.i])
                        .unwrap()
                        .parse()
                        .expect("number"),
                )
            }
        }
    }
    fn hex4(&mut self) -> u32 {
        let s = std::str::from_utf8(&self.b[self.i..self.i + 4]).unwrap();
        self.i += 4;
        u32::from_str_radix(s, 16).unwrap()
    }
}

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub const TEXLIVE_OPENTYPE: &str = "/usr/local/texlive/2026/texmf-dist/fonts/opentype/public";

/// macOS system fonts, the Mac app's bundled fonts and the TeX Live OpenType
/// directories the fixtures use. Missing directories are skipped.
pub fn locator() -> DirectoryLocator {
    let mut dirs: Vec<PathBuf> = MACOS_FONT_DIRS.iter().map(PathBuf::from).collect();
    dirs.push(crate_dir().join("../../apps/mac/Fonts"));
    for sub in [
        "tex-gyre",
        "lm",
        "lm-math",
        "stix2-otf",
        "newcomputermodern",
    ] {
        dirs.push(PathBuf::from(TEXLIVE_OPENTYPE).join(sub));
    }
    DirectoryLocator::new(dirs.into_iter().filter(|d| d.is_dir()).collect(), 1)
}
