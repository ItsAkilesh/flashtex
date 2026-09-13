//! Faithful port of makeindex 2.18 processing: `scanid.c` (reading `.idx`),
//! `sortid.c` + `qsort.c` (ordering, duplicate detection, comparison count)
//! and `genind.c` (writing `.ind`).  Function names follow the C sources so
//! the two can be read side by side.

use crate::ctype::CtypeLocale;
use crate::io::{EOF, LFD, Reader, SPC, TAB, Transcript, cat, cstr, sc};
use crate::style::{ALPL, ALPU, ARAB, ARRAY_MAX, FIELD_MAX, ROML, ROMU, Style};

const ARGUMENT_MAX: usize = 10240;
const NUMBER_MAX: usize = 99;
const ARABIC_MAX: usize = 99;
const ROMAN_MAX: usize = 99;
const PAGEFIELD_MAX: usize = 10;
const EMPTY: i32 = -9999;
const DUPLICATE: i32 = 9999;
const SYMBOL: i32 = -1;
const ALPHA: i32 = -2;
const DOT_MAX: i32 = 1000;
const CMP_MAX: i32 = 1500;
const THRESH: usize = 4;
const MTHRESH: usize = 6;

#[derive(Clone, Debug)]
pub(crate) struct Entry {
    pub sf: [Vec<u8>; FIELD_MAX],
    pub af: [Vec<u8>; FIELD_MAX],
    pub group: i32,
    pub lpg: Vec<u8>,
    pub npg: [i32; PAGEFIELD_MAX],
    pub count: usize,
    pub typ: i32,
    pub encap: Vec<u8>,
    pub file: usize,
    pub lc: i32,
}

