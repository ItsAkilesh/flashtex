//! `.ist` index style files: defaults (`scanst.h`) and the parser
//! (`scanst.c` `scan_sty`, `scan_spec`, `scan_string`, `scan_char`,
//! `process_precedence`).

use crate::io::{EOF, LFD, Reader, SPC, TAB, Transcript, cat, cstr};

pub(crate) const ARRAY_MAX: usize = 1024;
pub(crate) const STRING_MAX: usize = 999;
pub(crate) const FIELD_MAX: usize = 3;
pub(crate) const PAGETYPE_MAX: usize = 5;

pub(crate) const ROML: usize = 0;
pub(crate) const ROMU: usize = 1;
pub(crate) const ARAB: usize = 2;
pub(crate) const ALPL: usize = 3;
pub(crate) const ALPU: usize = 4;

const ROMAN_LOWER_OFFSET: i32 = 10000;
const ROMAN_UPPER_OFFSET: i32 = 10000;
const ARABIC_OFFSET: i32 = 10000;
const ALPHA_LOWER_OFFSET: i32 = 26;
const ALPHA_UPPER_OFFSET: i32 = 26;

/// Every attribute makeindex 2.18 understands, with `scanst.h` defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Style {
    pub keyword: Vec<u8>,
    pub arg_open: u8,
    pub arg_close: u8,
    pub range_open: u8,
    pub range_close: u8,
    pub level: u8,
    pub quote: u8,
    pub actual: u8,
    pub encap: u8,
    pub escape: u8,

    pub preamble: Vec<u8>,
    pub postamble: Vec<u8>,
    pub prelen: i32,
    pub postlen: i32,
    pub setpage_prefix: Vec<u8>,
    pub setpage_suffix: Vec<u8>,
    pub setpagelen: i32,
    pub group_skip: Vec<u8>,
    pub skiplen: i32,
    pub headings_flag: i32,
    pub heading_prefix: Vec<u8>,
    pub heading_suffix: Vec<u8>,
    pub headprelen: i32,
    pub headsuflen: i32,
    pub symhead_positive: Vec<u8>,
    pub symhead_negative: Vec<u8>,
    pub numhead_positive: Vec<u8>,
    pub numhead_negative: Vec<u8>,

    /// `item_0`, `item_1`, `item_2`.
    pub item_r: [Vec<u8>; FIELD_MAX],
    /// `item_01`, `item_12` (index 0 unused).
    pub item_u: [Vec<u8>; FIELD_MAX],
    /// `item_x1`, `item_x2` (index 0 unused).
    pub item_x: [Vec<u8>; FIELD_MAX],
    pub ilen_r: [i32; FIELD_MAX],
    pub ilen_u: [i32; FIELD_MAX],
    pub ilen_x: [i32; FIELD_MAX],

    /// `delim_0`, `delim_1`, `delim_2`.
    pub delim_p: [Vec<u8>; FIELD_MAX],
    pub delim_n: Vec<u8>,
    pub delim_r: Vec<u8>,
    pub delim_t: Vec<u8>,
    pub suffix_2p: Vec<u8>,
    pub suffix_3p: Vec<u8>,
    pub suffix_mp: Vec<u8>,

    pub encap_prefix: Vec<u8>,
    pub encap_infix: Vec<u8>,
    pub encap_suffix: Vec<u8>,

    pub line_max: i32,
    pub indent_space: Vec<u8>,
    pub indent_length: i32,

    pub page_compositor: Vec<u8>,
    /// Indexed by ROML, ROMU, ARAB, ALPL, ALPU.
    pub page_offset: [i32; PAGETYPE_MAX],
    pub page_precedence: Vec<u8>,
}

