//! Core state of the BibTeX port: logging, line input, token scanning, the
//! (value-based) string tables, `.aux` reading and the final clean-up.
//!
//! Section numbers (§N) refer to `bibtex.web` 0.99e; `[N]` refers to the TeX
//! Live change file `bibtex.ch`.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::chars::{lex, lowered, white, ALPHA, NUMERIC};
use crate::{FileKind, FileSource, Options};

pub(crate) type Str = Rc<[u8]>;

/// The non-local exits of `bibtex.web`.
#[derive(Debug)]
pub(crate) enum Flow {
    /// `goto bst_done` (§149: end of the style file while recovering).
    BstDone,
    /// `goto close_up_shop` (fatal error, §§44–45).
    Fatal,
}

pub(crate) type R<T> = Result<T, Flow>;

/// Something that can be written to the log (`print` in `bibtex.web`).
pub(crate) trait Pr {
    fn pr(&self, out: &mut Vec<u8>);
}

impl Pr for &str {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_bytes());
    }
}
impl Pr for &[u8] {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}
impl Pr for Str {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}
impl Pr for &Str {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}
impl Pr for &Vec<u8> {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}
impl Pr for i32 {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.to_string().as_bytes());
    }
}
impl Pr for i64 {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.to_string().as_bytes());
    }
}
impl Pr for usize {
    fn pr(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.to_string().as_bytes());
    }
}
/// A single character (`xchr[c]`, the identity in TeX Live).
pub(crate) struct Ch(pub u8);
impl Pr for Ch {
    fn pr(&self, out: &mut Vec<u8>) {
        out.push(self.0);
    }
}

macro_rules! p {
    ($e:expr $(, $a:expr)* $(,)?) => {{
        $( crate::engine::Pr::pr(&$a, &mut $e.log); )*
    }};
}
macro_rules! pln {
    ($e:expr $(, $a:expr)* $(,)?) => {{
        p!($e $(, $a)*);
        $e.log.push(b'\n');
    }};
}
pub(crate) use {p, pln};

/// An input file with web2c `eof`/`eoln` semantics (`lib/eofeoln.c`):
/// a line ends at `\n` or `\r`, and exactly one end-of-line byte is skipped.
pub(crate) struct InFile {
    data: Rc<[u8]>,
    pos: usize,
}

impl InFile {
    pub(crate) fn new(data: Rc<[u8]>) -> Self {
        InFile { data, pos: 0 }
    }
    pub(crate) fn eof(&self) -> bool {
        self.pos >= self.data.len()
    }
}

