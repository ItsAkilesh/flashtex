//! Reading the database files: §§218–289.

use std::rc::Rc;

use crate::chars::{lowered, white};
use crate::engine::{input_ln, p, pln, Ch, Engine, FnClass, ScanResult, Str, TypeRef, R};

const N_BIB_PREAMBLE: u8 = 1;
const N_BIB_STRING: u8 = 2;

impl<'a> Engine<'a> {
    /// Makes the per-cite arrays large enough for cite number `i`.
    fn ensure_cite(&mut self, i: usize) {
        let n = i + 1;
        if self.type_list.len() < n {
            self.type_list.resize(n, TypeRef::Empty);
            self.entry_exists.resize(n, false);
            self.cite_info_str.resize(n, None);
            self.cite_info_cnt.resize(n, 0);
        }
        if self.cite_list.len() < n {
            self.cite_list.resize(n, Rc::from(&b""[..]));
        }
        let nf = self.num_fields * n;
        if self.field_info.len() < nf {
            self.field_info.resize(nf, None);
        }
    }

    /// §223.
    pub(crate) fn read_bib_files(&mut self) -> R<()> {
        // §§224–227
        self.field_info = vec![None; self.num_fields * self.num_cites];
        self.type_list = vec![TypeRef::Empty; self.num_cites];
        self.entry_exists = vec![false; self.num_cites];
        self.cite_info_str = vec![None; self.num_cites];
        self.cite_info_cnt = vec![0; self.num_cites];
        self.old_num_cites = self.num_cites;
        if self.all_entries {
            for cp in self.all_marker..self.old_num_cites {
                self.cite_info_str[cp] = Some(self.cite_list[cp].clone());
                self.entry_exists[cp] = false;
            }
            self.cite_ptr = self.all_marker;
        } else {
            self.cite_ptr = self.num_cites;
            self.all_marker = 0;
        }
        self.read_performed = true;
        self.bib_ptr = 0;
        while self.bib_ptr < self.num_bib_files {
            p!(self, "Database file #", self.bib_ptr + 1, ": ");
            self.print_bib_name();
            self.bib_line_num = 0;
            self.buf_ptr2 = self.last;
            while !self.bib_files[self.bib_ptr].as_ref().unwrap().eof() {
                self.get_bib_command_or_entry_and_process()?;
            }
            self.bib_files[self.bib_ptr] = None;
            self.bib_ptr += 1;
        }
        self.reading_completed = true;
        self.final_init_for_entries()?;
        self.read_completed = true;
        Ok(())
    }

    fn bib_ln_num_print(&mut self) {
        p!(self, "--line ", self.bib_line_num, " of file ");
        self.print_bib_name();
    }

    /// `bib_err` (§221): print the message first, then call this.
    fn bib_err_print(&mut self) {
        p!(self, "-");
        self.bib_ln_num_print();
        self.print_bad_input_line();
        self.print_skipping_whatever_remains();
        if self.at_bib_command {
            pln!(self, "command");
        } else {
            pln!(self, "entry");
        }
    }

    fn bib_warn_print(&mut self) {
        self.bib_ln_num_print();
        self.mark_warning();
    }

    fn bib_input_ln(&mut self) -> bool {
        let f = self.bib_files[self.bib_ptr].as_mut().unwrap();
        input_ln(f, &mut self.buffer, &mut self.last)
    }

    /// §228.
    fn eat_bib_white_space(&mut self) -> bool {
        while !self.scan_white_space() {
            if !self.bib_input_ln() {
                return false;
            }
            self.bib_line_num += 1;
            self.buf_ptr2 = 0;
        }
        true
    }

    /// `eat_bib_white_and_eof_check` (§229): `false` means "return".
    fn eat_bib(&mut self) -> bool {
        if !self.eat_bib_white_space() {
            self.eat_bib_print();
            return false;
        }
        true
    }

    fn eat_bib_print(&mut self) {
        p!(self, "Illegal end of database file");
        self.bib_err_print();
    }

    fn bib_one_of_two_print(&mut self, c1: u8, c2: u8) {
        p!(self, "I was expecting a `", Ch(c1), "' or a `", Ch(c2), "'");
        self.bib_err_print();
    }