impl Entry {
    fn new() -> Self {
        Entry {
            sf: Default::default(),
            af: Default::default(),
            group: 0,
            lpg: Vec::new(),
            npg: [0; PAGEFIELD_MAX],
            count: 0,
            typ: EMPTY,
            encap: Vec::new(),
            file: 0,
            lc: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Flags {
    pub letter_ordering: bool,
    pub compress_blanks: bool,
    pub merge_page: bool,
    pub german_sort: bool,
    pub ctype: CtypeLocale,
}

fn isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

fn strspn(s: &[u8], set: &[u8]) -> usize {
    s.iter().take_while(|&&c| c != 0 && set.contains(&c)).count()
}

fn strtoint(s: &[u8]) -> i32 {
    let mut v: i32 = 0;
    for &c in cstr(s) {
        v = v.wrapping_mul(10).wrapping_add(c as i32 - 48);
    }
    v
}

/// `group_type` (`scanid.c` lines 406-421).
pub(crate) fn group_type(s: &[u8]) -> i32 {
    let s = cstr(s);
    if s.iter().all(|&c| isdigit(c)) {
        // sscanf("%d"): strtoimax-style saturation, then truncation to int.
        let mut v: i128 = 0;
        for &c in s {
            v = (v * 10 + (c - b'0') as i128).min(i64::MAX as i128);
        }
        return v as i64 as i32;
    }
    let c0 = s[0] as i8 as i32;
    let issym = (b'!' as i32 <= c0 && c0 <= b'@' as i32)
        || (b'[' as i32 <= c0 && c0 <= b'`' as i32)
        || (b'{' as i32 <= c0 && c0 <= b'~' as i32);
    if issym { SYMBOL } else { ALPHA }
}

pub(crate) struct Engine<'s> {
    pub st: &'s Style,
    pub fl: Flags,
    pub log: Transcript,
    pub names: Vec<Vec<u8>>,
    pub entries: Vec<Entry>,
    // scanid.c statics
    idx_lc: i32,
    idx_tc: i32,
    idx_ec: i32,
    pub idx_tt: i32,
    pub idx_et: i32,
    comp_len: usize,
    key: Vec<u8>,
    no: Vec<u8>,
    type_guess: [i32; PAGEFIELD_MAX],
    cur_file: usize,
    // sortid.c
    idx_gc: i64,
}

impl<'s> Engine<'s> {
    pub fn new(st: &'s Style, fl: Flags, log: Transcript) -> Self {
        Engine {
            st,
            fl,
            log,
            names: Vec::new(),
            entries: Vec::new(),
            idx_lc: 0,
            idx_tc: 0,
            idx_ec: 0,
            idx_tt: 0,
            idx_et: 0,
            comp_len: 0,
            key: Vec::new(),
            no: Vec::new(),
            type_guess: [EMPTY; PAGEFIELD_MAX],
            cur_file: 0,
            idx_gc: 0,
        }
    }

    // ------------------------------------------------------------------
    // scanid.c
    // ------------------------------------------------------------------

    fn idx_error(&mut self, msg: &[u8]) {
        self.log.break_dots();
        let head = cat(&[
            b"!! Input index error (file = ",
            &self.names[self.cur_file],
            format!(", line = {}):\n   -- ", self.idx_lc).as_bytes(),
        ]);
        self.log.put(&head);
        self.log.put(msg);
        self.idx_ec += 1;
    }

    fn skipline(rd: &mut Reader, lc: &mut i32) {
        loop {
            let t = rd.getc();
            if t == LFD || t == EOF {
                break;
            }
        }
        *lc += 1;
    }

    fn flush_to_eol(rd: &mut Reader) {
        loop {
            let a = rd.getc();
            if a == LFD || a == EOF {
                break;
            }
        }
    }

    pub fn scan_idx(&mut self, name: &[u8], data: &[u8]) {
        self.names.push(name.to_vec());
        self.cur_file = self.names.len() - 1;
        self.log.put(&cat(&[b"Scanning input file ", name, b"..."]));
        self.idx_lc = 0;
        self.idx_tc = 0;
        self.idx_ec = 0;
        self.log.idx_dc = 0;
        self.comp_len = cstr(&self.st.page_compositor).len();
        let mut rd = Reader::new(data);
        let mut keyword: Vec<u8> = Vec::new();
        let mut arg_count: i32 = -1;
        let mut not_eof = true;
        while not_eof {
            let c = rd.getc();
            match c {
                EOF => {
                    if arg_count == 2 {
                        self.idx_lc += 1;
                        if self.make_key() {
                            self.log.idx_dot(DOT_MAX);
                        }
                        arg_count = -1;
                    } else {
                        if arg_count > -1 {
                            self.idx_lc += 1;
                            self.idx_error(b"Missing arguments -- need two (premature EOF).\n");
                        }
                        not_eof = false;
                    }
                }
                LFD => {
                    self.idx_lc += 1;
                    if arg_count == 2 {
                        if self.make_key() {
                            self.log.idx_dot(DOT_MAX);
                        }
                        arg_count = -1;
                    } else if arg_count > -1 {
                        self.idx_error(b"Missing arguments -- need two (premature LFD).\n");
                        arg_count = -1;
                    }
                }
                TAB | SPC => {}
                _ => match arg_count {
                    -1 => {
                        keyword.clear();
                        keyword.push(c as u8);
                        arg_count += 1;
                        self.idx_tc += 1;
                    }
                    0 => {
                        if c == sc(self.st.arg_open) {
                            arg_count += 1;
                            if cstr(&keyword) == cstr(&self.st.keyword) {
                                if !self.scan_arg1(&mut rd) {
                                    arg_count = -1;
                                }
                            } else {
                                Self::skipline(&mut rd, &mut self.idx_lc);
                                arg_count = -1;
                                let m = cat(&[b"Unknown index keyword ", cstr(&keyword), b".\n"]);
                                self.idx_error(&m);
                            }
                        } else if keyword.len() < ARRAY_MAX {
                            keyword.push(c as u8);
                        } else {
                            Self::skipline(&mut rd, &mut self.idx_lc);
                            arg_count = -1;
                            let m = cat(&[
                                b"Index keyword ",
                                cstr(&keyword),
                                format!(" too long (max {}).\n", ARRAY_MAX).as_bytes(),
                            ]);
                            self.idx_error(&m);
                        }
                    }
                    1 => {
                        if c == sc(self.st.arg_open) {
                            arg_count += 1;
                            if !self.scan_arg2(&mut rd) {
                                arg_count = -1;
                            }
                        } else {
                            Self::skipline(&mut rd, &mut self.idx_lc);
                            arg_count = -1;
                            let m = cat(&[
                                b"No opening delimiter for second argument (illegal character `",
                                &[c as u8],
                                b"').\n",
                            ]);
                            self.idx_error(&m);
                        }
                    }
                    2 => {
                        Self::skipline(&mut rd, &mut self.idx_lc);
                        arg_count = -1;
                        let m = cat(&[
                            b"No closing delimiter for second argument (illegal character `",
                            &[c as u8],
                            b"').\n",
                        ]);
                        self.idx_error(&m);
                    }
                    _ => {}
                },
            }
        }
        self.idx_tt += self.idx_tc;
        self.idx_et += self.idx_ec;
        let done = format!(
            "done ({} entries accepted, {} rejected).\n",
            self.idx_tc - self.idx_ec,
            self.idx_ec
        );
        self.log.puts(&done);
    }

    fn scan_arg1(&mut self, rd: &mut Reader) -> bool {
        let st = self.st;
        let mut a;
        if self.fl.compress_blanks {
            loop {
                a = rd.getc();
                if a != SPC && a != TAB {
                    break;
                }
            }
        } else {
            a = rd.getc();
        }
        let mut key: Vec<u8> = Vec::new();
        let mut n = 0;
        while key.len() < ARGUMENT_MAX && a != EOF {
            if a == sc(st.quote) || a == sc(st.escape) {
                key.push(a as u8);
                a = rd.getc();
                key.push(a as u8);
            } else if a == sc(st.arg_open) {
                key.push(a as u8);
                n += 1;
            } else if a == sc(st.arg_close) {
                if n == 0 {
                    if self.fl.compress_blanks && key.last() == Some(&b' ') {
                        key.pop();
                    }
                    self.key = cstr(&key).to_vec();
                    return true;
                } else {
                    key.push(a as u8);
                    n -= 1;
                }
            } else {
                match a {
                    LFD => {
                        self.idx_lc += 1;
                        self.idx_error(b"Incomplete first argument (premature LFD).\n");
                        return false;
                    }
                    TAB | SPC if self.fl.compress_blanks => {
                        if let Some(&l) = key.last() {
                            if l != b' ' && l != b'\t' {
                                key.push(b' ');
                            }
                        }
                    }
                    _ => key.push(a as u8),
                }
            }
            a = rd.getc();
        }
        Self::flush_to_eol(rd);
        self.idx_lc += 1;
        self.idx_error(format!("First argument too long (max {}).\n", ARGUMENT_MAX).as_bytes());
        false
    }

    fn scan_arg2(&mut self, rd: &mut Reader) -> bool {
        let mut a;
        loop {
            a = rd.getc();
            if a != SPC && a != TAB {
                break;
            }
        }
        let mut no: Vec<u8> = Vec::new();
        let mut hit_blank = false;
        while no.len() < NUMBER_MAX {
            if a == sc(self.st.arg_close) {
                self.no = cstr(&no).to_vec();
                return true;
            }
            match a {
                LFD => {
                    self.idx_lc += 1;
                    self.idx_error(b"Incomplete second argument (premature LFD).\n");
                    return false;
                }
                TAB | SPC => hit_blank = true,
                _ => {
                    if hit_blank {
                        Self::flush_to_eol(rd);
                        self.idx_lc += 1;
                        self.idx_error(b"Illegal space within numerals in second argument.\n");
                        return false;
                    }
                    no.push(a as u8);
                }
            }
            a = rd.getc();
        }
        Self::flush_to_eol(rd);
        self.idx_lc += 1;
        self.idx_error(format!("Second argument too long (max {}).\n", NUMBER_MAX).as_bytes());
        false
    }

    fn make_key(&mut self) -> bool {
        let mut e = Entry::new();
        if !self.scan_key(&mut e) {
            return false;
        }
        e.group = group_type(&e.sf[0]);
        e.lpg = self.no.clone();
        let no = self.no.clone();
        let mut count = 0usize;
        let mut typ = EMPTY;
        if !self.scan_no(&no, &mut e.npg, &mut count, &mut typ) {
            return false;
        }
        e.count = count;
        e.typ = typ;
        e.lc = self.idx_lc;
        e.file = self.cur_file;
        self.entries.push(e);
        true
    }

    fn scan_key(&mut self, e: &mut Entry) -> bool {
        let st = self.st;
        let mut i = 0usize;
        let mut n = 0usize;
        let mut second_round = false;
        let last = FIELD_MAX - 1;
        loop {
            let k = at(&self.key, n);
            if k == 0 {
                break;
            }
            if k == st.encap {
                n += 1;
                match self.scan_field(&mut n, false, false, false) {
                    Some(f) => {
                        e.encap = f;
                        break;
                    }
                    None => return false,
                }
            }
            if k == st.actual {
                n += 1;
                match self.scan_field(&mut n, i != last, true, false) {
                    Some(f) => e.af[i] = f,
                    None => return false,
                }
            } else {
                if second_round {
                    i += 1;
                    n += 1;
                }
                if i >= FIELD_MAX {
                    // Unreachable in C (level delimiters are rejected at the
                    // last level); guard against malformed state.
                    return false;
                }
                match self.scan_field(&mut n, i != last, true, true) {
                    Some(f) => e.sf[i] = f,
                    None => return false,
                }
                second_round = true;
                if self.fl.german_sort && e.sf[i].contains(&b'"') {
                    let (s, a) = search_quote(&e.sf[i]);
                    e.sf[i] = s;
                    e.af[i] = a;
                }
            }
        }
        if e.sf[0].is_empty() {
            self.idx_error(b"Illegal null field.\n");
            return false;
        }
        if e.sf[1].is_empty() && (!e.af[1].is_empty() || !e.sf[2].is_empty()) {
            self.idx_error(b"Illegal null field.\n");
            return false;
        }
        if e.sf[2].is_empty() && !e.af[2].is_empty() {
            self.idx_error(b"Illegal null field.\n");
            return false;
        }
        true
    }

    fn scan_field(&mut self, n: &mut usize, ck_level: bool, ck_encap: bool, ck_actual: bool) -> Option<Vec<u8>> {
        let st = self.st;
        let mut f: Vec<u8> = Vec::new();
        if self.fl.compress_blanks && matches!(at(&self.key, *n), b' ' | b'\t') {
            *n += 1;
        }
        loop {
            let mut nbsh = 0;
            while at(&self.key, *n) == st.escape && st.escape != 0 {
                nbsh += 1;
                f.push(at(&self.key, *n));
                *n += 1;
            }
            let k = at(&self.key, *n);
            if k == st.quote && k != 0 {
                if nbsh % 2 == 0 {
                    *n += 1;
                    f.push(at(&self.key, *n));
                } else {
                    f.push(k);
                }
            } else if (ck_level && k == st.level) || (ck_encap && k == st.encap) || (ck_actual && k == st.actual) || k == 0 {
                if !f.is_empty() && self.fl.compress_blanks && f.last() == Some(&b' ') {
                    f.pop();
                }
                return Some(cstr(&f).to_vec());
            } else {
                f.push(k);
                let pos = *n + 1;
                if !ck_level && k == st.level {
                    let m = cat(&[b"Extra `", &[st.level], format!("' at position {} of first argument.\n", pos).as_bytes()]);
                    self.idx_error(&m);
                    return None;
                } else if !ck_encap && k == st.encap {
                    let m = cat(&[b"Extra `", &[st.encap], format!("' at position {} of first argument.\n", pos).as_bytes()]);
                    self.idx_error(&m);
                    return None;
                } else if !ck_actual && k == st.actual {
                    let m = cat(&[b"Extra `", &[st.actual], format!("' at position {} of first argument.\n", pos).as_bytes()]);
                    self.idx_error(&m);
                    return None;
                }
            }
            if *n > self.key.len() + 1 {
                // Past the C string terminator: C would read stale buffer
                // contents; stop here.
                return Some(cstr(&f).to_vec());
            }
            *n += 1;
        }
    }

    fn has_prec(&self, ch: u8) -> bool {
        cstr(&self.st.page_precedence).contains(&ch)
    }

    fn is_comp(&self, no: &[u8], i: usize) -> bool {
        let comp = &self.st.page_compositor;
        for k in 0..self.comp_len {
            let a = at(no, i + k);
            let b = at(comp, k);
            if a != b {
                return false;
            }
            if a == 0 {
                return true;
            }
        }
        true
    }

    fn enter(&mut self, no: &[u8], npg: &mut [i32; PAGEFIELD_MAX], count: &mut usize, v: i32) -> bool {
        if *count >= PAGEFIELD_MAX {
            let m = cat(&[b"Page number ", cstr(no), format!(" has too many fields (max. {}).", PAGEFIELD_MAX).as_bytes()]);
            self.idx_error(&m);
            return false;
        }
        npg[*count] = v;
        *count += 1;
        true
    }

    fn sub<'a>(no: &'a [u8], i: usize) -> &'a [u8] {
        if i <= no.len() { &no[i..] } else { &[] }
    }