fn b(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

impl Default for Style {
    fn default() -> Self {
        Style {
            keyword: b("\\indexentry"),
            arg_open: b'{',
            arg_close: b'}',
            range_open: b'(',
            range_close: b')',
            level: b'!',
            quote: b'"',
            actual: b'@',
            encap: b'|',
            escape: b'\\',
            preamble: b("\\begin{theindex}\n"),
            postamble: b("\n\n\\end{theindex}\n"),
            prelen: 1,
            postlen: 3,
            setpage_prefix: b("\n  \\setcounter{page}{"),
            setpage_suffix: b("}\n"),
            setpagelen: 2,
            group_skip: b("\n\n  \\indexspace\n"),
            skiplen: 3,
            headings_flag: 0,
            heading_prefix: Vec::new(),
            heading_suffix: Vec::new(),
            headprelen: 0,
            headsuflen: 0,
            symhead_positive: b("Symbols"),
            symhead_negative: b("symbols"),
            numhead_positive: b("Numbers"),
            numhead_negative: b("numbers"),
            item_r: [b("\n  \\item "), b("\n    \\subitem "), b("\n      \\subsubitem ")],
            item_u: [Vec::new(), b("\n    \\subitem "), b("\n      \\subsubitem ")],
            item_x: [Vec::new(), b("\n    \\subitem "), b("\n      \\subsubitem ")],
            ilen_r: [1, 1, 1],
            ilen_u: [0, 1, 1],
            ilen_x: [0, 1, 1],
            delim_p: [b(", "), b(", "), b(", ")],
            delim_n: b(", "),
            delim_r: b("--"),
            delim_t: Vec::new(),
            suffix_2p: Vec::new(),
            suffix_3p: Vec::new(),
            suffix_mp: Vec::new(),
            encap_prefix: b("\\"),
            encap_infix: b("{"),
            encap_suffix: b("}"),
            line_max: 72,
            indent_space: b("\t\t"),
            indent_length: 16,
            page_compositor: b("-"),
            // Static initializer in scanst.c lines 97-103 (note: this is
            // *not* what process_precedence derives from "rnaRA").
            page_offset: [
                0,
                ROMAN_LOWER_OFFSET,
                ROMAN_LOWER_OFFSET + ARABIC_OFFSET,
                ROMAN_LOWER_OFFSET + ARABIC_OFFSET + ALPHA_LOWER_OFFSET,
                ROMAN_LOWER_OFFSET + ARABIC_OFFSET + ALPHA_LOWER_OFFSET + ROMAN_UPPER_OFFSET,
            ],
            page_precedence: b("rnaRA"),
        }
    }
}

fn count_lfd(s: &[u8]) -> i32 {
    s.iter().filter(|&&c| c == b'\n').count() as i32
}

enum Spec {
    None,
    Some(Vec<u8>),
}

struct StyleScanner<'a, 'b> {
    rd: Reader<'a>,
    log: &'b mut Transcript,
    name: &'b [u8],
    lc: i32,
    tc: i32,
    ec: i32,
    put_dot: bool,
}

impl StyleScanner<'_, '_> {
    fn error(&mut self, msg: &[u8]) {
        self.log.break_dots();
        let head = format!("** Input style error (file = {}, line = {}):\n   -- ", String::from_utf8_lossy(self.name), self.lc);
        self.log.puts(&head);
        self.log.put(msg);
        self.ec += 1;
        self.put_dot = false;
    }

    fn skipline(&mut self) {
        loop {
            let a = self.rd.getc();
            if a == LFD || a == EOF {
                break;
            }
        }
        self.lc += 1;
    }

    fn next_nonblank(&mut self) -> i32 {
        loop {
            let c = self.rd.getc();
            match c {
                EOF => return -1,
                LFD => self.lc += 1,
                SPC | TAB => {}
                _ => return c,
            }
        }
    }

    /// Returns `None` for FALSE; `Some(spec)` for TRUE or the -1 return.
    fn scan_spec(&mut self) -> Spec {
        let mut c;
        loop {
            c = self.next_nonblank();
            if c == -1 {
                return Spec::None;
            } else if c == b'%' as i32 {
                self.skipline();
            } else {
                break;
            }
        }
        let mut spec = vec![(c as u8).to_ascii_lowercase()];
        let mut i = 0usize;
        loop {
            let ok = i < STRING_MAX;
            i += 1;
            if !ok {
                break;
            }
            c = self.rd.getc();
            if c == SPC || c == TAB || c == LFD || c == EOF {
                break;
            }
            spec.push((c as u8).to_ascii_lowercase());
        }
        if i < STRING_MAX {
            spec.truncate(i);
            if c == EOF {
                let m = cat(&[b"No attribute for specifier ", cstr(&spec), b" (premature EOF)\n"]);
                self.error(&m);
                return Spec::Some(spec);
            }
            if c == LFD {
                self.lc += 1;
            }
            Spec::Some(spec)
        } else {
            let m = cat(&[b"Specifier ", cstr(&spec), format!(" too long (max {}).\n", STRING_MAX).as_bytes()]);
            self.error(&m);
            Spec::None
        }
    }

    fn scan_string(&mut self, dst: &mut Vec<u8>) {
        let c = self.next_nonblank();
        if c == b'"' as i32 {
            let mut clone: Vec<u8> = Vec::new();
            loop {
                let c = self.rd.getc();
                if c == EOF {
                    let m = cat(&[b"No closing delimiter in ", cstr(&clone), b".\n"]);
                    self.error(&m);
                    return;
                } else if c == b'"' as i32 {
                    *dst = cstr(&clone).to_vec();
                    return;
                } else if c == b'\\' as i32 {
                    let c = self.rd.getc();
                    match c {
                        x if x == b't' as i32 => clone.push(b'\t'),
                        x if x == b'n' as i32 => clone.push(b'\n'),
                        _ => clone.push(c as u8),
                    }
                } else {
                    if c == LFD {
                        self.lc += 1;
                    }
                    if clone.len() < ARRAY_MAX {
                        clone.push(c as u8);
                    } else {
                        self.skipline();
                        let m = cat(&[b"Attribute string ", cstr(&clone), format!(" too long (max {}).\n", ARRAY_MAX).as_bytes()]);
                        self.error(&m);
                        return;
                    }
                }
            }
        } else if c == b'%' as i32 {
            self.skipline();
        } else {
            self.skipline();
            self.error(b"No opening delimiter.\n");
        }
    }

    fn scan_char(&mut self, dst: &mut u8) {
        let c = self.next_nonblank();
        if c == b'\'' as i32 {
            let mut clone = self.rd.getc();
            if clone == b'\'' as i32 {
                self.skipline();
                self.error(b"Premature closing delimiter.\n");
                return;
            }
            if clone == LFD || clone == EOF {
                if clone == LFD {
                    self.lc += 1;
                }
                self.error(b"No character (premature EOF).\n");
                return;
            }
            if clone == b'\\' as i32 {
                clone = self.rd.getc();
            }
            if self.rd.getc() == b'\'' as i32 {
                *dst = clone as u8;
            } else {
                self.error(b"No closing delimiter or too many letters.\n");
            }
        } else if c == b'%' as i32 {
            self.skipline();
        } else {
            self.skipline();
            self.error(b"No opening delimiter.\n");
        }
    }

    fn scan_no(&mut self, dst: &mut i32) {
        if let Some(v) = self.rd.scan_int() {
            *dst = v;
        }
    }
}

fn multiple(sc: &mut StyleScanner, ch: u8, prec: &[u8]) {
    sc.skipline();
    let m = cat(&[b"Multiple instances of type `", &[ch], b"' in page precedence specification `", cstr(prec), b"'.\n"]);
    sc.error(&m);
}

fn process_precedence(st: &mut Style, sc: &mut StyleScanner) {
    let prec = st.page_precedence.clone();
    let at = |i: usize| prec.get(i).copied().unwrap_or(0);
    let (mut roml, mut romu, mut arab, mut alpl, mut alpu) = (false, false, false, false, false);
    let mut i = 0;
    while i < PAGETYPE_MAX && at(i) != 0 {
        match at(i) {
            b'r' => {
                if roml {
                    return multiple(sc, b'r', &prec);
                }
                roml = true;
            }
            b'R' => {
                if romu {
                    return multiple(sc, b'R', &prec);
                }
                romu = true;
            }
            b'n' => {
                if arab {
                    return multiple(sc, b'n', &prec);
                }
                arab = true;
            }
            b'a' => {
                if alpl {
                    return multiple(sc, b'A', &prec);
                }
                alpl = true;
            }
            b'A' => {
                if alpu {
                    return multiple(sc, b'A', &prec);
                }
                alpu = true;
            }
            other => {
                sc.skipline();
                let m = cat(&[b"Unknow type `", &[other], b"' in page precedence specification.\n"]);
                sc.error(&m);
                return;
            }
        }
        i += 1;
    }
    if at(i) != 0 {
        sc.skipline();
        sc.error(b"Page precedence specification string too long.\n");
        return;
    }
    let mut last = i;
    if last == 0 {
        // C reads uninitialized order/type; keep the previous offsets.
        return;
    }
    let mut order = [0i32; PAGETYPE_MAX + 1];
    let mut typ = [0usize; PAGETYPE_MAX + 1];
    let (o0, t0) = match at(0) {
        b'r' => (ROMAN_LOWER_OFFSET, ROML),
        b'R' => (ROMAN_UPPER_OFFSET, ROMU),
        b'n' => (ARABIC_OFFSET, ARAB),
        b'a' => (ALPHA_LOWER_OFFSET, ALPL),
        _ => (ALPHA_LOWER_OFFSET, ALPU),
    };
    order[0] = o0;
    typ[0] = t0;
    for i in 1..last {
        let (o, t) = match at(i) {
            b'r' => (ROMAN_LOWER_OFFSET, ROML),
            b'R' => (ROMAN_UPPER_OFFSET, ROMU),
            b'n' => (ARABIC_OFFSET, ARAB),
            b'a' => (ALPHA_LOWER_OFFSET, ALPL),
            _ => (ALPHA_LOWER_OFFSET, ALPU),
        };
        order[i] = order[i - 1] + o;
        typ[i] = t;
    }
    let mut off = [-1i32; PAGETYPE_MAX];
    off[typ[0]] = 0;
    for i in 1..last {
        off[typ[i]] = order[i - 1];
    }
    for i in 0..PAGETYPE_MAX {
        if off[i] == -1 {
            let add = match typ[last - 1] {
                ROML => ROMAN_LOWER_OFFSET,
                ROMU => ROMAN_UPPER_OFFSET,
                ARAB => ARABIC_OFFSET,
                ALPL => ALPHA_LOWER_OFFSET,
                _ => ALPHA_UPPER_OFFSET,
            };
            order[last] = order[last - 1] + add;
            typ[last] = i;
            off[i] = order[last];
            last += 1;
        }
    }
    st.page_offset = off;
}

/// `scan_sty`: apply a style file to `st`, writing messages to the transcript.
pub(crate) fn scan_sty(st: &mut Style, name: &[u8], data: &[u8], log: &mut Transcript) {
    log.put(&cat(&[b"Scanning style file ", name]));
    let mut sc = StyleScanner { rd: Reader::new(data), log, name, lc: 0, tc: 0, ec: 0, put_dot: false };
    loop {
        let spec = match sc.scan_spec() {
            Spec::None => break,
            Spec::Some(s) => s,
        };
        sc.tc += 1;
        sc.put_dot = true;
        let s = cstr(&spec).to_vec();
        macro_rules! string {
            ($f:expr) => {{
                let mut v = std::mem::take(&mut $f);
                sc.scan_string(&mut v);
                $f = v;
            }};
            ($f:expr, $len:expr) => {{
                string!($f);
                $len = count_lfd(&$f);
            }};
        }
        macro_rules! chr {
            ($f:expr) => {{
                let mut v = $f;
                sc.scan_char(&mut v);
                $f = v;
            }};
        }
        match s.as_slice() {
            b"preamble" => string!(st.preamble, st.prelen),
            b"postamble" => string!(st.postamble, st.postlen),
            b"group_skip" => string!(st.group_skip, st.skiplen),
            b"headings_flag" => sc.scan_no(&mut st.headings_flag),
            b"heading_prefix" => string!(st.heading_prefix, st.headprelen),
            b"heading_suffix" => string!(st.heading_suffix, st.headsuflen),
            b"symhead_positive" => string!(st.symhead_positive),
            b"symhead_negative" => string!(st.symhead_negative),
            b"numhead_positive" => string!(st.numhead_positive),
            b"numhead_negative" => string!(st.numhead_negative),
            b"setpage_prefix" => string!(st.setpage_prefix, st.setpagelen),
            b"setpage_suffix" => string!(st.setpage_suffix, st.setpagelen),
            b"item_0" => string!(st.item_r[0], st.ilen_r[0]),
            b"item_1" => string!(st.item_r[1], st.ilen_r[1]),
            b"item_2" => string!(st.item_r[2], st.ilen_r[2]),
            b"item_01" => string!(st.item_u[1], st.ilen_u[1]),
            b"item_12" => string!(st.item_u[2], st.ilen_u[2]),
            b"item_x1" => string!(st.item_x[1], st.ilen_x[1]),
            b"item_x2" => string!(st.item_x[2], st.ilen_x[2]),
            b"encap_prefix" => string!(st.encap_prefix),
            b"encap_infix" => string!(st.encap_infix),
            b"encap_suffix" => string!(st.encap_suffix),
            b"delim_0" => string!(st.delim_p[0]),
            b"delim_1" => string!(st.delim_p[1]),
            b"delim_2" => string!(st.delim_p[2]),
            b"delim_n" => string!(st.delim_n),
            b"delim_r" => string!(st.delim_r),
            b"delim_t" => string!(st.delim_t),
            b"suffix_2p" => string!(st.suffix_2p),
            b"suffix_3p" => string!(st.suffix_3p),
            b"suffix_mp" => string!(st.suffix_mp),
            b"line_max" => {
                let mut tmp = 0;
                sc.scan_no(&mut tmp);
                if tmp > 0 {
                    st.line_max = tmp;
                } else {
                    sc.error(format!("line_max must be positive (got {})", tmp).as_bytes());
                }
            }
            b"indent_length" => {
                let mut tmp = 0;
                sc.scan_no(&mut tmp);
                if tmp >= 0 {
                    st.indent_length = tmp;
                } else {
                    sc.error(format!("indent_length must be nonnegative (got {})", tmp).as_bytes());
                }
            }
            b"indent_space" => string!(st.indent_space),
            b"page_compositor" => string!(st.page_compositor),
            b"page_precedence" => string!(st.page_precedence),
            b"keyword" => string!(st.keyword),
            b"arg_open" => chr!(st.arg_open),
            b"arg_close" => chr!(st.arg_close),
            b"level" => chr!(st.level),
            b"range_open" => chr!(st.range_open),
            b"range_close" => chr!(st.range_close),
            b"quote" => chr!(st.quote),
            b"actual" => chr!(st.actual),
            b"encap" => chr!(st.encap),
            b"escape" => chr!(st.escape),
            _ => {
                sc.next_nonblank();
                sc.skipline();
                let m = cat(&[b"Unknown specifier ", &s, b".\n"]);
                sc.error(&m);
                sc.put_dot = false;
            }
        }
        if sc.put_dot {
            sc.log.idx_dot = true;
            sc.log.put(b".");
        }
    }
    process_precedence(st, &mut sc);
    if st.quote == st.escape {
        let m = cat(&[b"Quote and escape symbols must be distinct (both `", &[st.quote], b"' now).\n"]);
        sc.error(&m);
        st.quote = b'"';
        st.escape = b'\\';
    }
    let done = format!("done ({} attributes redefined, {} ignored).\n", sc.tc - sc.ec, sc.ec);
    sc.log.puts(&done);
}