/// `input_ln` (§47 with [47]).
pub(crate) fn input_ln(f: &mut InFile, buf: &mut Vec<u8>, last: &mut usize) -> bool {
    *last = 0;
    if f.eof() {
        return false;
    }
    while f.pos < f.data.len() && f.data[f.pos] != b'\n' && f.data[f.pos] != b'\r' {
        if *last + 2 >= buf.len() {
            let n = buf.len() + 20000;
            buf.resize(n, 0);
        }
        buf[*last] = f.data[f.pos];
        f.pos += 1;
        *last += 1;
    }
    if f.pos < f.data.len() {
        f.pos += 1;
    }
    while *last > 0 && white(buf[*last - 1]) {
        *last -= 1;
    }
    true
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScanResult {
    IdNull,
    SpecifiedCharAdjacent,
    OtherCharAdjacent,
    WhiteAdjacent,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TypeRef {
    Empty,
    Undefined,
    Fn(usize),
}

#[derive(Clone, Debug)]
pub(crate) enum FnClass {
    BuiltIn(usize),
    Wiz(usize),
    IntLit(i32),
    StrLit(Str),
    Field(usize),
    IntEntry(usize),
    StrEntry(usize),
    IntGlobal(i32),
    StrGlobal(usize),
}

pub(crate) struct Func {
    pub name: Str,
    pub class: FnClass,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum WizItem {
    Call(usize),
    Quote,
    End,
}

#[derive(Clone, Debug)]
pub(crate) enum Glob {
    Static(Str),
    Buf(Vec<u8>),
}

/// A literal on the execution stack (§291). The `bool` of `Str` is `true`
/// for strings created during the current command (`>= cmd_str_ptr`).
#[derive(Clone, Debug)]
pub(crate) enum Lit {
    Int(i32),
    Str(Str, bool),
    Fn(usize),
    Missing(Str),
    Empty,
}

pub(crate) const SPOTLESS: u8 = 0;
pub(crate) const WARNING_MESSAGE: u8 = 1;
pub(crate) const ERROR_MESSAGE: u8 = 2;
pub(crate) const FATAL_MESSAGE: u8 = 3;

pub(crate) const NUM_BLT_IN_FNS: usize = 37;
pub(crate) const BUILTIN_NAMES: [&str; NUM_BLT_IN_FNS] = [
    "=", ">", "<", "+", "-", "*", ":=", "add.period$", "call.type$", "change.case$",
    "chr.to.int$", "cite$", "duplicate$", "empty$", "format.name$", "if$", "int.to.chr$",
    "int.to.str$", "missing$", "newline$", "num.names$", "pop$", "preamble$", "purify$",
    "quote$", "skip$", "stack$", "substring$", "swap$", "text.length$", "text.prefix$",
    "top$", "type$", "warning$", "while$", "width$", "write$",
];

pub(crate) struct Engine<'a> {
    pub fs: &'a dyn FileSource,
    pub opts: Options,
    pub log: Vec<u8>,
    pub bbl: Vec<u8>,
    pub history: u8,
    pub err_count: i32,

    // String pool bookkeeping, for the usage statistics (§465).
    pub pool: HashSet<Vec<u8>>,
    pub pool_chars: usize,

    // Line buffers (§§41–43, §290).
    pub buffer: Vec<u8>,
    pub last: usize,
    pub buf_ptr1: usize,
    pub buf_ptr2: usize,
    pub sv_buffer: Vec<u8>,
    pub ex_buf: Vec<u8>,
    pub ex_buf_ptr: i64,
    pub ex_buf_xptr: i64,
    pub ex_buf_yptr: i64,
    pub ex_buf_length: i64,
    pub out_buf: Vec<u8>,
    pub out_buf_length: usize,
    pub scan_result: ScanResult,
    pub token_value: i32,

    // .aux state (§§104, 117, 124, 129).
    pub aux_files: Vec<Option<InFile>>,
    pub aux_list: Vec<Str>,
    pub aux_ln_stack: Vec<i32>,
    pub aux_ptr: usize,
    pub aux_seen: HashSet<Vec<u8>>,
    pub top_lev_str: Str,
    pub bib_list: Vec<Str>,
    pub bib_files: Vec<Option<InFile>>,
    pub bib_seen_names: HashSet<Vec<u8>>,
    pub bib_ptr: usize,
    pub num_bib_files: usize,
    pub bib_seen: bool,
    pub bst_seen: bool,
    pub bst_str: Option<Str>,
    pub bst_file: Option<InFile>,
    pub citation_seen: bool,
    pub all_entries: bool,
    pub all_marker: usize,
    pub cite_list: Vec<Str>,
    pub cite_idx: HashMap<Vec<u8>, usize>,
    pub lc_cite: HashMap<Vec<u8>, Vec<u8>>,
    pub cite_ptr: usize,
    pub num_cites: usize,
    pub old_num_cites: usize,
    /// TeX Live's `max_cites` ([138]): initial `MAX_CITES=750`, grows by 750.
    pub max_cites: usize,
    /// TeX Live's `max_fields` ([226]): initial `MAX_FIELDS=5000`; when
    /// `total_fields > max_fields` it jumps directly to `total_fields + 5000`
    /// (not incremented in a loop).
    pub max_fields_cap: usize,
    /// TeX Live's `max_bib_files` ([242]/[123]): initial `MAX_BIB_FILES=20`,
    /// grows by 20.
    pub max_bib_files_cap: usize,
    /// TeX Live's `max_glob_strs` ([216]): initial `MAX_GLOB_STRS=10`, grows
    /// by 10.
    pub max_glob_strs_cap: usize,
    /// TeX Live's `lit_stk_size` ([307]): initial `LIT_STK_SIZE=50`, grows
    /// by 50.
    pub lit_stk_size_cap: usize,
    pub entry_cite_ptr: usize,
    pub cite_xptr: usize,

    // .bst state.
    pub bst_line_num: i32,
    pub bbl_line_num: i32,
    pub fns: Vec<Func>,
    pub fn_map: HashMap<Vec<u8>, usize>,
    pub int_lits: HashMap<Vec<u8>, usize>,
    pub str_lits: HashMap<Vec<u8>, usize>,
    pub macros: HashMap<Vec<u8>, Str>,
    pub wiz_functions: Vec<WizItem>,
    /// TeX Live's `wiz_fn_space` ([200]): initial `WIZ_FN_SPACE=3000`, grows
    /// by 3000 whenever `single_ptr + wiz_def_ptr > wiz_fn_space`.
    pub wiz_fn_space: usize,
    pub wiz_loc: usize,
    pub impl_fn_num: i32,
    pub b_default: usize,
    pub b_skip: usize,
    pub num_fields: usize,
    pub num_pre_defined_fields: usize,
    pub crossref_num: usize,
    pub num_ent_ints: usize,
    pub num_ent_strs: usize,
    pub sort_key_num: usize,
    pub globs: Vec<Glob>,
    pub entry_seen: bool,
    pub read_seen: bool,
    pub read_performed: bool,
    pub reading_completed: bool,
    pub read_completed: bool,

    // .bib state (§219).
    pub bib_line_num: i32,
    pub type_list: Vec<TypeRef>,
    pub entry_exists: Vec<bool>,
    pub cite_info_str: Vec<Option<Str>>,
    pub cite_info_cnt: Vec<i32>,
    pub sorted_cites: Vec<usize>,
    pub field_info: Vec<Option<Str>>,
    pub entry_type: Option<usize>,
    pub type_exists: bool,
    pub store_entry: bool,
    pub store_field: bool,
    pub field_name_fn: Option<usize>,
    pub right_outer_delim: u8,
    pub right_str_delim: u8,
    pub at_bib_command: bool,
    pub command_num: u8,
    pub cur_macro: Vec<u8>,
    pub bib_brace_level: i32,
    pub s_preamble: Vec<Str>,
    pub preamble_ptr: usize,
    pub num_preamble_strings: usize,

    // Execution state (§290, §344).
    pub stack: Vec<Lit>,
    pub entry_ints: Vec<i32>,
    pub entry_strs: Vec<Vec<u8>>,
    pub mess_with_entries: bool,
    pub brace_level: i32,
    pub prev_colon: bool,
    pub exec_count: [i64; NUM_BLT_IN_FNS],
    pub builtin_locs: [usize; NUM_BLT_IN_FNS],
    pub name_tok: Vec<i64>,
    pub name_sep_char: Vec<u8>,
}

impl<'a> Engine<'a> {
    pub(crate) fn new(fs: &'a dyn FileSource, opts: Options) -> Self {
        let empty: Str = Rc::from(&b""[..]);
        Engine {
            fs,
            opts,
            log: Vec::new(),
            bbl: Vec::new(),
            history: SPOTLESS,
            err_count: 0,
            pool: HashSet::new(),
            pool_chars: 0,
            buffer: vec![0; 20001],
            last: 0,
            buf_ptr1: 0,
            buf_ptr2: 0,
            sv_buffer: vec![0; 20001],
            ex_buf: vec![0; 20001],
            ex_buf_ptr: 0,
            ex_buf_xptr: 0,
            ex_buf_yptr: 0,
            ex_buf_length: 0,
            out_buf: vec![0; 20001],
            out_buf_length: 0,
            scan_result: ScanResult::IdNull,
            token_value: 0,
            aux_files: Vec::new(),
            aux_list: Vec::new(),
            aux_ln_stack: Vec::new(),
            aux_ptr: 0,
            aux_seen: HashSet::new(),
            top_lev_str: empty.clone(),
            bib_list: Vec::new(),
            bib_files: Vec::new(),
            bib_seen_names: HashSet::new(),
            bib_ptr: 0,
            num_bib_files: 0,
            bib_seen: false,
            bst_seen: false,
            bst_str: None,
            bst_file: None,
            citation_seen: false,
            all_entries: false,
            all_marker: 0,
            cite_list: Vec::new(),
            cite_idx: HashMap::new(),
            lc_cite: HashMap::new(),
            cite_ptr: 0,
            num_cites: 0,
            old_num_cites: 0,
            max_cites: 750,
            max_fields_cap: 5000,
            max_bib_files_cap: 20,
            max_glob_strs_cap: 10,
            lit_stk_size_cap: 50,
            entry_cite_ptr: 0,
            cite_xptr: 0,
            bst_line_num: 0,
            bbl_line_num: 0,
            fns: Vec::new(),
            fn_map: HashMap::new(),
            int_lits: HashMap::new(),
            str_lits: HashMap::new(),
            macros: HashMap::new(),
            wiz_functions: Vec::new(),
            wiz_fn_space: 3000,
            wiz_loc: 0,
            impl_fn_num: 0,
            b_default: 0,
            b_skip: 0,
            num_fields: 0,
            num_pre_defined_fields: 0,
            crossref_num: 0,
            num_ent_ints: 0,
            num_ent_strs: 0,
            sort_key_num: 0,
            globs: Vec::new(),
            entry_seen: false,
            read_seen: false,
            read_performed: false,
            reading_completed: false,
            read_completed: false,
            bib_line_num: 0,
            type_list: Vec::new(),
            entry_exists: Vec::new(),
            cite_info_str: Vec::new(),
            cite_info_cnt: Vec::new(),
            sorted_cites: Vec::new(),
            field_info: Vec::new(),
            entry_type: None,
            type_exists: false,
            store_entry: false,
            store_field: false,
            field_name_fn: None,
            right_outer_delim: 0,
            right_str_delim: 0,
            at_bib_command: false,
            command_num: 0,
            cur_macro: Vec::new(),
            bib_brace_level: 0,
            s_preamble: Vec::new(),
            preamble_ptr: 0,
            num_preamble_strings: 0,
            stack: Vec::new(),
            entry_ints: Vec::new(),
            entry_strs: Vec::new(),
            mess_with_entries: false,
            brace_level: 0,
            prev_colon: false,
            exec_count: [0; NUM_BLT_IN_FNS],
            builtin_locs: [0; NUM_BLT_IN_FNS],
            name_tok: vec![0; 20001],
            name_sep_char: vec![0; 20001],
        }
    }

    // ---------------------------------------------------------------- pool

    /// Records a string entering the string pool through `str_lookup`
    /// with `do_insert` (strings are shared between ilks, §70).
    pub(crate) fn intern(&mut self, s: &[u8]) {
        if !self.pool.contains(s) {
            self.pool_chars += s.len();
            self.pool.insert(s.to_vec());
        }
    }

    /// `pre_define_certain_strings` (§§75, 79, 334, 339, 340).
    pub(crate) fn pre_define(&mut self) {
        for s in [
            ".aux", ".bbl", ".blg", ".bst", ".bib", "texinputs:", "texbib:", "\\citation",
            "\\bibdata", "\\bibstyle", "\\@input", "entry", "execute", "function", "integers",
            "iterate", "macro", "read", "reverse", "sort", "strings", "comment", "preamble",
            "string",
        ] {
            self.intern(s.as_bytes());
        }
        for (i, name) in BUILTIN_NAMES.iter().enumerate() {
            let id = self.insert_fn(name.as_bytes(), FnClass::BuiltIn(i));
            self.builtin_locs[i] = id;
        }
        self.b_skip = self.builtin_locs[25];
        self.b_default = self.b_skip;
        self.intern(b"");
        self.intern(b"default.type");
        for s in ["i", "j", "oe", "OE", "ae", "AE", "aa", "AA", "o", "O", "l", "L", "ss"] {
            self.intern(s.as_bytes());
        }
        self.insert_fn(b"crossref", FnClass::Field(self.num_fields));
        self.crossref_num = self.num_fields;
        self.num_fields += 1;
        self.num_pre_defined_fields = self.num_fields;
        self.insert_fn(b"sort.key$", FnClass::StrEntry(self.num_ent_strs));
        self.sort_key_num = self.num_ent_strs;
        self.num_ent_strs += 1;
        let e = self.opts.ent_str_size as i32;
        let g = self.opts.glob_str_size as i32;
        self.insert_fn(b"entry.max$", FnClass::IntGlobal(e));
        self.insert_fn(b"global.max$", FnClass::IntGlobal(g));
    }

    fn insert_fn(&mut self, name: &[u8], class: FnClass) -> usize {
        self.intern(name);
        let id = self.fns.len();
        self.fns.push(Func { name: Rc::from(name), class });
        self.fn_map.insert(name.to_vec(), id);
        id
    }

    /// `str_lookup(..., bst_fn_ilk, do_insert)`: returns `(loc, hash_found)`.
    pub(crate) fn lookup_insert_fn(&mut self, name: &[u8]) -> (usize, bool) {
        if let Some(&id) = self.fn_map.get(name) {
            return (id, true);
        }
        (self.insert_fn(name, FnClass::BuiltIn(0)), false)
    }

    // ---------------------------------------------------------------- log

    pub(crate) fn mark_warning(&mut self) {
        if self.history == WARNING_MESSAGE {
            self.err_count += 1;
        } else if self.history == SPOTLESS {
            self.history = WARNING_MESSAGE;
            self.err_count = 1;
        }
    }

    pub(crate) fn mark_error(&mut self) {
        if self.history < ERROR_MESSAGE {
            self.history = ERROR_MESSAGE;
            self.err_count = 1;
        } else {
            self.err_count += 1;
        }
    }

    pub(crate) fn mark_fatal(&mut self) {
        self.history = FATAL_MESSAGE;
    }

    /// `confusion` (§45).
    pub(crate) fn confusion(&mut self, msg: &str) -> Flow {
        p!(self, msg);
        pln!(self, "---this can't happen");
        pln!(self, "*Please notify the BibTeX maintainer*");
        self.mark_fatal();
        Flow::Fatal
    }

    /// `overflow` (§44).
    pub(crate) fn overflow(&mut self, what: &str, n: usize) -> Flow {
        p!(self, "Sorry---you've exceeded BibTeX's ");
        self.mark_fatal();
        pln!(self, what, n);
        Flow::Fatal
    }

    pub(crate) fn print_token(&mut self) {
        let (a, b) = (self.buf_ptr1, self.buf_ptr2);
        self.log.extend_from_slice(&self.buffer[a..b]);
    }

    pub(crate) fn token(&self) -> Vec<u8> {
        self.buffer[self.buf_ptr1..self.buf_ptr2].to_vec()
    }

    /// `print_bad_input_line` (§95).
    pub(crate) fn print_bad_input_line(&mut self) {
        p!(self, " : ");
        for i in 0..self.buf_ptr2 {
            let c = self.buffer[i];
            self.log.push(if white(c) { b' ' } else { c });
        }
        self.log.push(b'\n');
        p!(self, " : ");
        for _ in 0..self.buf_ptr2 {
            self.log.push(b' ');
        }
        for i in self.buf_ptr2..self.last {
            let c = self.buffer[i];
            self.log.push(if white(c) { b' ' } else { c });
        }
        self.log.push(b'\n');
        let mut i = 0;
        while i < self.buf_ptr2 && white(self.buffer[i]) {
            i += 1;
        }
        if i == self.buf_ptr2 {
            pln!(self, "(Error may have been on previous line)");
        }
        self.mark_error();
    }

    pub(crate) fn print_skipping_whatever_remains(&mut self) {
        p!(self, "I'm skipping whatever remains of this ");
    }

    // ---------------------------------------------------------------- scanning (§§83–94)

    #[inline]
    pub(crate) fn sc(&self) -> u8 {
        self.buffer[self.buf_ptr2]
    }

    pub(crate) fn token_len(&self) -> usize {
        self.buf_ptr2 - self.buf_ptr1
    }

    pub(crate) fn scan1(&mut self, c1: u8) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        while self.sc() != c1 && self.buf_ptr2 < self.last {
            self.buf_ptr2 += 1;
        }
        self.buf_ptr2 < self.last
    }

    pub(crate) fn scan1_white(&mut self, c1: u8) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        while !white(self.sc()) && self.sc() != c1 && self.buf_ptr2 < self.last {
            self.buf_ptr2 += 1;
        }
        self.buf_ptr2 < self.last
    }

    pub(crate) fn scan2(&mut self, c1: u8, c2: u8) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        while self.sc() != c1 && self.sc() != c2 && self.buf_ptr2 < self.last {
            self.buf_ptr2 += 1;
        }
        self.buf_ptr2 < self.last
    }

    pub(crate) fn scan2_white(&mut self, c1: u8, c2: u8) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        while self.sc() != c1 && self.sc() != c2 && !white(self.sc()) && self.buf_ptr2 < self.last
        {
            self.buf_ptr2 += 1;
        }
        self.buf_ptr2 < self.last
    }

    pub(crate) fn scan3(&mut self, c1: u8, c2: u8, c3: u8) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        while self.sc() != c1 && self.sc() != c2 && self.sc() != c3 && self.buf_ptr2 < self.last {
            self.buf_ptr2 += 1;
        }
        self.buf_ptr2 < self.last
    }

    pub(crate) fn scan_alpha(&mut self) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        while lex(self.sc()) == ALPHA && self.buf_ptr2 < self.last {
            self.buf_ptr2 += 1;
        }
        self.token_len() != 0
    }

    pub(crate) fn scan_identifier(&mut self, c1: u8, c2: u8, c3: u8) {
        self.buf_ptr1 = self.buf_ptr2;
        if lex(self.sc()) != NUMERIC {
            while crate::chars::legal_id(self.sc()) && self.buf_ptr2 < self.last {
                self.buf_ptr2 += 1;
            }
        }
        let c = self.sc();
        self.scan_result = if self.token_len() == 0 {
            ScanResult::IdNull
        } else if white(c) || self.buf_ptr2 == self.last {
            ScanResult::WhiteAdjacent
        } else if c == c1 || c == c2 || c == c3 {
            ScanResult::SpecifiedCharAdjacent
        } else {
            ScanResult::OtherCharAdjacent
        };
    }

    pub(crate) fn scan_nonneg_integer(&mut self) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        self.token_value = 0;
        while lex(self.sc()) == NUMERIC && self.buf_ptr2 < self.last {
            self.token_value = self
                .token_value
                .wrapping_mul(10)
                .wrapping_add((self.sc() - b'0') as i32);
            self.buf_ptr2 += 1;
        }
        self.token_len() != 0
    }

    pub(crate) fn scan_integer(&mut self) -> bool {
        self.buf_ptr1 = self.buf_ptr2;
        let sign_length = if self.sc() == b'-' {
            self.buf_ptr2 += 1;
            1
        } else {
            0
        };
        self.token_value = 0;
        while lex(self.sc()) == NUMERIC && self.buf_ptr2 < self.last {
            self.token_value = self
                .token_value
                .wrapping_mul(10)
                .wrapping_add((self.sc() - b'0') as i32);
            self.buf_ptr2 += 1;
        }
        if sign_length == 1 {
            self.token_value = self.token_value.wrapping_neg();
        }
        self.token_len() != sign_length
    }

    pub(crate) fn scan_white_space(&mut self) -> bool {
        while white(self.sc()) && self.buf_ptr2 < self.last {
            self.buf_ptr2 += 1;
        }
        self.buf_ptr2 < self.last
    }

    pub(crate) fn lower_buffer_token(&mut self) {
        let (a, b) = (self.buf_ptr1, self.buf_ptr2);
        crate::chars::lower(&mut self.buffer[a..b]);
    }

    // ---------------------------------------------------------------- .aux (§§97–145)

    pub(crate) fn print_aux_name(&mut self) {
        let s = self.aux_list[self.aux_ptr].clone();
        pln!(self, s);
    }

    pub(crate) fn print_bib_name(&mut self) {
        let s = self.bib_list[self.bib_ptr].clone();
        p!(self, s);
        if !s.ends_with(b".bib") {
            p!(self, ".bib");
        }
        self.log.push(b'\n');
    }

    pub(crate) fn print_bst_name(&mut self) {
        let s = self.bst_str.clone().unwrap_or_else(|| Rc::from(&b""[..]));
        pln!(self, s, ".bst");
    }

    fn set_aux_slot(&mut self, i: usize, s: Str) {
        if self.aux_list.len() <= i {
            self.aux_list.resize(i + 1, Rc::from(&b""[..]));
            self.aux_ln_stack.resize(i + 1, 0);
        }
        self.aux_list[i] = s;
    }

    /// §§103–107: returns `false` when the top-level `.aux` cannot be opened.
    pub(crate) fn open_top_level_aux(&mut self, name: &str) -> bool {
        let mut base = name.as_bytes().to_vec();
        if base.len() >= 4 && base.ends_with(b".aux") {
            base.truncate(base.len() - 4);
        }
        let mut full = base.clone();
        full.extend_from_slice(b".aux");
        let data = match self.fs.read(FileKind::Aux, &String::from_utf8_lossy(&full)) {
            Some(d) => d,
            None => return false,
        };
        self.intern(&base);
        self.top_lev_str = Rc::from(&base[..]);
        self.intern(&full);
        self.aux_seen.insert(full.clone());
        self.set_aux_slot(0, Rc::from(&full[..]));
        self.aux_files.push(Some(InFile::new(Rc::from(data))));
        self.aux_ln_stack[0] = 0;
        true
    }

    /// §110.
    pub(crate) fn read_aux(&mut self) -> R<()> {
        p!(self, "The top-level auxiliary file: ");
        self.print_aux_name();
        loop {
            self.aux_ln_stack[self.aux_ptr] += 1;
            let ok = {
                let f = self.aux_files[self.aux_ptr].as_mut().unwrap();
                input_ln(f, &mut self.buffer, &mut self.last)
            };
            if !ok {
                // pop_the_aux_stack (§142)
                self.aux_files[self.aux_ptr] = None;
                if self.aux_ptr == 0 {
                    break;
                }
                self.aux_ptr -= 1;
            } else {
                self.get_aux_command_and_process()?;
            }
        }
        self.last_check_for_aux_errors();
        Ok(())
    }

    fn aux_err_print(&mut self) {
        let line = self.aux_ln_stack[self.aux_ptr];
        p!(self, "---line ", line, " of file ");
        self.print_aux_name();
        self.print_bad_input_line();
        self.print_skipping_whatever_remains();
        pln!(self, "command");
    }

    fn get_aux_command_and_process(&mut self) -> R<()> {
        self.buf_ptr2 = 0;
        if !self.scan1(b'{') {
            return Ok(());
        }
        let tok = self.token();
        match &tok[..] {
            b"\\bibdata" => self.aux_bib_data_command(),
            b"\\bibstyle" => self.aux_bib_style_command(),
            b"\\citation" => self.aux_citation_command(),
            b"\\@input" => self.aux_input_command(),
            _ => Ok(()),
        }
    }

    /// The three argument checks shared by the `.aux` commands; `true` if an
    /// error was reported (the caller must return).
    fn aux_arg_checks(&mut self, list: bool, scanned: bool) -> bool {
        if !scanned {
            p!(self, "No \"}\"");
            self.aux_err_print();
            return true;
        }
        if white(self.sc()) {
            p!(self, "White space in argument");
            self.aux_err_print();
            return true;
        }
        if self.last > self.buf_ptr2 + 1 && (!list || self.sc() == b'}') {
            p!(self, "Stuff after \"}\"");
            self.aux_err_print();
            return true;
        }
        false
    }

    fn aux_bib_data_command(&mut self) -> R<()> {
        if self.bib_seen {
            p!(self, "Illegal, another \\bibdata command");
            self.aux_err_print();
            return Ok(());
        }
        self.bib_seen = true;
        while self.sc() != b'}' {
            self.buf_ptr2 += 1;
            let ok = self.scan2_white(b'}', b',');
            if self.aux_arg_checks(true, ok) {
                return Ok(());
            }
            // §123
            let name = self.token();
            self.intern(&name);
            let found = !self.bib_seen_names.insert(name.clone());
            self.check_bib_files_overflow(self.bib_ptr);
            if self.bib_list.len() <= self.bib_ptr {
                self.bib_list.resize(self.bib_ptr + 1, Rc::from(&b""[..]));
                self.bib_files.resize_with(self.bib_ptr + 1, || None);
            }
            self.bib_list[self.bib_ptr] = Rc::from(&name[..]);
            if found {
                p!(self, "This database file appears more than once: ");
                self.print_bib_name();
                self.aux_err_print();
                return Ok(());
            }
            let n = String::from_utf8_lossy(&name).into_owned();
            match self.fs.read(FileKind::Bib, &n) {
                Some(d) => self.bib_files[self.bib_ptr] = Some(InFile::new(Rc::from(d))),
                None => {
                    p!(self, "I couldn't open database file ");
                    self.print_bib_name();
                    self.aux_err_print();
                    return Ok(());
                }
            }
            self.bib_ptr += 1;
        }
        Ok(())
    }

    fn aux_bib_style_command(&mut self) -> R<()> {
        if self.bst_seen {
            p!(self, "Illegal, another \\bibstyle command");
            self.aux_err_print();
            return Ok(());
        }
        self.bst_seen = true;
        self.buf_ptr2 += 1;
        let ok = self.scan1_white(b'}');
        if self.aux_arg_checks(false, ok) {
            return Ok(());
        }
        let name = self.token();
        self.intern(&name);
        self.bst_str = Some(Rc::from(&name[..]));
        let n = String::from_utf8_lossy(&name).into_owned();
        match self.fs.read(FileKind::Bst, &n) {
            Some(d) => self.bst_file = Some(InFile::new(Rc::from(d))),
            None => {
                p!(self, "I couldn't open style file ");
                self.print_bst_name();
                self.bst_str = None;
                self.aux_err_print();
                return Ok(());
            }
        }
        p!(self, "The style file: ");
        self.print_bst_name();
        Ok(())
    }

    fn aux_citation_command(&mut self) -> R<()> {
        self.citation_seen = true;
        while self.sc() != b'}' {
            self.buf_ptr2 += 1;
            let ok = self.scan2_white(b'}', b',');
            if self.aux_arg_checks(true, ok) {
                return Ok(());
            }
            // §134
            if self.token_len() == 1 && self.buffer[self.buf_ptr1] == b'*' {
                if self.all_entries {
                    pln!(self, "Multiple inclusions of entire database");
                    self.aux_err_print();
                    return Ok(());
                }
                self.all_entries = true;
                self.all_marker = self.cite_ptr;
                continue;
            }
            // §133
            let tok = self.token();
            let lc = lowered(&tok);
            self.copy_to_ex_buf(self.buf_ptr1, &lc);
            self.intern(&lc);
            if let Some(exact) = self.lc_cite.get(&lc).cloned() {
                // §135
                if !self.cite_idx.contains_key(&tok) {
                    p!(self, "Case mismatch error between cite keys ");
                    self.print_token();
                    p!(self, " and ");
                    let i = self.cite_idx[&exact];
                    let s = self.cite_list[i].clone();
                    pln!(self, s);
                    self.aux_err_print();
                    return Ok(());
                }
            } else {
                // §136
                self.intern(&tok);
                if self.cite_idx.contains_key(&tok) {
                    return Err(self.confusion("Cite hash error"));
                }
                self.check_cite_overflow();
                self.set_cite(self.cite_ptr, Rc::from(&tok[..]));
                self.cite_idx.insert(tok.clone(), self.cite_ptr);
                self.lc_cite.insert(lc, tok);
                self.cite_ptr += 1;
            }
        }
        Ok(())
    }

    pub(crate) fn copy_to_ex_buf(&mut self, at: usize, bytes: &[u8]) {
        if self.ex_buf.len() < at + bytes.len() + 1 {
            self.ex_buf.resize(at + bytes.len() + 20000, 0);
        }
        self.ex_buf[at..at + bytes.len()].copy_from_slice(bytes);
    }

    /// `check_cite_overflow` ([138]): checked right before a new cite key is
    /// stored at `self.cite_ptr`, i.e. `if (last_cite = max_cites)`.
    pub(crate) fn check_cite_overflow(&mut self) {
        while self.cite_ptr >= self.max_cites {
            let old = self.max_cites;
            self.max_cites += 750;
            let n = self.max_cites as i64;
            let o = old as i64;
            pln!(self, "Reallocated cite_list (elt_size=4) to ", n, " items from ", o, ".");
            pln!(self, "Reallocated type_list (elt_size=4) to ", n, " items from ", o, ".");
            pln!(self, "Reallocated entry_exists (elt_size=4) to ", n, " items from ", o, ".");
            pln!(self, "Reallocated cite_info (elt_size=4) to ", n, " items from ", o, ".");
        }
    }

    /// `check_field_overflow` ([226]): called with the final `total_fields
    /// = num_fields * num_cites`. Unlike the other capacity arrays this one
    /// does not grow in a fixed-size loop: it jumps straight to
    /// `total_fields + MAX_FIELDS`.
    pub(crate) fn check_field_overflow(&mut self, total_fields: usize) {
        if total_fields > self.max_fields_cap {
            let old = self.max_fields_cap;
            self.max_fields_cap = total_fields + 5000;
            pln!(
                self,
                "Reallocated field_info (elt_size=4) to ",
                self.max_fields_cap as i64,
                " items from ",
                old as i64,
                "."
            );
        }
    }

    /// `check_bib_files` ([242]/[123]): the `bib_list`/`bib_file`/`s_preamble`
    /// triple, checked before storing at index `self.bib_ptr` (or the
    /// preamble count), growing by `MAX_BIB_FILES=20` in a loop.
    pub(crate) fn check_bib_files_overflow(&mut self, next_index: usize) {
        while next_index >= self.max_bib_files_cap {
            let old = self.max_bib_files_cap;
            self.max_bib_files_cap += 20;
            let n = self.max_bib_files_cap as i64;
            let o = old as i64;
            pln!(self, "Reallocated bib_list (elt_size=4) to ", n, " items from ", o, ".");
            pln!(self, "Reallocated bib_file (elt_size=8) to ", n, " items from ", o, ".");
            pln!(self, "Reallocated s_preamble (elt_size=4) to ", n, " items from ", o, ".");
        }
    }

    /// `check_glob_str_overflow` ([216]): the `glb_str_ptr`/`global_strs`/
    /// `glb_str_end` triple, checked before storing a new global string
    /// variable, growing by `MAX_GLOB_STRS=10` in a loop. `global_strs`'
    /// `elt_size` is `glob_str_size + 1`, taken from `self.opts`.
    pub(crate) fn check_glob_str_overflow(&mut self, next_index: usize) {
        while next_index >= self.max_glob_strs_cap {
            let old = self.max_glob_strs_cap;
            self.max_glob_strs_cap += 10;
            let n = self.max_glob_strs_cap as i64;
            let o = old as i64;
            pln!(self, "Reallocated glb_str_ptr (elt_size=4) to ", n, " items from ", o, ".");
            pln!(
                self,
                "Reallocated global_strs (elt_size=",
                (self.opts.glob_str_size as i64) + 1,
                ") to ",
                n,
                " items from ",
                o,
                "."
            );
            pln!(self, "Reallocated glb_str_end (elt_size=4) to ", n, " items from ", o, ".");
        }
    }

    /// `check_lit_stk_overflow` ([307]): the `lit_stack`/`lit_stk_type` pair,
    /// checked before pushing a new literal-stack entry, growing by
    /// `LIT_STK_SIZE=50` in a loop.
    pub(crate) fn check_lit_stk_overflow(&mut self, next_index: usize) {
        while next_index >= self.lit_stk_size_cap {
            let old = self.lit_stk_size_cap;
            self.lit_stk_size_cap += 50;
            let n = self.lit_stk_size_cap as i64;
            let o = old as i64;
            pln!(self, "Reallocated lit_stack (elt_size=4) to ", n, " items from ", o, ".");
            pln!(self, "Reallocated lit_stk_type (elt_size=1) to ", n, " items from ", o, ".");
        }
    }

    pub(crate) fn set_cite(&mut self, i: usize, s: Str) {
        if self.cite_list.len() <= i {
            self.cite_list.resize(i + 1, Rc::from(&b""[..]));
        }
        self.cite_list[i] = s;
    }

    fn aux_input_command(&mut self) -> R<()> {
        self.buf_ptr2 += 1;
        let ok = self.scan1_white(b'}');
        if self.aux_arg_checks(false, ok) {
            return Ok(());
        }
        // §140
        self.aux_ptr += 1;
        if self.aux_ptr == 20 {
            self.print_token();
            p!(self, ": ");
            return Err(self.overflow("auxiliary file depth ", 20));
        }
        let tok = self.token();
        if tok.len() < 4 || !tok.ends_with(b".aux") {
            self.print_token();
            p!(self, " has a wrong extension");
            self.aux_ptr -= 1;
            self.aux_err_print();
            return Ok(());
        }
        self.intern(&tok);
        self.set_aux_slot(self.aux_ptr, Rc::from(&tok[..]));
        if !self.aux_seen.insert(tok.clone()) {
            p!(self, "Already encountered file ");
            self.print_aux_name();
            self.aux_ptr -= 1;
            self.aux_err_print();
            return Ok(());
        }
        // §141
        let n = String::from_utf8_lossy(&tok).into_owned();
        let data = self.fs.read(FileKind::Aux, &n);
        match data {
            None => {
                p!(self, "I couldn't open auxiliary file ");
                self.print_aux_name();
                self.aux_ptr -= 1;
                self.aux_err_print();
                return Ok(());
            }
            Some(d) => {
                if self.aux_files.len() <= self.aux_ptr {
                    self.aux_files.resize_with(self.aux_ptr + 1, || None);
                }
                self.aux_files[self.aux_ptr] = Some(InFile::new(Rc::from(d)));
            }
        }
        p!(self, "A level-", self.aux_ptr, " auxiliary file: ");
        self.print_aux_name();
        self.aux_ln_stack[self.aux_ptr] = 0;
        Ok(())
    }

    fn aux_end_err(&mut self, what: &str) {
        p!(self, "I found no ", what, "---while reading file ");
        self.print_aux_name();
        self.mark_error();
    }

    /// §145.
    fn last_check_for_aux_errors(&mut self) {
        self.num_cites = self.cite_ptr;
        self.num_bib_files = self.bib_ptr;
        if !self.citation_seen {
            self.aux_end_err("\\citation commands");
        } else if self.num_cites == 0 && !self.all_entries {
            self.aux_end_err("cite keys");
        }
        if !self.bib_seen {
            self.aux_end_err("\\bibdata command");
        } else if self.num_bib_files == 0 {
            self.aux_end_err("database files");
        }
        if !self.bst_seen {
            self.aux_end_err("\\bibstyle command");
        } else if self.bst_str.is_none() {
            self.aux_end_err("style file");
        }
    }

    // ---------------------------------------------------------------- clean up (§§455–466)

    pub(crate) fn clean_up(&mut self) {
        if self.read_performed && !self.reading_completed {
            p!(self, "Aborted at line ", self.bib_line_num, " of file ");
            self.print_bib_name();
        }
        // §465 (stat): the usage statistics.
        p!(self, "You've used ", self.num_cites);
        if self.num_cites == 1 {
            pln!(self, " entry,");
        } else {
            pln!(self, " entries,");
        }
        pln!(self, "            ", self.wiz_functions.len(), " wiz_defined-function locations,");
        pln!(
            self,
            "            ",
            self.pool.len() + 1,
            " strings with ",
            self.pool_chars,
            " characters,"
        );
        let total: i64 = self.exec_count.iter().sum();
        pln!(self, "and the built_in function-call counts, ", total, " in all, are:");
        for i in 0..NUM_BLT_IN_FNS {
            pln!(self, BUILTIN_NAMES[i], " -- ", self.exec_count[i]);
        }
        // §466
        match self.history {
            SPOTLESS => {}
            WARNING_MESSAGE => {
                if self.err_count == 1 {
                    pln!(self, "(There was 1 warning)");
                } else {
                    pln!(self, "(There were ", self.err_count, " warnings)");
                }
            }
            ERROR_MESSAGE => {
                if self.err_count == 1 {
                    pln!(self, "(There was 1 error message)");
                } else {
                    pln!(self, "(There were ", self.err_count, " error messages)");
                }
            }
            _ => pln!(self, "(That was a fatal error)"),
        }
    }
}