    fn bib_equals_sign_print(&mut self) {
        p!(self, "I was expecting an \"=\"");
        self.bib_err_print();
    }

    /// `bib_identifier_scan_check` (§235): `false` means "return".
    fn bib_id_check(&mut self, what: &str) -> bool {
        match self.scan_result {
            ScanResult::WhiteAdjacent | ScanResult::SpecifiedCharAdjacent => true,
            r => {
                if r == ScanResult::IdNull {
                    p!(self, "You're missing ");
                } else {
                    let c = self.sc();
                    p!(self, "\"", Ch(c), "\" immediately follows ");
                }
                p!(self, what);
                self.bib_err_print();
                false
            }
        }
    }

    fn macro_name_warning(&mut self, what: &str) {
        p!(self, "Warning--string name \"");
        self.print_token();
        p!(self, "\" is ");
        pln!(self, what);
        self.bib_warn_print();
    }

    /// §236.
    fn get_bib_command_or_entry_and_process(&mut self) -> R<()> {
        self.at_bib_command = false;
        // §237
        while !self.scan1(b'@') {
            if !self.bib_input_ln() {
                return Ok(());
            }
            self.bib_line_num += 1;
            self.buf_ptr2 = 0;
        }
        // §238
        if self.sc() != b'@' {
            return Err(self.confusion("An \"@\" disappeared"));
        }
        self.buf_ptr2 += 1;
        if !self.eat_bib() {
            return Ok(());
        }
        self.scan_identifier(b'{', b'(', b'(');
        if !self.bib_id_check("an entry type") {
            return Ok(());
        }
        self.lower_buffer_token();
        let tok = self.token();
        let cmd = match &tok[..] {
            b"comment" => Some(0u8),
            b"preamble" => Some(N_BIB_PREAMBLE),
            b"string" => Some(N_BIB_STRING),
            _ => None,
        };
        if let Some(c) = cmd {
            // §239
            self.at_bib_command = true;
            self.command_num = c;
            match c {
                0 => return Ok(()),
                N_BIB_PREAMBLE => return self.bib_preamble_command(),
                _ => return self.bib_string_command(),
            }
        }
        match self.fn_map.get(&tok) {
            Some(&id) if matches!(self.fns[id].class, FnClass::Wiz(_)) => {
                self.type_exists = true;
                self.entry_type = Some(id);
            }
            _ => {
                self.type_exists = false;
            }
        }
        if !self.eat_bib() {
            return Ok(());
        }
        // §266
        if !self.scan_outer_left_delim() {
            return Ok(());
        }
        if !self.eat_bib() {
            return Ok(());
        }
        if self.right_outer_delim == b')' {
            self.scan1_white(b',');
        } else {
            self.scan2_white(b',', b'}');
        }
        if !self.check_database_key()? {
            return Ok(());
        }
        if !self.eat_bib() {
            return Ok(());
        }
        // §274
        while self.sc() != self.right_outer_delim {
            if self.sc() != b',' {
                let rod = self.right_outer_delim;
                self.bib_one_of_two_print(b',', rod);
                return Ok(());
            }
            self.buf_ptr2 += 1;
            if !self.eat_bib() {
                return Ok(());
            }
            if self.sc() == self.right_outer_delim {
                break;
            }
            // §275
            self.scan_identifier(b'=', b'=', b'=');
            if !self.bib_id_check("a field name") {
                return Ok(());
            }
            self.store_field = false;
            if self.store_entry {
                self.lower_buffer_token();
                let name = self.token();
                if let Some(&id) = self.fn_map.get(&name) {
                    if matches!(self.fns[id].class, FnClass::Field(_)) {
                        self.store_field = true;
                        self.field_name_fn = Some(id);
                    }
                }
            }
            if !self.eat_bib() {
                return Ok(());
            }
            if self.sc() != b'=' {
                self.bib_equals_sign_print();
                return Ok(());
            }
            self.buf_ptr2 += 1;
            if !self.eat_bib() {
                return Ok(());
            }
            if !self.scan_and_store_the_field_value_and_eat_white()? {
                return Ok(());
            }
        }
        self.buf_ptr2 += 1;
        Ok(())
    }