    fn scan_no(&mut self, no: &[u8], npg: &mut [i32; PAGEFIELD_MAX], count: &mut usize, typ: &mut i32) -> bool {
        let c0 = at(no, 0);
        let is_rl = |c: u8| b"ivxlcdm".contains(&c);
        let is_ru = |c: u8| b"IVXLCDM".contains(&c);
        let is_al = |c: u8| c.is_ascii_lowercase();
        let is_au = |c: u8| c.is_ascii_uppercase();
        let (hr, ha, hru, hau) = (self.has_prec(b'r'), self.has_prec(b'a'), self.has_prec(b'R'), self.has_prec(b'A'));
        let cnt = *count;
        let guess = self.type_guess.get(cnt).copied().unwrap_or(EMPTY);
        let mut set_guess = |g: i32, tg: &mut [i32; PAGEFIELD_MAX]| {
            if cnt < PAGEFIELD_MAX {
                tg[cnt] = g;
            }
        };
        let mut tg = self.type_guess;
        if isdigit(c0) {
            set_guess(ARAB as i32, &mut tg);
        } else if is_rl(c0) && is_al(c0) && hr && ha {
            if strspn(no, b"ivxlcdm") == 1 && guess != ROML as i32 && guess != ALPL as i32 {
                set_guess(if strspn(no, b"ivx") == 1 { ROML as i32 } else { ALPL as i32 }, &mut tg);
            }
            if strspn(no, b"ivxlcdm") > 1 {
                set_guess(ROML as i32, &mut tg);
            }
        } else if is_ru(c0) && is_au(c0) && hru && hau {
            if strspn(no, b"IVXLCDM") == 1 && guess != ROMU as i32 && guess != ALPU as i32 {
                set_guess(if strspn(no, b"IVX") == 1 { ROMU as i32 } else { ALPU as i32 }, &mut tg);
            }
            if strspn(no, b"IVXLCDM") > 1 {
                set_guess(ROMU as i32, &mut tg);
            }
        } else if is_rl(c0) && hr {
            set_guess(ROML as i32, &mut tg);
        } else if is_ru(c0) && hru {
            set_guess(ROMU as i32, &mut tg);
        } else if is_al(c0) && ha {
            set_guess(ALPL as i32, &mut tg);
        } else if is_au(c0) && hau {
            set_guess(ALPU as i32, &mut tg);
        } else {
            set_guess(EMPTY, &mut tg);
        }
        self.type_guess = tg;
        let guess = self.type_guess.get(cnt).copied().unwrap_or(EMPTY);

        if isdigit(c0) {
            *typ = ARAB as i32;
            self.scan_arabic(no, npg, count)
        } else if is_rl(c0) && hr && (!ha || guess == ROML as i32) {
            *typ = ROML as i32;
            self.scan_roman(no, npg, count, false)
        } else if is_ru(c0) && hru && (!hau || guess == ROMU as i32) {
            *typ = ROMU as i32;
            self.scan_roman(no, npg, count, true)
        } else if is_al(c0) && ha {
            *typ = ALPL as i32;
            self.scan_alpha(no, npg, count, ALPL)
        } else if is_au(c0) && hau {
            *typ = ALPU as i32;
            self.scan_alpha(no, npg, count, ALPU)
        } else {
            let m = cat(&[b"Illegal page number ", cstr(no), b" or page_precedence ", cstr(&self.st.page_precedence), b".\n"]);
            self.idx_error(&m);
            false
        }
    }

