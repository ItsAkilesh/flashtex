//! Reading and executing the style file: §§146–217.

use std::rc::Rc;

use crate::chars::{lex, lower, white};
use crate::engine::{input_ln, p, pln, Engine, Flow, FnClass, Glob, ScanResult, WizItem, R};

impl<'a> Engine<'a> {
    /// §151.
    pub(crate) fn read_bst(&mut self) -> R<()> {
        if self.bst_str.is_none() {
            return Ok(());
        }
        self.bst_line_num = 0;
        self.bbl_line_num = 1;
        self.buf_ptr2 = self.last;
        let r = (|| -> R<()> {
            loop {
                if !self.eat_bst_white_space() {
                    return Ok(());
                }
                self.get_bst_command_and_process()?;
            }
        })();
        match r {
            Ok(()) | Err(Flow::BstDone) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn bst_ln_num_print(&mut self) {
        p!(self, "--line ", self.bst_line_num, " of file ");
        self.print_bst_name();
    }

    /// §149.
    pub(crate) fn bst_err_print_and_look_for_blank_line(&mut self) -> R<()> {
        p!(self, "-");
        self.bst_ln_num_print();
        self.print_bad_input_line();
        while self.last != 0 {
            let ok = {
                let f = self.bst_file.as_mut().unwrap();
                input_ln(f, &mut self.buffer, &mut self.last)
            };
            if !ok {
                return Err(Flow::BstDone);
            }
            self.bst_line_num += 1;
        }
        self.buf_ptr2 = self.last;
        Ok(())
    }

    /// §150.
    pub(crate) fn bst_warn_print(&mut self) {
        self.bst_ln_num_print();
        self.mark_warning();
    }

    /// §152.
    fn eat_bst_white_space(&mut self) -> bool {
        loop {
            if self.scan_white_space() && self.sc() != b'%' {
                return true;
            }
            let ok = {
                let f = self.bst_file.as_mut().unwrap();
                input_ln(f, &mut self.buffer, &mut self.last)
            };
            if !ok {
                return false;
            }
            self.bst_line_num += 1;
            self.buf_ptr2 = 0;
        }
    }

    /// `eat_bst_white_and_eof_check` (§153); `Ok(false)` means "return".
    fn eat_check(&mut self, cmd: &str) -> R<bool> {
        if !self.eat_bst_white_space() {
            p!(self, "Illegal end of style file in command: ", cmd);
            self.bst_err_print_and_look_for_blank_line()?;
            return Ok(false);
        }
        Ok(true)
    }

    /// `bst_get_and_check_left_brace` (§167).
    fn left_brace(&mut self, cmd: &str) -> R<bool> {
        if self.sc() != b'{' {
            p!(self, "\"{\" is missing in command: ", cmd);
            self.bst_err_print_and_look_for_blank_line()?;
            return Ok(false);
        }
        self.buf_ptr2 += 1;
        Ok(true)
    }

    /// `bst_get_and_check_right_brace` (§168).
    fn right_brace(&mut self, cmd: &str) -> R<bool> {
        if self.sc() != b'}' {
            p!(self, "\"}\" is missing in command: ", cmd);
            self.bst_err_print_and_look_for_blank_line()?;
            return Ok(false);
        }
        self.buf_ptr2 += 1;
        Ok(true)
    }

    /// `bst_identifier_scan` (§166).
    fn ident_scan(&mut self, cmd: &str) -> R<bool> {
        self.scan_identifier(b'}', b'%', b'%');
        match self.scan_result {
            ScanResult::WhiteAdjacent | ScanResult::SpecifiedCharAdjacent => Ok(true),
            r => {
                let c = self.sc();
                if r == ScanResult::IdNull {
                    p!(self, "\"", crate::engine::Ch(c), "\" begins identifier, command: ");
                } else {
                    p!(self, "\"", crate::engine::Ch(c), "\" immediately follows identifier, command: ");
                }
                p!(self, cmd);
                self.bst_err_print_and_look_for_blank_line()?;
                Ok(false)
            }
        }
    }

    pub(crate) fn print_fn_class(&mut self, id: usize) {
        let s = match self.fns[id].class {
            FnClass::BuiltIn(_) => "built-in",
            FnClass::Wiz(_) => "wizard-defined",
            FnClass::IntLit(_) => "integer-literal",
            FnClass::StrLit(_) => "string-literal",
            FnClass::Field(_) => "field",
            FnClass::IntEntry(_) => "integer-entry-variable",
            FnClass::StrEntry(_) => "string-entry-variable",
            FnClass::IntGlobal(_) => "integer-global-variable",
            FnClass::StrGlobal(_) => "string-global-variable",
        };
        p!(self, s);
    }

    /// `check_for_already_seen_function` (§169); `Ok(true)` means "return".
    fn already_seen(&mut self, id: usize, found: bool) -> R<bool> {
        if found {
            let name = self.fns[id].name.clone();
            p!(self, name, " is already a type \"");
            self.print_fn_class(id);
            pln!(self, "\" function name");
            self.bst_err_print_and_look_for_blank_line()?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Lower-cases the current token and inserts it as a `.bst` function.
    fn insert_token_fn(&mut self) -> (usize, bool) {
        self.lower_buffer_token();
        let tok = self.token();
        self.lookup_insert_fn(&tok)
    }

    /// §154.
    fn get_bst_command_and_process(&mut self) -> R<()> {
        if !self.scan_alpha() {
            let c = self.sc();
            p!(self, "\"", crate::engine::Ch(c), "\" can't start a style-file command");
            return self.bst_err_print_and_look_for_blank_line();
        }
        self.lower_buffer_token();
        let tok = self.token();
        match &tok[..] {
            b"entry" => self.bst_entry_command(),
            b"execute" => self.bst_execute_command(),
            b"function" => self.bst_function_command(),
            b"integers" => self.bst_integers_command(),
            b"iterate" => self.bst_iterate_command(),
            b"macro" => self.bst_macro_command(),
            b"read" => self.bst_read_command(),
            b"reverse" => self.bst_reverse_command(),
            b"sort" => self.bst_sort_command(),
            b"strings" => self.bst_strings_command(),
            _ => {
                self.print_token();
                p!(self, " is an illegal style-file command");
                self.bst_err_print_and_look_for_blank_line()
            }
        }
    }

    /// Scans `{ id id ... }` calling `insert` for each identifier
    /// (§§171, 173, 175, 201, 215). `Ok(false)` means "return".
    fn scan_id_list(
        &mut self,
        cmd: &str,
        insert: fn(&mut Self, usize, bool) -> R<bool>,
    ) -> R<bool> {
        if !self.left_brace(cmd)? {
            return Ok(false);
        }
        if !self.eat_check(cmd)? {
            return Ok(false);
        }
        while self.sc() != b'}' {
            if !self.ident_scan(cmd)? {
                return Ok(false);
            }
            let (id, found) = self.insert_token_fn();
            if self.already_seen(id, found)? {
                return Ok(false);
            }
            if !insert(self, id, found)? {
                return Ok(false);
            }
            if !self.eat_check(cmd)? {
                return Ok(false);
            }
        }
        self.buf_ptr2 += 1;
        Ok(true)
    }

    /// §170.
    fn bst_entry_command(&mut self) -> R<()> {
        if self.entry_seen {
            p!(self, "Illegal, another entry command");
            return self.bst_err_print_and_look_for_blank_line();
        }
        self.entry_seen = true;
        if !self.eat_check("entry")? {
            return Ok(());
        }
        if !self.scan_id_list("entry", |e, id, _| {
            e.fns[id].class = FnClass::Field(e.num_fields);
            e.num_fields += 1;
            Ok(true)
        })? {
            return Ok(());
        }
        if !self.eat_check("entry")? {
            return Ok(());
        }
        if self.num_fields == self.num_pre_defined_fields {
            p!(self, "Warning--I didn't find any fields");
            self.bst_warn_print();
        }
        if !self.scan_id_list("entry", |e, id, _| {
            e.fns[id].class = FnClass::IntEntry(e.num_ent_ints);
            e.num_ent_ints += 1;
            Ok(true)
        })? {
            return Ok(());
        }
        if !self.eat_check("entry")? {
            return Ok(());
        }
        self.scan_id_list("entry", |e, id, _| {
            e.fns[id].class = FnClass::StrEntry(e.num_ent_strs);
            e.num_ent_strs += 1;
            Ok(true)
        })?;
        Ok(())
    }

    /// §177: `Ok(Some(fn))` when the token names a built-in or wizard function.
    fn bad_argument_token(&mut self) -> R<Option<usize>> {
        self.lower_buffer_token();
        let tok = self.token();
        match self.fn_map.get(&tok).copied() {
            None => {
                self.print_token();
                p!(self, " is an unknown function");
                self.bst_err_print_and_look_for_blank_line()?;
                Ok(None)
            }
            Some(id) => match self.fns[id].class {
                FnClass::BuiltIn(_) | FnClass::Wiz(_) => Ok(Some(id)),
                _ => {
                    self.print_token();
                    p!(self, " has bad function type ");
                    self.print_fn_class(id);
                    self.bst_err_print_and_look_for_blank_line()?;
                    Ok(None)
                }
            },
        }
    }

    /// The common shape of `execute`, `iterate` and `reverse`
    /// (§§178, 203, 212).
    fn scan_fn_argument(&mut self, cmd: &str) -> R<Option<usize>> {
        let what = match cmd {
            "execute" => "Illegal, execute command before read command",
            "iterate" => "Illegal, iterate command before read command",
            _ => "Illegal, reverse command before read command",
        };
        if !self.read_seen {
            p!(self, what);
            self.bst_err_print_and_look_for_blank_line()?;
            return Ok(None);
        }
        if !self.eat_check(cmd)? || !self.left_brace(cmd)? || !self.eat_check(cmd)? {
            return Ok(None);
        }
        if !self.ident_scan(cmd)? {
            return Ok(None);
        }
        let id = match self.bad_argument_token()? {
            Some(id) => id,
            None => return Ok(None),
        };
        if !self.eat_check(cmd)? || !self.right_brace(cmd)? {
            return Ok(None);
        }
        Ok(Some(id))
    }

    fn bst_execute_command(&mut self) -> R<()> {
        if let Some(id) = self.scan_fn_argument("execute")? {
            // §296
            self.init_command_execution();
            self.mess_with_entries = false;
            self.execute_fn(id)?;
            self.check_command_execution()?;
        }
        Ok(())
    }

    fn bst_iterate_command(&mut self) -> R<()> {
        if let Some(id) = self.scan_fn_argument("iterate")? {
            // §297
            self.init_command_execution();
            self.mess_with_entries = true;
            let mut sort_cite_ptr = 0;
            while sort_cite_ptr < self.num_cites {
                self.cite_ptr = self.sorted_cites[sort_cite_ptr];
                self.execute_fn(id)?;
                self.check_command_execution()?;
                sort_cite_ptr += 1;
            }
        }
        Ok(())
    }

    fn bst_reverse_command(&mut self) -> R<()> {
        if let Some(id) = self.scan_fn_argument("reverse")? {
            // §298
            self.init_command_execution();
            self.mess_with_entries = true;
            let mut sort_cite_ptr = self.num_cites;
            while sort_cite_ptr > 0 {
                sort_cite_ptr -= 1;
                self.cite_ptr = self.sorted_cites[sort_cite_ptr];
                self.execute_fn(id)?;
                self.check_command_execution()?;
            }
        }
        Ok(())
    }

    /// §180.
    fn bst_function_command(&mut self) -> R<()> {
        if !self.eat_check("function")? {
            return Ok(());
        }
        // §181
        if !self.left_brace("function")?
            || !self.eat_check("function")?
            || !self.ident_scan("function")?
        {
            return Ok(());
        }
        // §182
        let (id, found) = self.insert_token_fn();
        self.wiz_loc = id;
        if self.already_seen(id, found)? {
            return Ok(());
        }
        self.fns[id].class = FnClass::Wiz(0);
        if &self.fns[id].name[..] == b"default.type" {
            self.b_default = id;
        }
        if !self.eat_check("function")?
            || !self.right_brace("function")?
            || !self.eat_check("function")?
            || !self.left_brace("function")?
        {
            return Ok(());
        }
        self.scan_fn_def(id)
    }

    /// `skip_token_print` (§183).
    fn skip_token_print(&mut self) {
        p!(self, "-");
        self.bst_ln_num_print();
        self.mark_error();
        self.scan2_white(b'}', b'%');
    }

    /// §187.
    fn scan_fn_def(&mut self, fn_hash_loc: usize) -> R<()> {
        let mut single: Vec<WizItem> = Vec::new();
        if !self.eat_check("function")? {
            return Ok(());
        }
        while self.sc() != b'}' {
            // §189
            match self.sc() {
                b'#' => {
                    // §190
                    self.buf_ptr2 += 1;
                    if !self.scan_integer() {
                        p!(self, "Illegal integer in integer literal");
                        self.skip_token_print();
                    } else {
                        let tok = self.token();
                        self.intern(&tok);
                        let id = match self.int_lits.get(&tok) {
                            Some(&id) => id,
                            None => {
                                let id = self.fns.len();
                                self.fns.push(crate::engine::Func {
                                    name: Rc::from(&tok[..]),
                                    class: FnClass::IntLit(self.token_value),
                                });
                                self.int_lits.insert(tok, id);
                                id
                            }
                        };
                        if self.illegal_after_literal() {
                            self.skip_illegal_stuff_after_token_print();
                        } else {
                            single.push(WizItem::Call(id));
                        }
                    }
                }
                b'"' => {
                    // §191
                    self.buf_ptr2 += 1;
                    if !self.scan1(b'"') {
                        p!(self, "No `\"' to end string literal");
                        self.skip_token_print();
                    } else {
                        let tok = self.token();
                        let id = self.str_literal(&tok);
                        self.buf_ptr2 += 1;
                        if self.illegal_after_literal() {
                            self.skip_illegal_stuff_after_token_print();
                        } else {
                            single.push(WizItem::Call(id));
                        }
                    }
                }
                b'\'' => {
                    // §192
                    self.buf_ptr2 += 1;
                    self.scan2_white(b'}', b'%');
                    self.lower_buffer_token();
                    let tok = self.token();
                    match self.fn_map.get(&tok).copied() {
                        None => self.skp_token_unknown_function_print(),
                        Some(id) if id == self.wiz_loc => self.print_recursion_illegal(),
                        Some(id) => {
                            single.push(WizItem::Quote);
                            single.push(WizItem::Call(id));
                        }
                    }
                }
                b'{' => {
                    // §194
                    let name = format!("'{}", self.impl_fn_num);
                    let name = name.as_bytes().to_vec();
                    self.copy_to_ex_buf(0, &name);
                    let (id, found) = self.lookup_insert_fn(&name);
                    if found {
                        return Err(self.confusion("Already encountered implicit function"));
                    }
                    self.impl_fn_num += 1;
                    self.fns[id].class = FnClass::Wiz(0);
                    single.push(WizItem::Quote);
                    single.push(WizItem::Call(id));
                    self.buf_ptr2 += 1;
                    self.scan_fn_def(id)?;
                }
                _ => {
                    // §199
                    self.scan2_white(b'}', b'%');
                    self.lower_buffer_token();
                    let tok = self.token();
                    match self.fn_map.get(&tok).copied() {
                        None => self.skp_token_unknown_function_print(),
                        Some(id) if id == self.wiz_loc => self.print_recursion_illegal(),
                        Some(id) => single.push(WizItem::Call(id)),
                    }
                }
            }
            // next_token:
            if !self.eat_check("function")? {
                return Ok(());
            }
        }
        // §200
        single.push(WizItem::End);
        self.fns[fn_hash_loc].class = FnClass::Wiz(self.wiz_functions.len());
        self.wiz_functions.extend_from_slice(&single);
        self.buf_ptr2 += 1;
        Ok(())
    }

    fn str_literal(&mut self, tok: &[u8]) -> usize {
        self.intern(tok);
        match self.str_lits.get(tok) {
            Some(&id) => id,
            None => {
                let id = self.fns.len();
                let s: crate::engine::Str = Rc::from(tok);
                self.fns.push(crate::engine::Func { name: s.clone(), class: FnClass::StrLit(s) });
                self.str_lits.insert(tok.to_vec(), id);
                id
            }
        }
    }

    fn illegal_after_literal(&self) -> bool {
        let c = self.sc();
        !white(c) && self.buf_ptr2 < self.last && c != b'}' && c != b'%'
    }

    fn skip_illegal_stuff_after_token_print(&mut self) {
        let c = self.sc();
        p!(self, "\"", crate::engine::Ch(c), "\" can't follow a literal");
        self.skip_token_print();
    }

    fn skp_token_unknown_function_print(&mut self) {
        self.print_token();
        p!(self, " is an unknown function");
        self.skip_token_print();
    }

    fn print_recursion_illegal(&mut self) {
        pln!(self, "Curse you, wizard, before you recurse me:");
        p!(self, "function ");
        self.print_token();
        pln!(self, " is illegal in its own definition");
        self.skip_token_print();
    }

    /// §201.
    fn bst_integers_command(&mut self) -> R<()> {
        if !self.eat_check("integers")? {
            return Ok(());
        }
        self.scan_id_list("integers", |e, id, _| {
            e.fns[id].class = FnClass::IntGlobal(0);
            Ok(true)
        })?;
        Ok(())
    }

    /// §215.
    fn bst_strings_command(&mut self) -> R<()> {
        if !self.eat_check("strings")? {
            return Ok(());
        }
        self.scan_id_list("strings", |e, id, _| {
            e.fns[id].class = FnClass::StrGlobal(e.globs.len());
            e.globs.push(Glob::Buf(Vec::new()));
            Ok(true)
        })?;
        Ok(())
    }

    /// §205.
    fn bst_macro_command(&mut self) -> R<()> {
        if self.read_seen {
            p!(self, "Illegal, macro command after read command");
            return self.bst_err_print_and_look_for_blank_line();
        }
        if !self.eat_check("macro")?
            || !self.left_brace("macro")?
            || !self.eat_check("macro")?
            || !self.ident_scan("macro")?
        {
            return Ok(());
        }
        // §207
        self.lower_buffer_token();
        let name = self.token();
        self.intern(&name);
        if self.macros.contains_key(&name) {
            self.print_token();
            p!(self, " is already defined as a macro");
            return self.bst_err_print_and_look_for_blank_line();
        }
        self.macros.insert(name.clone(), Rc::from(&name[..]));
        if !self.eat_check("macro")?
            || !self.right_brace("macro")?
            || !self.eat_check("macro")?
            || !self.left_brace("macro")?
            || !self.eat_check("macro")?
        {
            return Ok(());
        }
        // §208
        if self.sc() != b'"' {
            p!(self, "A macro definition must be \"-delimited");
            return self.bst_err_print_and_look_for_blank_line();
        }
        // §209
        self.buf_ptr2 += 1;
        if !self.scan1(b'"') {
            p!(self, "There's no `\"' to end macro definition");
            return self.bst_err_print_and_look_for_blank_line();
        }
        let def = self.token();
        self.str_literal(&def);
        self.macros.insert(name, Rc::from(&def[..]));
        self.buf_ptr2 += 1;
        if !self.eat_check("macro")? || !self.right_brace("macro")? {
            return Ok(());
        }
        Ok(())
    }

    /// §211.
    fn bst_read_command(&mut self) -> R<()> {
        if self.read_seen {
            p!(self, "Illegal, another read command");
            return self.bst_err_print_and_look_for_blank_line();
        }
        self.read_seen = true;
        if !self.entry_seen {
            p!(self, "Illegal, read command before entry command");
            return self.bst_err_print_and_look_for_blank_line();
        }
        let sv_ptr1 = self.buf_ptr2;
        let sv_ptr2 = self.last;
        if self.sv_buffer.len() < self.buffer.len() {
            self.sv_buffer.resize(self.buffer.len(), 0);
        }
        self.sv_buffer[sv_ptr1..sv_ptr2].copy_from_slice(&self.buffer[sv_ptr1..sv_ptr2]);
        self.read_bib_files()?;
        self.buf_ptr2 = sv_ptr1;
        self.last = sv_ptr2;
        if self.buffer.len() <= sv_ptr2 {
            self.buffer.resize(sv_ptr2 + 20000, 0);
        }
        self.buffer[sv_ptr1..sv_ptr2].copy_from_slice(&self.sv_buffer[sv_ptr1..sv_ptr2]);
        Ok(())
    }

    /// §214.
    fn bst_sort_command(&mut self) -> R<()> {
        if !self.read_seen {
            p!(self, "Illegal, sort command before read command");
            return self.bst_err_print_and_look_for_blank_line();
        }
        // §§299–306: `less_than` is a strict total order (ties are broken
        // by cite number), so any correct sort yields BibTeX's permutation.
        let n = self.num_cites;
        if n > 1 {
            let ns = self.num_ent_strs;
            let sk = self.sort_key_num;
            let mut keys: Vec<(Vec<u8>, usize)> = self.sorted_cites[..n]
                .iter()
                .map(|&c| {
                    let s = &self.entry_strs[c * ns + sk];
                    (s.clone(), c)
                })
                .collect();
            keys.sort();
            for (i, (_, c)) in keys.into_iter().enumerate() {
                self.sorted_cites[i] = c;
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
fn _unused(c: u8) -> u8 {
    let mut v = [c];
    lower(&mut v);
    lex(v[0])
}