    /// The `{`/`(` choice of §§242, 244, 266 (including the skip).
    fn scan_outer_left_delim(&mut self) -> bool {
        if self.sc() == b'{' {
            self.right_outer_delim = b'}';
        } else if self.sc() == b'(' {
            self.right_outer_delim = b')';
        } else {
            self.bib_one_of_two_print(b'{', b'(');
            return false;
        }
        self.buf_ptr2 += 1;
        true
    }

    /// §242.
    fn bib_preamble_command(&mut self) -> R<()> {
        if !self.eat_bib() || !self.scan_outer_left_delim() || !self.eat_bib() {
            return Ok(());
        }
        self.store_field = true;
        if !self.scan_and_store_the_field_value_and_eat_white()? {
            return Ok(());
        }
        if self.sc() != self.right_outer_delim {
            let rod = self.right_outer_delim;
            p!(self, "Missing \"", Ch(rod), "\" in preamble command");
            self.bib_err_print();
            return Ok(());
        }
        self.buf_ptr2 += 1;
        Ok(())
    }

    /// §§243–246.
    fn bib_string_command(&mut self) -> R<()> {
        if !self.eat_bib() || !self.scan_outer_left_delim() || !self.eat_bib() {
            return Ok(());
        }
        self.scan_identifier(b'=', b'=', b'=');
        if !self.bib_id_check("a string name") {
            return Ok(());
        }
        // §245
        self.lower_buffer_token();
        let name = self.token();
        self.intern(&name);
        self.macros.insert(name.clone(), Rc::from(&name[..]));
        self.cur_macro = name;
        if !self.eat_bib() {
            return Ok(());
        }
        // §246
        if self.sc() != b'=' {
            self.bib_equals_sign_print();
            return Ok(());
        }
        self.buf_ptr2 += 1;
        if !self.eat_bib() {
            return Ok(());
        }
        self.store_field = true;
        if !self.scan_and_store_the_field_value_and_eat_white()? {
            return Ok(());
        }
        if self.sc() != self.right_outer_delim {
            let rod = self.right_outer_delim;
            p!(self, "Missing \"", Ch(rod), "\" in string command");
            self.bib_err_print();
            return Ok(());
        }
        self.buf_ptr2 += 1;
        Ok(())
    }

    // ------------------------------------------------------------ field values

    fn copy_char(&mut self, c: u8) {
        let i = self.ex_buf_ptr as usize;
        if i + 2 >= self.ex_buf.len() {
            let n = self.ex_buf.len() + 20000;
            self.ex_buf.resize(n, 0);
        }
        self.ex_buf[i] = c;
        self.ex_buf_ptr += 1;
    }

    /// §249.
    fn scan_and_store_the_field_value_and_eat_white(&mut self) -> R<bool> {
        self.ex_buf_ptr = 0;
        if !self.scan_a_field_token_and_eat_white() {
            return Ok(false);
        }
        while self.sc() == b'#' {
            self.buf_ptr2 += 1;
            if !self.eat_bib() {
                return Ok(false);
            }
            if !self.scan_a_field_token_and_eat_white() {
                return Ok(false);
            }
        }
        if self.store_field {
            self.store_the_field_value()?;
        }
        Ok(true)
    }