    fn scan_arabic(&mut self, no: &[u8], npg: &mut [i32; PAGEFIELD_MAX], count: &mut usize) -> bool {
        let mut i = 0usize;
        let mut s: Vec<u8> = Vec::new();
        while at(no, i) != 0 && i <= ARABIC_MAX && !self.is_comp(no, i) {
            if isdigit(at(no, i)) {
                s.push(at(no, i));
                i += 1;
            } else {
                let m = cat(&[format!("Illegal Arabic digit: position {} in ", i + 1).as_bytes(), cstr(no), b".\n"]);
                self.idx_error(&m);
                return false;
            }
        }
        if i > ARABIC_MAX {
            let m = cat(&[b"Arabic page number ", cstr(no), format!(" too big (max {} digits).\n", ARABIC_MAX).as_bytes()]);
            self.idx_error(&m);
            return false;
        }
        let v = strtoint(&s).wrapping_add(self.st.page_offset[ARAB]);
        if !self.enter(no, npg, count, v) {
            return false;
        }
        if self.is_comp(no, i) {
            let mut dummy = 0;
            let rest = Self::sub(no, i + self.comp_len).to_vec();
            self.scan_no(&rest, npg, count, &mut dummy)
        } else {
            true
        }
    }

    fn scan_roman(&mut self, no: &[u8], npg: &mut [i32; PAGEFIELD_MAX], count: &mut usize, upper: bool) -> bool {
        let val = |c: u8| -> i32 {
            match c.to_ascii_lowercase() {
                b'i' => 1,
                b'v' => 5,
                b'x' => 10,
                b'l' => 50,
                b'c' => 100,
                b'd' => 500,
                b'm' => 1000,
                _ => 0,
            }
        };
        let is_r = |c: u8| if upper { b"IVXLCDM".contains(&c) } else { b"ivxlcdm".contains(&c) };
        let mut i = 0usize;
        let mut inp = 0i32;
        let mut prev = 0i32;
        while at(no, i) != 0 && i < ROMAN_MAX && !self.is_comp(no, i) {
            let c = at(no, i);
            let mut the_new;
            if is_r(c) && {
                the_new = val(c);
                the_new != 0
            } {
                if prev == 0 {
                    prev = the_new;
                } else {
                    if prev < the_new {
                        prev = the_new - prev;
                        the_new = 0;
                    }
                    inp += prev;
                    prev = the_new;
                }
            } else {
                let m = cat(&[format!("Illegal Roman number: position {} in ", i + 1).as_bytes(), cstr(no), b".\n"]);
                self.idx_error(&m);
                return false;
            }
            i += 1;
        }
        if i == ROMAN_MAX {
            let m = cat(&[b"Roman page number ", cstr(no), format!(" too big (max {} digits).\n", ROMAN_MAX).as_bytes()]);
            self.idx_error(&m);
            return false;
        }
        inp += prev;
        let off = self.st.page_offset[if upper { ROMU } else { ROML }];
        if !self.enter(no, npg, count, inp + off) {
            return false;
        }
        if self.is_comp(no, i) {
            let mut dummy = 0;
            let rest = Self::sub(no, i + self.comp_len).to_vec();
            self.scan_no(&rest, npg, count, &mut dummy)
        } else {
            true
        }
    }

    fn scan_alpha(&mut self, no: &[u8], npg: &mut [i32; PAGEFIELD_MAX], count: &mut usize, t: usize) -> bool {
        let c = at(no, 0);
        let v = if c.is_ascii_uppercase() {
            (c - b'A') as i32
        } else if c.is_ascii_lowercase() {
            (c - b'a') as i32
        } else {
            0
        };
        if !self.enter(no, npg, count, v + self.st.page_offset[t]) {
            return false;
        }
        if self.is_comp(no, 1) {
            let mut dummy = 0;
            let rest = Self::sub(no, self.comp_len + 1).to_vec();
            self.scan_no(&rest, npg, count, &mut dummy)
        } else {
            true
        }
    }

    // ------------------------------------------------------------------
    // sortid.c + qsort.c
    // ------------------------------------------------------------------

    pub fn sort_idx(&mut self, order: &mut [usize]) {
        self.log.puts("Sorting entries...");
        self.log.idx_dc = 0;
        self.idx_gc = 0;
        self.qqsort(order);
        let m = format!("done ({} comparisons).\n", self.idx_gc);
        self.log.puts(&m);
    }

