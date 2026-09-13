//! Byte readers and the `.ilg` transcript, mirroring the C stdio usage.

pub(crate) const EOF: i32 = -1;
pub(crate) const LFD: i32 = b'\n' as i32;
pub(crate) const TAB: i32 = b'\t' as i32;
pub(crate) const SPC: i32 = b' ' as i32;

/// Promote a `char` style setting the way C does when it is compared with
/// the `int` returned by `getc` (`char` is signed on the reference builds).
pub(crate) fn sc(c: u8) -> i32 {
    c as i8 as i32
}

/// A `FILE*` opened `"rb"` read through `mk_getc` (`mkind.c` lines 81-95).
pub(crate) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    lookahead: i32,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0, lookahead: -2 }
    }

    fn raw(&mut self) -> i32 {
        if self.pos < self.data.len() {
            let c = self.data[self.pos];
            self.pos += 1;
            c as i32
        } else {
            EOF
        }
    }

    /// `mk_getc`: CR LF is read as LF.
    pub fn getc(&mut self) -> i32 {
        let mut ch = if self.lookahead != -2 { self.lookahead } else { self.raw() };
        self.lookahead = if ch == b'\r' as i32 { self.raw() } else { -2 };
        if self.lookahead == LFD {
            ch = LFD;
            self.lookahead = -2;
        }
        ch
    }

    /// `fscanf(fp, "%d", &n)`; returns `None` on a matching failure, leaving
    /// the destination untouched.  Reads bypass the `mk_getc` lookahead just
    /// like the C code does.
    pub fn scan_int(&mut self) -> Option<i32> {
        while self.pos < self.data.len() && matches!(self.data[self.pos], b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') {
            self.pos += 1;
        }
        let mut neg = false;
        if self.pos < self.data.len() && matches!(self.data[self.pos], b'+' | b'-') {
            neg = self.data[self.pos] == b'-';
            self.pos += 1;
        }
        let start = self.pos;
        let mut v: i128 = 0;
        while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            v = (v * 10 + (self.data[self.pos] - b'0') as i128).min(i64::MAX as i128 + 1);
            self.pos += 1;
        }
        if self.pos == start {
            return None;
        }
        let v = if neg { -v } else { v };
        let v = v.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        Some(v as i32)
    }
}

/// The `.ilg` stream plus the global dot bookkeeping (`idx_dot`, `idx_dc`).
pub(crate) struct Transcript {
    pub buf: Vec<u8>,
    pub idx_dot: bool,
    pub idx_dc: i32,
}

impl Transcript {
    pub fn new() -> Self {
        Transcript { buf: Vec::new(), idx_dot: true, idx_dc: 0 }
    }

    pub fn put(&mut self, s: &[u8]) {
        self.buf.extend_from_slice(s);
    }

    pub fn puts(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }

    /// The `if (idx_dot) { "\n"; idx_dot = FALSE; }` prefix of every error.
    pub fn break_dots(&mut self) {
        if self.idx_dot {
            self.buf.push(b'\n');
            self.idx_dot = false;
        }
    }

    /// `IDX_DOT(MAX)` (`mkind.h` lines 512-521).
    pub fn idx_dot(&mut self, max: i32) {
        self.idx_dot = true;
        let was = self.idx_dc;
        self.idx_dc += 1;
        if was == 0 {
            self.buf.push(b'.');
        }
        if self.idx_dc == max {
            self.idx_dc = 0;
        }
    }
}

/// C string view: bytes up to the first NUL.
pub(crate) fn cstr(s: &[u8]) -> &[u8] {
    match s.iter().position(|&b| b == 0) {
        Some(p) => &s[..p],
        None => s,
    }
}

pub(crate) fn cat(parts: &[&[u8]]) -> Vec<u8> {
    let mut v = Vec::new();
    for p in parts {
        v.extend_from_slice(p);
    }
    v
}