    /// §250.
    fn scan_a_field_token_and_eat_white(&mut self) -> bool {
        match self.sc() {
            b'{' => {
                self.right_str_delim = b'}';
                if !self.scan_balanced_braces() {
                    return false;
                }
            }
            b'"' => {
                self.right_str_delim = b'"';
                if !self.scan_balanced_braces() {
                    return false;
                }
            }
            b'0'..=b'9' => {
                // §258
                self.scan_nonneg_integer();
                if self.store_field {
                    for i in self.buf_ptr1..self.buf_ptr2 {
                        let c = self.buffer[i];
                        self.copy_char(c);
                    }
                }
            }
            _ => {
                // §259
                let rod = self.right_outer_delim;
                self.scan_identifier(b',', rod, b'#');
                if !self.bib_id_check("a field part") {
                    return false;
                }
                if self.store_field {
                    self.lower_buffer_token();
                    let name = self.token();
                    let def = self.macros.get(&name).cloned();
                    let mut store_token = true;
                    if self.at_bib_command
                        && self.command_num == N_BIB_STRING
                        && def.is_some()
                        && name == self.cur_macro
                    {
                        store_token = false;
                        self.macro_name_warning("used in its own definition");
                    }
                    if def.is_none() {
                        store_token = false;
                        self.macro_name_warning("undefined");
                    }
                    if store_token {
                        // §260
                        let s = def.unwrap();
                        let mut t = 0;
                        let e = s.len();
                        if self.ex_buf_ptr == 0 && t < e && white(s[t]) {
                            self.copy_char(b' ');
                            t += 1;
                            while t < e && white(s[t]) {
                                t += 1;
                            }
                        }
                        while t < e {
                            if !white(s[t]) {
                                self.copy_char(s[t]);
                            } else if self.exb(self.ex_buf_ptr - 1) != b' ' {
                                self.copy_char(b' ');
                            }
                            t += 1;
                        }
                    }
                }
            }
        }
        self.eat_bib()
    }

    /// `check_for_and_compress_bib_white_space` (§252): `false` means "return".
    fn check_compress(&mut self) -> bool {
        if white(self.sc()) || self.buf_ptr2 == self.last {
            // compress_bib_white
            self.copy_char(b' ');
            while !self.scan_white_space() {
                if !self.bib_input_ln() {
                    self.eat_bib_print();
                    return false;
                }
                self.bib_line_num += 1;
                self.buf_ptr2 = 0;
            }
        }
        true
    }