    fn compare(&mut self, a: usize, b: usize) -> i32 {
        self.idx_gc += 1;
        self.log.idx_dot(CMP_MAX);
        let mut dif = 0;
        let mut i = 0;
        while i < FIELD_MAX {
            dif = self.compare_one(&self.entries[a].sf[i], &self.entries[b].sf[i]);
            if dif != 0 {
                break;
            }
            dif = self.compare_one(&self.entries[a].af[i], &self.entries[b].af[i]);
            if dif != 0 {
                break;
            }
            i += 1;
        }
        if i == FIELD_MAX {
            dif = self.compare_page(a, b);
        }
        dif
    }

    fn compare_one(&self, x: &[u8], y: &[u8]) -> i32 {
        if x.is_empty() && y.is_empty() {
            return 0;
        }
        if x.is_empty() {
            return -1;
        }
        if y.is_empty() {
            return 1;
        }
        let m = group_type(x);
        let n = group_type(y);
        if m >= 0 && n >= 0 {
            return m.wrapping_sub(n);
        }
        if m >= 0 {
            return if self.fl.german_sort { 1 } else if n == -1 { 1 } else { -1 };
        }
        if n >= 0 {
            return if self.fl.german_sort { -1 } else if m == -1 { -1 } else { 1 };
        }
        if m == SYMBOL && n == SYMBOL {
            let dm = isdigit(x[0]);
            let dn = isdigit(y[0]);
            if dm && !dn {
                return 1;
            }
            if !dm && dn {
                return -1;
            }
            return strcmp(x, y);
        }
        if m == SYMBOL {
            return -1;
        }
        if n == SYMBOL {
            return 1;
        }
        self.compare_string(x, y)
    }

    fn compare_string(&self, a: &[u8], b: &[u8]) -> i32 {
        let (mut i, mut j) = (0usize, 0usize);
        while at(a, i) != 0 || at(b, j) != 0 {
            if at(a, i) == 0 {
                return -1;
            }
            if at(b, j) == 0 {
                return 1;
            }
            if self.fl.letter_ordering {
                if at(a, i) == b' ' {
                    i += 1;
                }
                if at(b, j) == b' ' {
                    j += 1;
                }
            }
            let al = CtypeLocale::C.tolower(at(a, i)) as i32;
            let bl = CtypeLocale::C.tolower(at(b, j)) as i32;
            if al != bl {
                return al - bl;
            }
            i += 1;
            j += 1;
        }
        if self.fl.german_sort {
            // new_strcmp(a, b, GERMAN)
            let mut k = 0;
            while at(a, k) == at(b, k) {
                if at(a, k) == 0 {
                    return 0;
                }
                k += 1;
            }
            if at(a, k).is_ascii_uppercase() { 1 } else { -1 }
        } else {
            strcmp(a, b)
        }
    }

    fn compare_page(&mut self, a: usize, b: usize) -> i32 {
        let (ea, eb) = (&self.entries[a], &self.entries[b]);
        let mut m = 0;
        let mut i = 0;
        while i < ea.count && i < eb.count && {
            m = ea.npg[i].wrapping_sub(eb.npg[i]);
            m == 0
        } {
            i += 1;
        }
        if m == 0 {
            if i == ea.count && i == eb.count {
                let ro = self.st.range_open;
                let rc = self.st.range_close;
                let isrange = |c: u8| c == ro || c == rc;
                let ca = at(&ea.encap, 0);
                let cb = at(&eb.encap, 0);
                if isrange(ca) && isrange(cb) {
                    m = ea.lc - eb.lc;
                } else if ea.encap == eb.encap {
                    if ea.typ != DUPLICATE && eb.typ != DUPLICATE {
                        self.entries[b].typ = DUPLICATE;
                    }
                } else if isrange(ca) || isrange(cb) {
                    m = ea.lc - eb.lc;
                } else {
                    m = self.compare_string(&ea.encap, &eb.encap);
                }
            } else if i == ea.count && i < eb.count {
                m = -1;
            } else if i < ea.count && i == eb.count {
                m = 1;
            }
        }
        m
    }

    fn qqsort(&mut self, v: &mut [usize]) {
        let n = v.len();
        if n <= 1 {
            return;
        }
        let mut hi = if n >= THRESH {
            self.qst(v, 0, n);
            THRESH
        } else {
            n
        };
        let mut j = 0usize;
        let mut lo = 0usize;
        loop {
            lo += 1;
            if lo >= hi {
                break;
            }
            if self.compare(v[j], v[lo]) > 0 {
                j = lo;
            }
        }
        if j != 0 {
            v.swap(0, j);
        }
        let mut min = 0usize;
        loop {
            min += 1;
            hi = min;
            if hi >= n {
                break;
            }
            loop {
                if hi == 0 {
                    // C would step before the array; the sentinel prevents
                    // this for consistent comparators.
                    break;
                }
                hi -= 1;
                if self.compare(v[hi], v[min]) <= 0 {
                    hi += 1;
                    break;
                }
            }
            if hi != min {
                let x = v[min];
                v.copy_within(hi..min, hi + 1);
                v[hi] = x;
            }
        }
    }

    fn qst(&mut self, v: &mut [usize], mut base: usize, mut max: usize) {
        let mut lo = max - base;
        loop {
            let mut i = base + (lo >> 1);
            let mut mid = i;
            if lo >= MTHRESH {
                let jj = base;
                let mut j = if self.compare(v[jj], v[i]) > 0 { jj } else { i };
                let tmp = max - 1;
                if self.compare(v[j], v[tmp]) > 0 {
                    j = if j == jj { i } else { jj };
                    if self.compare(v[j], v[tmp]) < 0 {
                        j = tmp;
                    }
                }
                if j != i {
                    v.swap(i, j);
                }
            }
            i = base;
            let mut j = max - 1;
            loop {
                while i < mid && self.compare(v[i], v[mid]) <= 0 {
                    i += 1;
                }
                let jj;
                let tmp;
                let mut swapped = false;
                let mut sw = (0usize, 0usize, 0usize);
                while j > mid {
                    if self.compare(v[mid], v[j]) <= 0 {
                        j -= 1;
                        continue;
                    }
                    let t = i + 1;
                    let jjv;
                    if i == mid {
                        mid = j;
                        jjv = j;
                    } else {
                        jjv = j;
                        j -= 1;
                    }
                    sw = (i, jjv, t);
                    swapped = true;
                    break;
                }
                if swapped {
                    jj = sw.1;
                    tmp = sw.2;
                } else if i == mid {
                    break;
                } else {
                    jj = mid;
                    mid = i;
                    tmp = i;
                    j -= 1;
                }
                v.swap(i, jj);
                i = tmp;
            }
            j = mid;
            i = mid + 1;
            let lo2 = j - base;
            let hi2 = max - i;
            if lo2 <= hi2 {
                if lo2 >= THRESH {
                    self.qst(v, base, j);
                }
                base = i;
                lo = hi2;
            } else {
                if hi2 >= THRESH {
                    self.qst(v, i, max);
                }
                max = j;
                lo = lo2;
            }
            if lo < THRESH {
                break;
            }
        }
    }
}

fn strcmp(a: &[u8], b: &[u8]) -> i32 {
    let (a, b) = (cstr(a), cstr(b));
    let mut k = 0;
    loop {
        let (x, y) = (at(a, k), at(b, k));
        if x != y {
            return x as i32 - y as i32;
        }
        if x == 0 {
            return 0;
        }
        k += 1;
    }
}

/// `search_quote` (`scanid.c` lines 756-799): German umlaut sort keys.
fn search_quote(sort_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let actual = sort_key.to_vec();
    let mut sk = sort_key.to_vec();
    let mut found = false;
    let mut p = sk.iter().position(|&c| c == b'"');
    while let Some(pos) = p {
        let next = at(&sk, pos + 1);
        let rep: Option<&[u8; 2]> = match next {
            b'a' => Some(b"ae"),
            b'A' => Some(b"Ae"),
            b'o' => Some(b"oe"),
            b'O' => Some(b"Oe"),
            b'u' => Some(b"ue"),
            b'U' => Some(b"Ue"),
            b's' => Some(b"ss"),
            _ => None,
        };
        if let Some(r) = rep {
            found = true;
            sk[pos] = r[0];
            sk[pos + 1] = r[1];
        }
        p = sk.iter().skip(pos + 1).position(|&c| c == b'"').map(|q| q + pos + 1);
    }
    if found { (sk, actual) } else { (sk, Vec::new()) }
}

// ----------------------------------------------------------------------
// genind.c
// ----------------------------------------------------------------------

pub(crate) struct PageStart {
    pub pageno: Vec<u8>,
    pub even_odd: i32,
}

pub(crate) struct Gen<'e, 's> {
    e: &'e mut Engine<'s>,
    order: Vec<usize>,
    ind: Vec<u8>,
    ind_name: Vec<u8>,
    curr: Option<usize>,
    prev: Option<usize>,
    begin: usize,
    the_end: usize,
    range_ptr: usize,
    level: usize,
    prev_level: usize,
    encap: Vec<u8>,
    prev_encap: Option<Vec<u8>>,
    in_range: bool,
    encap_range: bool,
    buff: Vec<u8>,
    line: Vec<u8>,
    ind_lc: i32,
    ind_ec: i32,
    ind_indent: i32,
}

impl<'e, 's> Gen<'e, 's> {
    pub fn new(e: &'e mut Engine<'s>, order: Vec<usize>, ind_name: &[u8]) -> Self {
        Gen {
            e,
            order,
            ind: Vec::new(),
            ind_name: ind_name.to_vec(),
            curr: None,
            prev: None,
            begin: 0,
            the_end: 0,
            range_ptr: 0,
            level: 0,
            prev_level: 0,
            encap: Vec::new(),
            prev_encap: None,
            in_range: false,
            encap_range: false,
            buff: Vec::new(),
            line: Vec::new(),
            ind_lc: 0,
            ind_ec: 0,
            ind_indent: 0,
        }
    }

    fn en(&self, i: usize) -> &Entry {
        &self.e.entries[i]
    }

    fn put(&mut self, s: &[u8]) {
        self.ind.extend_from_slice(cstr(s));
    }

    fn putln(&mut self, s: &[u8]) {
        self.ind.extend_from_slice(cstr(s));
        self.ind.push(b'\n');
        self.ind_lc += 1;
    }

    fn ind_error(&mut self, msg: &[u8]) {
        let c = self.curr.expect("curr set before warnings");
        self.e.log.break_dots();
        let head = cat(&[
            b"## Warning (input = ",
            &self.e.names[self.e.entries[c].file],
            format!(", line = {}; output = ", self.e.entries[c].lc).as_bytes(),
            &self.ind_name,
            format!(", line = {}):\n   -- ", self.ind_lc + 1).as_bytes(),
        ]);
        self.e.log.put(&head);
        self.e.log.put(msg);
        self.ind_ec += 1;
    }

    pub fn gen_ind(mut self, page: Option<PageStart>) -> (Vec<u8>, i32, i32) {
        let st = self.e.st;
        self.e.log.put(&cat(&[b"Generating output file ", &self.ind_name, b"..."]));
        self.put(&st.preamble);
        self.ind_lc += st.prelen;
        if let Some(p) = page {
            self.insert_page(p);
        }
        self.e.log.idx_dc = 0;
        for n in 0..self.order.len() {
            let id = self.order[n];
            if self.e.entries[id].typ != DUPLICATE {
                self.make_entry(n);
                self.e.log.idx_dot(DOT_MAX);
            }
        }
        if self.in_range {
            self.curr = Some(self.range_ptr);
            let m = cat(&[b"Unmatched range opening operator ", &[st.range_open], b".\n"]);
            self.ind_error(&m);
        }
        self.prev = self.curr;
        if self.curr.is_some() {
            self.flush_line(true);
        }
        self.put(&st.delim_t);
        self.put(&st.postamble);
        let tmp_lc = self.ind_lc + st.postlen;
        let m = if self.ind_ec == 1 {
            format!("done ({} lines written, {} warning).\n", tmp_lc, self.ind_ec)
        } else {
            format!("done ({} lines written, {} warnings).\n", tmp_lc, self.ind_ec)
        };
        self.e.log.puts(&m);
        (self.ind, tmp_lc, self.ind_ec)
    }