    /// §253.
    fn scan_balanced_braces(&mut self) -> bool {
        self.buf_ptr2 += 1;
        if !self.check_compress() {
            return false;
        }
        let fe = self.ex_buf_ptr;
        if fe > 1 && self.exb(fe - 1) == b' ' && self.exb(fe - 2) == b' ' {
            self.ex_buf_ptr -= 1;
        }
        self.bib_brace_level = 0;
        let rsd = self.right_str_delim;
        if self.store_field {
            // §256
            while self.sc() != rsd {
                match self.sc() {
                    b'{' => {
                        self.bib_brace_level += 1;
                        self.copy_char(b'{');
                        self.buf_ptr2 += 1;
                        if !self.check_compress() {
                            return false;
                        }
                        // §257
                        loop {
                            match self.sc() {
                                b'}' => {
                                    self.bib_brace_level -= 1;
                                    self.copy_char(b'}');
                                    self.buf_ptr2 += 1;
                                    if !self.check_compress() {
                                        return false;
                                    }
                                    if self.bib_brace_level == 0 {
                                        break;
                                    }
                                }
                                b'{' => {
                                    self.bib_brace_level += 1;
                                    self.copy_char(b'{');
                                    self.buf_ptr2 += 1;
                                    if !self.check_compress() {
                                        return false;
                                    }
                                }
                                c => {
                                    self.copy_char(c);
                                    self.buf_ptr2 += 1;
                                    if !self.check_compress() {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                    b'}' => {
                        p!(self, "Unbalanced braces");
                        self.bib_err_print();
                        return false;
                    }
                    c => {
                        self.copy_char(c);
                        self.buf_ptr2 += 1;
                        if !self.check_compress() {
                            return false;
                        }
                    }
                }
            }
        } else {
            // §254
            while self.sc() != rsd {
                if self.sc() == b'{' {
                    self.bib_brace_level += 1;
                    self.buf_ptr2 += 1;
                    if !self.eat_bib() {
                        return false;
                    }
                    while self.bib_brace_level > 0 {
                        // §255
                        if self.sc() == b'}' {
                            self.bib_brace_level -= 1;
                            self.buf_ptr2 += 1;
                            if !self.eat_bib() {
                                return false;
                            }
                        } else if self.sc() == b'{' {
                            self.bib_brace_level += 1;
                            self.buf_ptr2 += 1;
                            if !self.eat_bib() {
                                return false;
                            }
                        } else {
                            self.buf_ptr2 += 1;
                            if !self.scan2(b'}', b'{') && !self.eat_bib() {
                                return false;
                            }
                        }
                    }
                } else if self.sc() == b'}' {
                    p!(self, "Unbalanced braces");
                    self.bib_err_print();
                    return false;
                } else {
                    self.buf_ptr2 += 1;
                    if !self.scan3(rsd, b'{', b'}') && !self.eat_bib() {
                        return false;
                    }
                }
            }
        }
        self.buf_ptr2 += 1;
        true
    }

    /// §261.
    fn store_the_field_value(&mut self) -> R<()> {
        if !self.at_bib_command && self.ex_buf_ptr > 0 && self.exb(self.ex_buf_ptr - 1) == b' ' {
            self.ex_buf_ptr -= 1;
        }
        self.ex_buf_xptr = if !self.at_bib_command && self.exb(0) == b' ' && self.ex_buf_ptr > 0 {
            1
        } else {
            0
        };
        let (fs, fe) = (self.ex_buf_xptr as usize, self.ex_buf_ptr as usize);
        let val: Vec<u8> = self.ex_buf[fs..fe].to_vec();
        self.intern(&val);
        let sval: Str = Rc::from(&val[..]);
        if self.at_bib_command {
            // §262
            if self.command_num == N_BIB_PREAMBLE {
                self.s_preamble.push(sval);
                self.preamble_ptr += 1;
            } else {
                let name = self.cur_macro.clone();
                self.macros.insert(name, sval);
            }
            return Ok(());
        }
        // §263
        let fid = self.field_name_fn.unwrap();
        let fnum = match self.fns[fid].class {
            FnClass::Field(k) => k,
            _ => 0,
        };
        let fp = self.entry_cite_ptr * self.num_fields + fnum;
        if self.field_info[fp].is_some() {
            p!(self, "Warning--I'm ignoring ");
            let key = self.cite_list[self.entry_cite_ptr].clone();
            let name = self.fns[fid].name.clone();
            p!(self, key, "'s extra \"", name);
            pln!(self, "\" field");
            self.bib_warn_print();
        } else {
            self.field_info[fp] = Some(sval);
            if fnum == self.crossref_num && !self.all_entries {
                // §264
                let lc = lowered(&val);
                if self.out_buf.len() < fe + 1 {
                    self.out_buf.resize(fe + 20000, 0);
                }
                self.out_buf[fs..fe].copy_from_slice(&lc);
                self.intern(&lc);
                if let Some(exact) = self.lc_cite.get(&lc) {
                    let ci = self.cite_idx[exact];
                    if ci >= self.old_num_cites {
                        self.cite_info_cnt[ci] += 1;
                    }
                } else {
                    self.intern(&val);
                    if self.cite_idx.contains_key(&val) {
                        return Err(self.confusion("Cite hash error"));
                    }
                    let new = self.cite_ptr;
                    self.add_database_cite(val.clone(), lc);
                    self.cite_info_cnt[new] = 1;
                }
            }
        }
        Ok(())
    }

    /// `add_database_cite` (§265) for the cite key `exact` and its
    /// lower-case form `lc`; increments `cite_ptr`.
    fn add_database_cite(&mut self, exact: Vec<u8>, lc: Vec<u8>) {
        let new = self.cite_ptr;
        self.ensure_cite(new);
        self.cite_list[new] = Rc::from(&exact[..]);
        self.cite_idx.insert(exact.clone(), new);
        self.lc_cite.insert(lc, exact);
        self.cite_ptr += 1;
    }

    /// §§267–273: `Ok(false)` means "return" (after a `Repeated entry` error).
    fn check_database_key(&mut self) -> R<bool> {
        let tok = self.token();
        let lc = lowered(&tok);
        self.copy_to_ex_buf(self.buf_ptr1, &lc);
        if self.all_entries {
            self.intern(&lc);
        }
        let found = self.lc_cite.contains_key(&lc);
        if found {
            let exact = self.lc_cite[&lc].clone();
            self.entry_cite_ptr = self.cite_idx[&exact];
            let ecp = self.entry_cite_ptr;
            // §268
            let mut first_time = false;
            if !self.all_entries || ecp < self.all_marker || ecp >= self.old_num_cites {
                if self.type_list[ecp] == TypeRef::Empty {
                    // §269
                    if !self.all_entries && ecp >= self.old_num_cites {
                        self.intern(&tok);
                        if !self.cite_idx.contains_key(&tok) {
                            self.lc_cite.insert(lc.clone(), tok.clone());
                            self.cite_idx.insert(tok.clone(), ecp);
                            self.cite_list[ecp] = Rc::from(&tok[..]);
                        }
                    }
                    first_time = true;
                }
            } else if !self.entry_exists[ecp] {
                // §270
                let orig = self.cite_info_str[ecp].clone().unwrap_or_else(|| Rc::from(&b""[..]));
                let lcx = lowered(&orig);
                self.copy_to_ex_buf(0, &lcx);
                if !self.lc_cite.contains_key(&lcx) {
                    return Err(self.confusion("A cite key disappeared"));
                }
                if lcx == lc {
                    first_time = true;
                }
            }
            if !first_time {
                if self.type_list[ecp] == TypeRef::Empty {
                    return Err(self.confusion("The cite list is messed up"));
                }
                p!(self, "Repeated entry");
                self.bib_err_print();
                return Ok(false);
            }
        }
        self.store_entry = true;
        if self.all_entries {
            // §272
            let mut set = true;
            if found {
                if self.entry_cite_ptr < self.all_marker {
                    set = false;
                } else {
                    let ecp = self.entry_cite_ptr;
                    self.entry_exists[ecp] = true;
                    let exact = self.lc_cite[&lc].clone();
                    self.entry_cite_ptr = self.cite_ptr;
                    self.add_database_cite(exact, lc.clone());
                }
            } else {
                self.intern(&tok);
                if self.cite_idx.contains_key(&tok) {
                    return Err(self.confusion("Cite hash error"));
                }
                self.entry_cite_ptr = self.cite_ptr;
                self.add_database_cite(tok.clone(), lc.clone());
            }
            let _ = set;
        } else if !found {
            self.store_entry = false;
        }
        if self.store_entry {
            // §273
            let ecp = self.entry_cite_ptr;
            if self.type_exists {
                self.type_list[ecp] = TypeRef::Fn(self.entry_type.unwrap());
            } else {
                self.type_list[ecp] = TypeRef::Undefined;
                p!(self, "Warning--entry type for \"");
                self.print_token();
                pln!(self, "\" isn't style-file defined");
                self.bib_warn_print();
            }
        }
        Ok(true)
    }

    // ------------------------------------------------------------ §276

    /// `find_cite_locs_for_this_cite_key` (§278): returns
    /// `(lc found, exact found, lc key)`.
    fn find_cite_locs(&mut self, s: &[u8]) -> (bool, bool, Vec<u8>) {
        self.copy_to_ex_buf(0, s);
        let exact_found = self.cite_idx.contains_key(s);
        let lc = lowered(s);
        self.copy_to_ex_buf(0, &lc);
        (self.lc_cite.contains_key(&lc), exact_found, lc)
    }

    fn final_init_for_entries(&mut self) -> R<()> {
        self.num_cites = self.cite_ptr;
        self.num_preamble_strings = self.preamble_ptr;
        if self.num_cites > 0 {
            self.ensure_cite(self.num_cites - 1);
        }
        let nf = self.num_fields;
        let cr = self.crossref_num;
        // §277
        for cp in 0..self.num_cites {
            let fp = cp * nf + cr;
            if let Some(v) = self.field_info[fp].clone() {
                let (ok, _, lc) = self.find_cite_locs(&v);
                if ok {
                    let exact = self.lc_cite[&lc].clone();
                    self.field_info[fp] = Some(Rc::from(&exact[..]));
                    let parent = self.cite_idx[&exact];
                    for k in self.num_pre_defined_fields..nf {
                        if self.field_info[cp * nf + k].is_none() {
                            self.field_info[cp * nf + k] = self.field_info[parent * nf + k].clone();
                        }
                    }
                }
            }
        }
        // §279
        for cp in 0..self.num_cites {
            let fp = cp * nf + cr;
            if let Some(v) = self.field_info[fp].clone() {
                let (ok, exact_found, lc) = self.find_cite_locs(&v);
                if !ok {
                    if exact_found {
                        return Err(self.confusion("Cite hash error"));
                    }
                    self.nonexistent_cross_reference_error(cp, &v);
                    self.field_info[fp] = None;
                } else {
                    if !exact_found || self.lc_cite[&lc][..] != v[..] {
                        return Err(self.confusion("Cite hash error"));
                    }
                    let parent = self.cite_idx[&v[..]];
                    if self.type_list[parent] == TypeRef::Empty {
                        self.nonexistent_cross_reference_error(cp, &v);
                        self.field_info[fp] = None;
                    } else {
                        if self.field_info[parent * nf + cr].is_some() {
                            // §282
                            p!(self, "Warning--you've nested cross references");
                            let ps = self.cite_list[parent].clone();
                            self.bad_cross_reference_print(cp, &ps);
                            pln!(self, "\", which also refers to something");
                            self.mark_warning();
                        }
                        if !self.all_entries
                            && parent >= self.old_num_cites
                            && self.cite_info_cnt[parent] < self.opts.min_crossrefs
                        {
                            self.field_info[fp] = None;
                        }
                    }
                }
            }
        }
        // §283
        for cp in 0..self.num_cites {
            if self.type_list[cp] == TypeRef::Empty {
                let s = self.cite_list[cp].clone();
                self.print_missing_entry(&s);
            } else if self.all_entries
                || cp < self.old_num_cites
                || self.cite_info_cnt[cp] >= self.opts.min_crossrefs
            {
                if cp > self.cite_xptr {
                    // §285
                    let x = self.cite_xptr;
                    self.cite_list[x] = self.cite_list[cp].clone();
                    self.type_list[x] = self.type_list[cp];
                    let key = self.cite_list[cp].clone();
                    let (ok, exact_found, lc) = self.find_cite_locs(&key);
                    if !ok {
                        return Err(self.confusion("A cite key disappeared"));
                    }
                    if !exact_found || self.lc_cite[&lc][..] != key[..] {
                        return Err(self.confusion("Cite hash error"));
                    }
                    self.cite_idx.insert(key.to_vec(), x);
                    for k in 0..nf {
                        self.field_info[x * nf + k] = self.field_info[cp * nf + k].clone();
                    }
                }
                self.cite_xptr += 1;
            }
        }
        self.num_cites = self.cite_xptr;
        if self.all_entries {
            // §286
            for cp in self.all_marker..self.old_num_cites {
                if !self.entry_exists[cp] {
                    let s = self.cite_info_str[cp].clone().unwrap();
                    self.print_missing_entry(&s);
                }
            }
        }
        // §§287–289
        self.entry_ints = vec![0; self.num_ent_ints * self.num_cites];
        self.entry_strs = vec![Vec::new(); self.num_ent_strs * self.num_cites];
        self.sorted_cites = (0..self.num_cites).collect();
        Ok(())
    }

    fn bad_cross_reference_print(&mut self, cp: usize, s: &[u8]) {
        let key = self.cite_list[cp].clone();
        p!(self, "--entry \"", key);
        pln!(self, "\"");
        p!(self, "refers to entry \"", s);
    }

    fn nonexistent_cross_reference_error(&mut self, cp: usize, s: &[u8]) {
        p!(self, "A bad cross reference-");
        self.bad_cross_reference_print(cp, s);
        pln!(self, "\", which doesn't exist");
        self.mark_error();
    }

    fn print_missing_entry(&mut self, s: &[u8]) {
        p!(self, "Warning--I didn't find a database entry for \"", s);
        pln!(self, "\"");
        self.mark_warning();
    }

    pub(crate) fn exb(&self, i: i64) -> u8 {
        if i >= 0 && (i as usize) < self.ex_buf.len() {
            self.ex_buf[i as usize]
        } else {
            0
        }
    }
}