    fn make_entry(&mut self, n: usize) {
        let st = self.e.st;
        self.prev = self.curr;
        let c = self.order[n];
        self.curr = Some(c);
        let enc0 = at(&self.en(c).encap, 0);
        self.encap = if enc0 == st.range_open || enc0 == st.range_close {
            self.en(c).encap[1..].to_vec()
        } else {
            self.en(c).encap.clone()
        };
        if self.prev.is_none() {
            // n == 0 in C (see CONTRACT.md for the duplicate-at-zero note).
            self.prev_level = 0;
            self.level = 0;
            let let_ = at(&self.en(c).sf[0], 0);
            self.put_header(let_, CtypeLocale::C);
            self.make_item(b"");
        } else {
            let p = self.prev.unwrap();
            self.prev_level = self.level;
            self.level = 0;
            while self.level < FIELD_MAX {
                let l = self.level;
                if self.en(c).sf[l] != self.en(p).sf[l] || self.en(c).af[l] != self.en(p).af[l] {
                    break;
                }
                self.level += 1;
            }
            if self.level < FIELD_MAX {
                self.new_entry();
            } else if !(enc0 == st.range_open && self.in_range) {
                self.old_entry();
            }
        }

        if enc0 == st.range_open {
            if self.in_range {
                let m = cat(&[b"Extra range opening operator ", &[st.range_open], b".\n"]);
                self.ind_error(&m);
            } else {
                self.in_range = true;
                self.range_ptr = c;
            }
        } else if enc0 == st.range_close {
            if self.in_range {
                self.in_range = false;
                let rest = self.en(c).encap[1..].to_vec();
                if !rest.is_empty() && self.prev_encap.as_deref().map(cstr) != Some(&rest[..]) {
                    let m = cat(&[b"Range closing operator has an inconsistent encapsulator ", &rest, b".\n"]);
                    self.ind_error(&m);
                }
            } else {
                let m = cat(&[b"Unmatched range closing operator ", &[st.range_close], b".\n"]);
                self.ind_error(&m);
            }
        } else if enc0 != 0 && self.prev_encap.as_deref() != Some(&self.en(c).encap[..]) && self.in_range {
            let m = cat(&[b"Inconsistent page encapsulator ", &self.en(c).encap.clone(), b" within range.\n"]);
            self.ind_error(&m);
        }
    }

    fn make_item(&mut self, term: &[u8]) {
        let st = self.e.st;
        let c = self.curr.unwrap();
        let l = self.level;
        let field = if self.en(c).af[l].is_empty() { self.en(c).sf[l].clone() } else { self.en(c).af[l].clone() };
        if self.level > self.prev_level {
            self.line = cat(&[term, cstr(&st.item_u[l]), &field]);
            self.ind_lc += st.ilen_u[l];
        } else {
            self.line = cat(&[term, cstr(&st.item_r[l]), &field]);
            self.ind_lc += st.ilen_r[l];
        }
        let mut i = l + 1;
        while i < FIELD_MAX && !self.en(c).sf[i].is_empty() {
            let line = std::mem::take(&mut self.line);
            self.put(&line);
            let field = if self.en(c).af[i].is_empty() { self.en(c).sf[i].clone() } else { self.en(c).af[i].clone() };
            self.line = cat(&[cstr(&st.item_x[i]), &field]);
            self.ind_lc += st.ilen_x[i];
            self.level = i;
            i += 1;
        }
        self.ind_indent = 0;
        let d = cstr(&st.delim_p[self.level]).to_vec();
        self.line.extend_from_slice(&d);
        self.save();
    }

    fn save(&mut self) {
        let c = self.curr.unwrap();
        self.begin = c;
        self.the_end = c;
        self.prev_encap = Some(self.encap.clone());
    }

    fn new_entry(&mut self) {
        let st = self.e.st;
        let ct = self.e.fl.ctype;
        if self.in_range {
            let ptr = self.curr;
            self.curr = Some(self.range_ptr);
            let m = cat(&[b"Unmatched range opening operator ", &[st.range_open], b".\n"]);
            self.ind_error(&m);
            self.in_range = false;
            self.curr = ptr;
        }
        self.flush_line(true);
        let c = self.curr.unwrap();
        let p = self.prev.unwrap();
        let (cg, pg) = (self.en(c).group, self.en(p).group);
        let first = |s: &[u8]| ct.tolower(at(s, 0));
        let mut let_: i32 = -1;
        let new_group = (cg != ALPHA && cg != pg && pg == SYMBOL)
            || (cg == ALPHA && {
                let l = first(&self.en(c).sf[0]);
                let_ = l as i32;
                l != first(&self.en(p).sf[0])
            })
            || (self.e.fl.german_sort && cg != ALPHA && pg == ALPHA);
        if new_group {
            self.put(&st.delim_t);
            self.put(&st.group_skip);
            self.ind_lc += st.skiplen;
            self.put_header(let_ as u8, ct);
            self.make_item(b"");
        } else {
            let t = cstr(&st.delim_t).to_vec();
            self.make_item(&t);
        }
    }

    fn old_entry(&mut self) {
        let st = self.e.st;
        let c = self.curr.unwrap();
        let p = self.prev.unwrap();
        let diff = self.page_diff(self.the_end, c);
        let same_type = self.en(p).typ == self.en(c).typ;
        let pe_eq = self.prev_encap.as_deref() == Some(&self.encap[..]);
        if same_type
            && diff != -1
            && ((diff == 0 && pe_eq) || (self.e.fl.merge_page && diff == 1 && pe_eq) || self.in_range)
        {
            self.the_end = c;
            let ce = self.en(c).encap.clone();
            if self.in_range && at(&ce, 0) != 0 && at(&ce, 0) != st.range_close && self.prev_encap.as_deref() != Some(&ce[..]) {
                self.buff = cat(&[cstr(&st.encap_prefix), &ce, cstr(&st.encap_infix), &self.en(c).lpg.clone(), cstr(&st.encap_suffix)]);
                self.wrap_line(false);
            }
            if self.in_range {
                self.encap_range = true;
            }
        } else {
            self.flush_line(false);
            if diff == 0 && same_type {
                self.ind_error(b"Conflicting entries: multiple encaps for the same page under same key.\n");
            } else if self.in_range && !same_type {
                self.ind_error(b"Illegal range formation: starting & ending pages are of different types.\n");
            } else if self.in_range && diff == -1 {
                self.ind_error(b"Illegal range formation: starting & ending pages cross chap/sec breaks.\n");
            }
            self.save();
        }
    }

    fn page_diff(&self, a: usize, b: usize) -> i32 {
        let (a, b) = (self.en(a), self.en(b));
        if a.count != b.count {
            return -1;
        }
        if a.count == 0 {
            return 0;
        }
        for i in 0..a.count - 1 {
            if a.npg[i] != b.npg[i] {
                return -1;
            }
        }
        b.npg[b.count - 1].wrapping_sub(a.npg[a.count - 1])
    }

    fn put_header(&mut self, let_: u8, ct: CtypeLocale) {
        let st = self.e.st;
        if st.headings_flag != 0 {
            self.put(&st.heading_prefix);
            self.ind_lc += st.headprelen;
            let c = self.curr.unwrap();
            match self.en(c).group {
                SYMBOL => {
                    if st.headings_flag > 0 {
                        self.put(&st.symhead_positive);
                    } else {
                        self.put(&st.symhead_negative);
                    }
                }
                ALPHA => {
                    let l = if st.headings_flag > 0 { ct.toupper(let_) } else { ct.tolower(let_) };
                    self.ind.push(l);
                }
                _ => {
                    if st.headings_flag > 0 {
                        self.put(&st.numhead_positive);
                    } else {
                        self.put(&st.numhead_negative);
                    }
                }
            }
            self.put(&st.heading_suffix);
            self.ind_lc += st.headsuflen;
        }
    }

    fn flush_line(&mut self, print: bool) {
        let st = self.e.st;
        let prev = self.prev.unwrap_or(self.begin);
        let begin_lpg = self.en(self.begin).lpg.clone();
        let end_lpg = self.en(self.the_end).lpg.clone();
        if self.page_diff(self.begin, self.the_end) != 0 {
            let thresh = if !st.suffix_2p.is_empty() { 0 } else { 1 };
            if self.encap_range || self.page_diff(self.begin, prev) > thresh {
                let diff = self.page_diff(self.begin, self.the_end);
                self.buff = if diff == 1 && !st.suffix_2p.is_empty() {
                    cat(&[&begin_lpg, cstr(&st.suffix_2p)])
                } else if diff == 2 && !st.suffix_3p.is_empty() {
                    cat(&[&begin_lpg, cstr(&st.suffix_3p)])
                } else if diff >= 2 && !st.suffix_mp.is_empty() {
                    cat(&[&begin_lpg, cstr(&st.suffix_mp)])
                } else {
                    cat(&[&begin_lpg, cstr(&st.delim_r), &end_lpg])
                };
                self.encap_range = false;
            } else {
                self.buff = cat(&[&begin_lpg, cstr(&st.delim_n), &end_lpg]);
            }
        } else {
            self.encap_range = false;
            self.buff = begin_lpg;
        }
        let pe = self.prev_encap.clone().unwrap_or_default();
        if !pe.is_empty() {
            self.buff = cat(&[cstr(&st.encap_prefix), &pe, cstr(&st.encap_infix), &self.buff, cstr(&st.encap_suffix)]);
        }
        self.wrap_line(print);
    }

    fn wrap_line(&mut self, print: bool) {
        let st = self.e.st;
        let len = self.line.len() as i32 + self.buff.len() as i32 + self.ind_indent;
        if print {
            if len > st.line_max {
                let line = std::mem::take(&mut self.line);
                self.putln(&line);
                self.line = line;
                self.put(&st.indent_space);
                self.ind_indent = st.indent_length;
            } else {
                let line = self.line.clone();
                self.put(&line);
            }
            let buff = self.buff.clone();
            self.put(&buff);
        } else if len > st.line_max {
            let line = std::mem::take(&mut self.line);
            self.putln(&line);
            self.line = cat(&[cstr(&st.indent_space), &self.buff, cstr(&st.delim_n)]);
            self.ind_indent = st.indent_length;
        } else {
            let d = cstr(&st.delim_n).to_vec();
            self.buff.extend_from_slice(&d);
            let b = self.buff.clone();
            self.line.extend_from_slice(&b);
        }
    }

    fn insert_page(&mut self, p: PageStart) {
        let st = self.e.st;
        let mut pageno = cstr(&p.pageno).to_vec();
        if p.even_odd >= 0 && !pageno.is_empty() {
            let len = pageno.len();
            let j0 = len;
            let mut i = len;
            loop {
                i -= 1;
                if !(isdigit(pageno[i]) && i > 0) {
                    break;
                }
            }
            if !isdigit(pageno[i]) {
                i += 1;
            }
            let mut page = strtoint(&pageno[i..]).wrapping_add(1);
            if (p.even_odd == 1 && page % 2 == 0) || (p.even_odd == 2 && page % 2 != 0) {
                page += 1;
            }
            let mut buf = pageno.clone();
            buf.resize(len + 2, 0);
            buf[j0 + 1] = 0;
            let mut j = j0;
            while page >= 10 {
                buf[j] = b'0' + (page % 10) as u8;
                j -= 1;
                page /= 10;
            }
            buf[j] = b'0' + page as u8;
            if i < j {
                while buf[j] != 0 {
                    buf[i] = buf[j];
                    i += 1;
                    j += 1;
                }
                buf[i] = 0;
            }
            pageno = cstr(&buf).to_vec();
        }
        self.put(&st.setpage_prefix);
        self.put(&pageno);
        self.put(&st.setpage_suffix);
        self.ind_lc += st.setpagelen;
    }
}

/// `find_pageno` (`mkind.c` lines 467-492): the last `[<digits>` in a log.
pub(crate) fn find_pageno(log: &[u8]) -> Option<Vec<u8>> {
    if log.len() < 2 {
        return None;
    }
    let mut k = log.len() - 1;
    loop {
        let p = log[k - 1];
        let c = log[k];
        if p == b'[' && isdigit(c) {
            let mut s = Vec::new();
            let mut q = k;
            while q < log.len() && isdigit(log[q]) {
                s.push(log[q]);
                q += 1;
            }
            return Some(s);
        }
        k -= 1;
        if k == 0 {
            break;
        }
    }
    None
}
