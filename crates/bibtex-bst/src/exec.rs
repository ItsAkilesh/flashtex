//! Executing the style file: the literal stack, output and the built-in
//! functions (§§290–454, except `format.name$`, see `names.rs`).

use std::rc::Rc;

use crate::chars::{char_width, control_seq, lex, lower, upper, white, Cs, ALPHA, NUMERIC, SEP};
use crate::chars::{AE_WIDTH, OE_WIDTH, SS_WIDTH, UPPER_AE_WIDTH, UPPER_OE_WIDTH};
use crate::engine::{p, pln, Engine, FnClass, Glob, Lit, Str, TypeRef, WizItem, R};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Want {
    Int,
    Str,
    Fn,
}

const MIN_PRINT_LINE: usize = 3;

fn null() -> Lit {
    Lit::Str(Rc::from(&b""[..]), false)
}

fn temp(v: &[u8]) -> Lit {
    Lit::Str(Rc::from(v), true)
}

/// `int_to_ASCII` (§198) with C integer semantics.
pub(crate) fn int_to_ascii(mut n: i32) -> Vec<u8> {
    let mut out = Vec::new();
    if n < 0 {
        out.push(b'-');
        n = n.wrapping_neg();
    }
    let start = out.len();
    loop {
        out.push((48 + (n % 10)) as u8);
        n /= 10;
        if n == 0 {
            break;
        }
    }
    out[start..].reverse();
    out
}

impl<'a> Engine<'a> {
    pub(crate) fn init_command_execution(&mut self) {
        self.stack.clear();
    }

    /// §317.
    pub(crate) fn check_command_execution(&mut self) -> R<()> {
        if !self.stack.is_empty() {
            pln!(self, "ptr=", self.stack.len(), ", stack=");
            self.pop_whole_stack();
            p!(self, "---the literal stack isn't empty");
            self.bst_ex_warn_print();
        }
        Ok(())
    }

    /// §293.
    pub(crate) fn bst_ex_warn_print(&mut self) {
        if self.mess_with_entries {
            let key = self.cite_list[self.cite_ptr].clone();
            p!(self, " for entry ", key);
        }
        self.log.push(b'\n');
        p!(self, "while executing-");
        self.bst_ln_num_print_pub();
        self.mark_error();
    }

    /// §294.
    pub(crate) fn bst_mild_ex_warn_print(&mut self) {
        if self.mess_with_entries {
            let key = self.cite_list[self.cite_ptr].clone();
            p!(self, " for entry ", key);
        }
        self.log.push(b'\n');
        p!(self, "while executing");
        self.bst_warn_print();
    }

    pub(crate) fn pop_lit(&mut self) -> Lit {
        self.pop()
    }

    pub(crate) fn push_lit(&mut self, l: Lit) {
        self.push(l)
    }

    pub(crate) fn wrong(&mut self, l: &Lit, want: Want) {
        self.print_wrong_stk_lit(l, want)
    }

    fn cant_mess(&mut self) {
        p!(self, "You can't mess with entries here");
        self.bst_ex_warn_print();
    }

    fn push(&mut self, l: Lit) {
        self.check_lit_stk_overflow(self.stack.len());
        self.stack.push(l);
    }

    /// §309.
    fn pop(&mut self) -> Lit {
        match self.stack.pop() {
            Some(l) => l,
            None => {
                p!(self, "You can't pop an empty literal stack");
                self.bst_ex_warn_print();
                Lit::Empty
            }
        }
    }

    /// §311.
    fn print_stk_lit(&mut self, l: &Lit) {
        match l {
            Lit::Int(n) => p!(self, *n, " is an integer literal"),
            Lit::Str(s, _) => p!(self, "\"", s, "\" is a string literal"),
            Lit::Fn(f) => {
                let name = self.fns[*f].name.clone();
                p!(self, "`", name, "' is a function literal");
            }
            Lit::Missing(s) => p!(self, "`", s, "' is a missing field"),
            Lit::Empty => {}
        }
    }

    /// §312.
    fn print_wrong_stk_lit(&mut self, l: &Lit, want: Want) {
        if matches!(l, Lit::Empty) {
            return;
        }
        self.print_stk_lit(l);
        match want {
            Want::Int => p!(self, ", not an integer,"),
            Want::Str => p!(self, ", not a string,"),
            Want::Fn => p!(self, ", not a function,"),
        }
        self.bst_ex_warn_print();
    }

    /// §313.
    fn print_lit(&mut self, l: &Lit) {
        match l {
            Lit::Int(n) => pln!(self, *n),
            Lit::Str(s, _) | Lit::Missing(s) => pln!(self, s),
            Lit::Fn(f) => {
                let name = self.fns[*f].name.clone();
                pln!(self, name);
            }
            Lit::Empty => {}
        }
    }

    /// §314.
    fn pop_top_and_print(&mut self) {
        let l = self.pop();
        if matches!(l, Lit::Empty) {
            pln!(self, "Empty literal");
        } else {
            self.print_lit(&l);
        }
    }

    /// §315.
    fn pop_whole_stack(&mut self) {
        while !self.stack.is_empty() {
            self.pop_top_and_print();
        }
    }

    // ------------------------------------------------------------ output (§§321–324)

    /// §321.
    pub(crate) fn output_bbl_line(&mut self) {
        if self.out_buf_length != 0 {
            while self.out_buf_length > 0 && white(self.out_buf[self.out_buf_length - 1]) {
                self.out_buf_length -= 1;
            }
            if self.out_buf_length == 0 {
                return;
            }
            self.bbl.extend_from_slice(&self.out_buf[..self.out_buf_length]);
        }
        self.bbl.push(b'\n');
        self.bbl_line_num += 1;
        self.out_buf_length = 0;
    }

    /// §322.
    fn add_out_pool(&mut self, s: &[u8]) {
        let need = self.out_buf_length + s.len() + 2;
        if self.out_buf.len() < need {
            self.out_buf.resize(need + 20000, 0);
        }
        self.out_buf[self.out_buf_length..self.out_buf_length + s.len()].copy_from_slice(s);
        self.out_buf_length += s.len();
        let max = self.opts.max_print_line;
        let mut unbreakable_tail = false;
        while self.out_buf_length > max && !unbreakable_tail {
            // §323
            let end_ptr = self.out_buf_length;
            let mut ptr = max;
            let mut break_pt_found = false;
            while !white(self.out_buf[ptr]) && ptr >= MIN_PRINT_LINE {
                ptr -= 1;
            }
            if ptr == MIN_PRINT_LINE - 1 {
                // §324
                ptr = max + 1;
                while ptr < end_ptr && !white(self.out_buf[ptr]) {
                    ptr += 1;
                }
                if ptr == end_ptr {
                    unbreakable_tail = true;
                } else {
                    break_pt_found = true;
                    while ptr + 1 < end_ptr && white(self.out_buf[ptr + 1]) {
                        ptr += 1;
                    }
                }
            } else {
                break_pt_found = true;
            }
            if break_pt_found {
                self.out_buf_length = ptr;
                let break_ptr = ptr + 1;
                self.output_bbl_line();
                self.out_buf[0] = b' ';
                self.out_buf[1] = b' ';
                self.out_buf.copy_within(break_ptr..end_ptr, 2);
                self.out_buf_length = end_ptr - break_ptr + 2;
            }
        }
    }

    // ------------------------------------------------------------ execute_fn (§325)

    pub(crate) fn execute_fn(&mut self, id: usize) -> R<()> {
        let class = self.fns[id].class.clone();
        match class {
            FnClass::BuiltIn(n) => {
                self.exec_count[n] += 1;
                self.execute_builtin(n)?;
            }
            FnClass::Wiz(start) => {
                let mut ptr = start;
                loop {
                    match self.wiz_functions[ptr] {
                        WizItem::End => break,
                        WizItem::Quote => {
                            ptr += 1;
                            if let WizItem::Call(f) = self.wiz_functions[ptr] {
                                self.push(Lit::Fn(f));
                            }
                        }
                        WizItem::Call(f) => self.execute_fn(f)?,
                    }
                    ptr += 1;
                }
            }
            FnClass::IntLit(v) | FnClass::IntGlobal(v) => self.push(Lit::Int(v)),
            FnClass::StrLit(s) => self.push(Lit::Str(s, false)),
            FnClass::Field(k) => {
                if !self.mess_with_entries {
                    self.cant_mess();
                } else {
                    match self.field_info[self.cite_ptr * self.num_fields + k].clone() {
                        None => {
                            let name = self.fns[id].name.clone();
                            self.push(Lit::Missing(name));
                        }
                        Some(v) => self.push(Lit::Str(v, false)),
                    }
                }
            }
            FnClass::IntEntry(k) => {
                if !self.mess_with_entries {
                    self.cant_mess();
                } else {
                    let v = self.entry_ints[self.cite_ptr * self.num_ent_ints + k];
                    self.push(Lit::Int(v));
                }
            }
            FnClass::StrEntry(k) => {
                if !self.mess_with_entries {
                    self.cant_mess();
                } else {
                    let v = self.entry_strs[self.cite_ptr * self.num_ent_strs + k].clone();
                    self.push(temp(&v));
                }
            }
            FnClass::StrGlobal(k) => match &self.globs[k] {
                Glob::Static(s) => {
                    let s = s.clone();
                    self.push(Lit::Str(s, false));
                }
                Glob::Buf(b) => {
                    let l = temp(b);
                    self.push(l);
                }
            },
        }
        Ok(())
    }

    fn execute_builtin(&mut self, n: usize) -> R<()> {
        match n {
            0 => self.x_equals(),
            1 | 2 | 3 | 4 => self.x_int_op(n),
            5 => self.x_concatenate(),
            6 => self.x_gets(),
            7 => self.x_add_period(),
            8 => {
                // §363
                if !self.mess_with_entries {
                    self.cant_mess();
                } else {
                    match self.type_list[self.cite_ptr] {
                        TypeRef::Undefined => self.execute_fn(self.b_default)?,
                        TypeRef::Empty => {}
                        TypeRef::Fn(f) => self.execute_fn(f)?,
                    }
                }
            }
            9 => self.x_change_case(),
            10 => self.x_chr_to_int(),
            11 => {
                // §378
                if !self.mess_with_entries {
                    self.cant_mess();
                } else {
                    let s = self.cite_list[self.cite_ptr].clone();
                    self.push(Lit::Str(s, false));
                }
            }
            12 => {
                // §379
                let l = self.pop();
                self.push(l.clone());
                self.push(l);
            }
            13 => self.x_empty(),
            14 => self.x_format_name()?,
            15 => {
                // §421
                let l1 = self.pop();
                let l2 = self.pop();
                let l3 = self.pop();
                match (&l1, &l2, &l3) {
                    (Lit::Fn(f1), Lit::Fn(f2), Lit::Int(v)) => {
                        if *v > 0 {
                            self.execute_fn(*f2)?;
                        } else {
                            self.execute_fn(*f1)?;
                        }
                    }
                    (Lit::Fn(_), Lit::Fn(_), _) => self.print_wrong_stk_lit(&l3, Want::Int),
                    (Lit::Fn(_), _, _) => self.print_wrong_stk_lit(&l2, Want::Fn),
                    _ => self.print_wrong_stk_lit(&l1, Want::Fn),
                }
            }
            16 => self.x_int_to_chr(),
            17 => {
                // §423
                let l = self.pop();
                match l {
                    Lit::Int(v) => {
                        let s = int_to_ascii(v);
                        self.push(temp(&s));
                    }
                    _ => {
                        self.print_wrong_stk_lit(&l, Want::Int);
                        self.push(null());
                    }
                }
            }
            18 => self.x_missing(),
            19 => self.output_bbl_line(),
            20 => self.x_num_names(),
            21 => {
                self.pop();
            }
            22 => {
                // §429
                let mut v = Vec::new();
                for i in 0..self.num_preamble_strings {
                    v.extend_from_slice(&self.s_preamble[i]);
                }
                self.push(temp(&v));
            }
            23 => self.x_purify(),
            24 => self.push(temp(b"\"")),
            25 => {}
            26 => self.pop_whole_stack(),
            27 => self.x_substring(),
            28 => {
                // §439
                let l1 = self.pop();
                let l2 = self.pop();
                self.push(l1);
                self.push(l2);
            }
            29 => self.x_text_length(),
            30 => self.x_text_prefix(),
            31 => self.pop_top_and_print(),
            32 => {
                // §447
                if !self.mess_with_entries {
                    self.cant_mess();
                } else {
                    match self.type_list[self.cite_ptr] {
                        TypeRef::Fn(f) => {
                            let s = self.fns[f].name.clone();
                            self.push(Lit::Str(s, false));
                        }
                        _ => self.push(null()),
                    }
                }
            }
            33 => {
                // §448
                let l = self.pop();
                if let Lit::Str(..) = l {
                    p!(self, "Warning--");
                    self.print_lit(&l);
                    self.mark_warning();
                } else {
                    self.print_wrong_stk_lit(&l, Want::Str);
                }
            }
            34 => {
                // §449
                let r1 = self.pop();
                let r2 = self.pop();
                match (&r1, &r2) {
                    (Lit::Fn(body), Lit::Fn(test)) => loop {
                        self.execute_fn(*test)?;
                        let l = self.pop();
                        match l {
                            Lit::Int(v) if v > 0 => self.execute_fn(*body)?,
                            Lit::Int(_) => break,
                            _ => {
                                self.print_wrong_stk_lit(&l, Want::Int);
                                break;
                            }
                        }
                    },
                    (Lit::Fn(_), _) => self.print_wrong_stk_lit(&r2, Want::Fn),
                    _ => self.print_wrong_stk_lit(&r1, Want::Fn),
                }
            }
            35 => self.x_width(),
            36 => {
                // §454
                let l = self.pop();
                match l {
                    Lit::Str(s, _) => self.add_out_pool(&s),
                    _ => self.print_wrong_stk_lit(&l, Want::Str),
                }
            }
            _ => return Err(self.confusion("Unknown built-in function")),
        }
        Ok(())
    }

    /// §345.
    fn x_equals(&mut self) {
        let l1 = self.pop();
        let l2 = self.pop();
        let r = match (&l1, &l2) {
            (Lit::Int(a), Lit::Int(b)) => (a == b) as i32,
            (Lit::Str(a, _), Lit::Str(b, _)) => (a[..] == b[..]) as i32,
            (Lit::Empty, Lit::Empty) => 0,
            (Lit::Fn(_), Lit::Fn(_)) | (Lit::Missing(_), Lit::Missing(_)) => {
                self.print_stk_lit(&l1);
                p!(self, ", not an integer or a string,");
                self.bst_ex_warn_print();
                0
            }
            (Lit::Empty, _) | (_, Lit::Empty) => 0,
            _ => {
                self.print_stk_lit(&l1);
                p!(self, ", ");
                self.print_stk_lit(&l2);
                self.log.push(b'\n');
                p!(self, "---they aren't the same literal types");
                self.bst_ex_warn_print();
                0
            }
        };
        self.push(Lit::Int(r));
    }

    /// §§346–349.
    fn x_int_op(&mut self, n: usize) {
        let l1 = self.pop();
        let l2 = self.pop();
        match (&l1, &l2) {
            (Lit::Int(a), Lit::Int(b)) => {
                let r = match n {
                    1 => (b > a) as i32,
                    2 => (b < a) as i32,
                    3 => b.wrapping_add(*a),
                    _ => b.wrapping_sub(*a),
                };
                self.push(Lit::Int(r));
            }
            (Lit::Int(_), _) => {
                self.print_wrong_stk_lit(&l2, Want::Int);
                self.push(Lit::Int(0));
            }
            _ => {
                self.print_wrong_stk_lit(&l1, Want::Int);
                self.push(Lit::Int(0));
            }
        }
    }

    /// §§350–353.
    fn x_concatenate(&mut self) {
        let l1 = self.pop();
        let l2 = self.pop();
        match (&l1, &l2) {
            (Lit::Str(s1, t1), Lit::Str(s2, t2)) => {
                if s1.is_empty() {
                    self.push(Lit::Str(s2.clone(), *t2));
                } else if s2.is_empty() {
                    self.push(Lit::Str(s1.clone(), *t1));
                } else {
                    let mut v = Vec::with_capacity(s1.len() + s2.len());
                    v.extend_from_slice(s2);
                    v.extend_from_slice(s1);
                    self.push(temp(&v));
                }
            }
            (Lit::Str(..), _) => {
                self.print_wrong_stk_lit(&l2, Want::Str);
                self.push(null());
            }
            _ => {
                self.print_wrong_stk_lit(&l1, Want::Str);
                self.push(null());
            }
        }
    }

    /// `bst_string_size_exceeded` (§356).
    fn string_size_exceeded(&mut self, size: usize, what: &str) {
        p!(self, "Warning--you've exceeded ", size, what, "-string-size,");
        self.bst_mild_ex_warn_print();
        pln!(self, "*Please notify the bibstyle designer*");
    }

    /// §§354–359.
    fn x_gets(&mut self) {
        let l1 = self.pop();
        let l2 = self.pop();
        let f = match l1 {
            Lit::Fn(f) => f,
            _ => {
                self.print_wrong_stk_lit(&l1, Want::Fn);
                return;
            }
        };
        let class = self.fns[f].class.clone();
        if !self.mess_with_entries && matches!(class, FnClass::StrEntry(_) | FnClass::IntEntry(_)) {
            self.cant_mess();
            return;
        }
        match class {
            FnClass::IntEntry(k) => match l2 {
                Lit::Int(v) => self.entry_ints[self.cite_ptr * self.num_ent_ints + k] = v,
                _ => self.print_wrong_stk_lit(&l2, Want::Int),
            },
            FnClass::StrEntry(k) => match l2 {
                Lit::Str(s, _) => {
                    let size = self.opts.ent_str_size;
                    let mut end = s.len();
                    if end > size {
                        self.string_size_exceeded(size, ", the entry");
                        end = size;
                    }
                    // `end_of_string` is code 127 (§216): reading stops there.
                    let bytes = &s[..end];
                    let bytes = match bytes.iter().position(|&c| c == 127) {
                        Some(i) => &bytes[..i],
                        None => bytes,
                    };
                    let idx = self.cite_ptr * self.num_ent_strs + k;
                    self.entry_strs[idx] = bytes.to_vec();
                }
                _ => self.print_wrong_stk_lit(&l2, Want::Str),
            },
            FnClass::IntGlobal(_) => match l2 {
                Lit::Int(v) => self.fns[f].class = FnClass::IntGlobal(v),
                _ => self.print_wrong_stk_lit(&l2, Want::Int),
            },
            FnClass::StrGlobal(k) => match l2 {
                Lit::Str(s, false) => self.globs[k] = Glob::Static(s),
                Lit::Str(s, true) => {
                    let size = self.opts.glob_str_size;
                    let mut end = s.len();
                    if end > size {
                        self.string_size_exceeded(size, ", the global");
                        end = size;
                    }
                    self.globs[k] = Glob::Buf(s[..end].to_vec());
                }
                _ => self.print_wrong_stk_lit(&l2, Want::Str),
            },
            _ => {
                p!(self, "You can't assign to type ");
                self.print_fn_class(f);
                p!(self, ", a nonvariable function class");
                self.bst_ex_warn_print();
            }
        }
    }

    /// §§360–362.
    fn x_add_period(&mut self) {
        let l = self.pop();
        match l {
            Lit::Str(ref s, t) => {
                if s.is_empty() {
                    self.push(null());
                    return;
                }
                let c = match s.iter().rposition(|&c| c != b'}') {
                    Some(i) => s[i],
                    None => s[0],
                };
                if c == b'.' || c == b'?' || c == b'!' {
                    self.push(Lit::Str(s.clone(), t));
                } else {
                    let mut v = s.to_vec();
                    v.push(b'.');
                    self.push(temp(&v));
                }
            }
            _ => {
                self.print_wrong_stk_lit(&l, Want::Str);
                self.push(null());
            }
        }
    }

    pub(crate) fn load_ex_buf(&mut self, s: &[u8]) {
        if self.ex_buf.len() < s.len() + 2 {
            self.ex_buf.resize(s.len() + 20000, 0);
        }
        self.ex_buf[..s.len()].copy_from_slice(s);
        self.ex_buf_length = s.len() as i64;
    }

    pub(crate) fn exb_set(&mut self, i: i64, c: u8) {
        if i < 0 {
            return;
        }
        let i = i as usize;
        if i + 1 >= self.ex_buf.len() {
            self.ex_buf.resize(i + 20000, 0);
        }
        self.ex_buf[i] = c;
    }

    fn ex_range_mut(&mut self, a: i64, b: i64) -> &mut [u8] {
        let a = a.max(0) as usize;
        let b = (b.max(0) as usize).max(a);
        if self.ex_buf.len() < b + 1 {
            self.ex_buf.resize(b + 20000, 0);
        }
        &mut self.ex_buf[a..b]
    }

    pub(crate) fn ex_string(&self) -> Vec<u8> {
        let n = self.ex_buf_length.max(0) as usize;
        self.ex_buf[..n].to_vec()
    }

    /// §§367–369.
    pub(crate) fn braces_unbalanced_complaint(&mut self, s: &[u8]) {
        p!(self, "Warning--\"", s, "\" isn't a brace-balanced string");
        self.bst_mild_ex_warn_print();
    }

    pub(crate) fn decr_brace_level(&mut self, s: &[u8]) {
        if self.brace_level == 0 {
            self.braces_unbalanced_complaint(s);
        } else {
            self.brace_level -= 1;
        }
    }

    pub(crate) fn check_brace_level(&mut self, s: &[u8]) {
        if self.brace_level > 0 {
            self.braces_unbalanced_complaint(s);
        }
    }

    /// §§364–376.
    fn x_change_case(&mut self) {
        let l1 = self.pop();
        let l2 = self.pop();
        let (spec, s) = match (&l1, &l2) {
            (Lit::Str(a, _), Lit::Str(b, _)) => (a.clone(), b.clone()),
            (Lit::Str(..), _) => {
                self.print_wrong_stk_lit(&l2, Want::Str);
                self.push(null());
                return;
            }
            _ => {
                self.print_wrong_stk_lit(&l1, Want::Str);
                self.push(null());
                return;
            }
        };
        // §366: 0 = title_lowers, 1 = all_lowers, 2 = all_uppers, 3 = bad
        let mut conv = match spec.first() {
            Some(b't') | Some(b'T') => 0,
            Some(b'l') | Some(b'L') => 1,
            Some(b'u') | Some(b'U') => 2,
            _ => 3,
        };
        if spec.len() != 1 || conv == 3 {
            conv = 3;
            p!(self, spec, " is an illegal case-conversion string");
            self.bst_ex_warn_print();
        }
        self.load_ex_buf(&s);
        // §370
        self.brace_level = 0;
        let mut ptr: i64 = 0;
        while ptr < self.ex_buf_length {
            let c = self.exb(ptr);
            if c == b'{' {
                self.brace_level += 1;
                let skip = self.brace_level != 1
                    || ptr + 4 > self.ex_buf_length
                    || self.exb(ptr + 1) != b'\\'
                    || (conv == 0 && (ptr == 0 || (self.prev_colon && white(self.exb(ptr - 1)))));
                if !skip {
                    ptr = self.convert_special_char(ptr, conv);
                }
                self.prev_colon = false;
            } else if c == b'}' {
                self.decr_brace_level(&s);
                self.prev_colon = false;
            } else if self.brace_level == 0 {
                // §376
                match conv {
                    0 => {
                        if ptr == 0 || (self.prev_colon && white(self.exb(ptr - 1))) {
                        } else {
                            lower(self.ex_range_mut(ptr, ptr + 1));
                        }
                        let c = self.exb(ptr);
                        if c == b':' {
                            self.prev_colon = true;
                        } else if !white(c) {
                            self.prev_colon = false;
                        }
                    }
                    1 => lower(self.ex_range_mut(ptr, ptr + 1)),
                    2 => upper(self.ex_range_mut(ptr, ptr + 1)),
                    _ => {}
                }
            }
            ptr += 1;
        }
        self.check_brace_level(&s);
        let out = self.ex_string();
        self.push(temp(&out));
    }

    /// §§371–375; returns the new `ex_buf_ptr`.
    fn convert_special_char(&mut self, mut ptr: i64, conv: u8) -> i64 {
        ptr += 1;
        while ptr < self.ex_buf_length && self.brace_level > 0 {
            ptr += 1;
            let mut xptr = ptr;
            while ptr < self.ex_buf_length && lex(self.exb(ptr)) == ALPHA {
                ptr += 1;
            }
            let name = self.ex_range_mut(xptr, ptr).to_vec();
            if let Some(cs) = control_seq(&name) {
                // §372
                match conv {
                    0 | 1 => {
                        if matches!(cs, Cs::LU | Cs::OU | Cs::OeU | Cs::AeU | Cs::AaU) {
                            lower(self.ex_range_mut(xptr, ptr));
                        }
                    }
                    2 => match cs {
                        Cs::L | Cs::O | Cs::Oe | Cs::Ae | Cs::Aa => {
                            upper(self.ex_range_mut(xptr, ptr));
                        }
                        Cs::I | Cs::J | Cs::Ss => {
                            // §374
                            upper(self.ex_range_mut(xptr, ptr));
                            while xptr < ptr {
                                let c = self.exb(xptr);
                                self.exb_set(xptr - 1, c);
                                xptr += 1;
                            }
                            xptr -= 1;
                            while ptr < self.ex_buf_length && white(self.exb(ptr)) {
                                ptr += 1;
                            }
                            let mut tmp = ptr;
                            while tmp < self.ex_buf_length {
                                let c = self.exb(tmp);
                                self.exb_set(tmp - (ptr - xptr), c);
                                tmp += 1;
                            }
                            self.ex_buf_length = tmp - (ptr - xptr);
                            ptr = xptr;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            xptr = ptr;
            while ptr < self.ex_buf_length && self.brace_level > 0 && self.exb(ptr) != b'\\' {
                let c = self.exb(ptr);
                if c == b'}' {
                    self.brace_level -= 1;
                } else if c == b'{' {
                    self.brace_level += 1;
                }
                ptr += 1;
            }
            // §375
            match conv {
                0 | 1 => lower(self.ex_range_mut(xptr, ptr)),
                2 => upper(self.ex_range_mut(xptr, ptr)),
                _ => {}
            }
        }
        ptr - 1
    }

    /// §377.
    fn x_chr_to_int(&mut self) {
        let l = self.pop();
        match l {
            Lit::Str(ref s, _) if s.len() == 1 => self.push(Lit::Int(s[0] as i32)),
            Lit::Str(ref s, _) => {
                let s = s.clone();
                p!(self, "\"", s, "\" isn't a single character");
                self.bst_ex_warn_print();
                self.push(Lit::Int(0));
            }
            _ => {
                self.print_wrong_stk_lit(&l, Want::Str);
                self.push(Lit::Int(0));
            }
        }
    }

    /// §§380–381.
    fn x_empty(&mut self) {
        let l = self.pop();
        match l {
            Lit::Str(ref s, _) => {
                let r = s.iter().all(|&c| white(c)) as i32;
                self.push(Lit::Int(r));
            }
            Lit::Missing(_) => self.push(Lit::Int(1)),
            Lit::Empty => self.push(Lit::Int(0)),
            _ => {
                self.print_stk_lit(&l);
                p!(self, ", not a string or missing field,");
                self.bst_ex_warn_print();
                self.push(Lit::Int(0));
            }
        }
    }

    /// §422.
    fn x_int_to_chr(&mut self) {
        let l = self.pop();
        match l {
            Lit::Int(v) if (0..=127).contains(&v) => self.push(temp(&[v as u8])),
            Lit::Int(v) => {
                p!(self, v, " isn't valid ASCII");
                self.bst_ex_warn_print();
                self.push(null());
            }
            _ => {
                self.print_wrong_stk_lit(&l, Want::Int);
                self.push(null());
            }
        }
    }

    /// §424.
    fn x_missing(&mut self) {
        let l = self.pop();
        if !self.mess_with_entries {
            self.cant_mess();
            return;
        }
        match l {
            Lit::Missing(_) => self.push(Lit::Int(1)),
            Lit::Str(..) => self.push(Lit::Int(0)),
            Lit::Empty => self.push(Lit::Int(0)),
            _ => {
                self.print_stk_lit(&l);
                p!(self, ", not a string or missing field,");
                self.bst_ex_warn_print();
                self.push(Lit::Int(0));
            }
        }
    }

    /// §§426–427.
    fn x_num_names(&mut self) {
        let l = self.pop();
        match l {
            Lit::Str(s, _) => {
                self.load_ex_buf(&s);
                self.ex_buf_ptr = 0;
                let mut num_names = 0;
                while self.ex_buf_ptr < self.ex_buf_length {
                    self.name_scan_for_and(&s);
                    num_names += 1;
                }
                self.push(Lit::Int(num_names));
            }
            _ => {
                self.print_wrong_stk_lit(&l, Want::Str);
                self.push(Lit::Int(0));
            }
        }
    }

    /// §384.
    pub(crate) fn name_scan_for_and(&mut self, s: &[u8]) {
        self.brace_level = 0;
        let mut preceding_white = false;
        let mut and_found = false;
        while !and_found && self.ex_buf_ptr < self.ex_buf_length {
            let c = self.exb(self.ex_buf_ptr);
            match c {
                b'a' | b'A' => {
                    self.ex_buf_ptr += 1;
                    if preceding_white {
                        // §386
                        let p0 = self.ex_buf_ptr;
                        if p0 <= self.ex_buf_length - 3
                            && (self.exb(p0) == b'n' || self.exb(p0) == b'N')
                            && (self.exb(p0 + 1) == b'd' || self.exb(p0 + 1) == b'D')
                            && white(self.exb(p0 + 2))
                        {
                            self.ex_buf_ptr += 2;
                            and_found = true;
                        }
                    }
                    preceding_white = false;
                }
                b'{' => {
                    self.brace_level += 1;
                    self.ex_buf_ptr += 1;
                    // §385
                    while self.brace_level > 0 && self.ex_buf_ptr < self.ex_buf_length {
                        let c = self.exb(self.ex_buf_ptr);
                        if c == b'}' {
                            self.brace_level -= 1;
                        } else if c == b'{' {
                            self.brace_level += 1;
                        }
                        self.ex_buf_ptr += 1;
                    }
                    preceding_white = false;
                }
                b'}' => {
                    self.decr_brace_level(s);
                    self.ex_buf_ptr += 1;
                    preceding_white = false;
                }
                _ => {
                    self.ex_buf_ptr += 1;
                    preceding_white = white(c);
                }
            }
        }
        self.check_brace_level(s);
    }

    /// §§430–433.
    fn x_purify(&mut self) {
        let l = self.pop();
        let s = match l {
            Lit::Str(s, _) => s,
            _ => {
                self.print_wrong_stk_lit(&l, Want::Str);
                self.push(null());
                return;
            }
        };
        self.load_ex_buf(&s);
        self.brace_level = 0;
        let mut xptr: i64 = 0;
        let mut ptr: i64 = 0;
        while ptr < self.ex_buf_length {
            let c = self.exb(ptr);
            match lex(c) {
                crate::chars::WHITE | SEP => {
                    self.exb_set(xptr, b' ');
                    xptr += 1;
                }
                ALPHA | NUMERIC => {
                    self.exb_set(xptr, c);
                    xptr += 1;
                }
                _ => {
                    if c == b'{' {
                        self.brace_level += 1;
                        if self.brace_level == 1
                            && ptr + 1 < self.ex_buf_length
                            && self.exb(ptr + 1) == b'\\'
                        {
                            // §432
                            ptr += 1;
                            while ptr < self.ex_buf_length && self.brace_level > 0 {
                                ptr += 1;
                                let yptr = ptr;
                                self.ex_buf_yptr = yptr;
                                while ptr < self.ex_buf_length && lex(self.exb(ptr)) == ALPHA {
                                    ptr += 1;
                                }
                                let name = self.ex_range_mut(yptr, ptr).to_vec();
                                if let Some(cs) = control_seq(&name) {
                                    // §433
                                    let c0 = self.exb(yptr);
                                    self.exb_set(xptr, c0);
                                    xptr += 1;
                                    if matches!(cs, Cs::Oe | Cs::OeU | Cs::Ae | Cs::AeU | Cs::Ss) {
                                        let c1 = self.exb(yptr + 1);
                                        self.exb_set(xptr, c1);
                                        xptr += 1;
                                    }
                                }
                                while ptr < self.ex_buf_length
                                    && self.brace_level > 0
                                    && self.exb(ptr) != b'\\'
                                {
                                    let c = self.exb(ptr);
                                    match lex(c) {
                                        ALPHA | NUMERIC => {
                                            self.exb_set(xptr, c);
                                            xptr += 1;
                                        }
                                        _ => {
                                            if c == b'}' {
                                                self.brace_level -= 1;
                                            } else if c == b'{' {
                                                self.brace_level += 1;
                                            }
                                        }
                                    }
                                    ptr += 1;
                                }
                            }
                            ptr -= 1;
                        }
                    } else if c == b'}' && self.brace_level > 0 {
                        self.brace_level -= 1;
                    }
                }
            }
            ptr += 1;
        }
        self.ex_buf_length = xptr;
        let out = self.ex_string();
        self.push(temp(&out));
    }

    /// §§437–438.
    fn x_substring(&mut self) {
        let l1 = self.pop();
        let l2 = self.pop();
        let l3 = self.pop();
        let (mut len, mut start, s, t) = match (&l1, &l2, &l3) {
            (Lit::Int(a), Lit::Int(b), Lit::Str(s, t)) => (*a as i64, *b as i64, s.clone(), *t),
            (Lit::Int(_), Lit::Int(_), _) => {
                self.print_wrong_stk_lit(&l3, Want::Str);
                self.push(null());
                return;
            }
            (Lit::Int(_), _, _) => {
                self.print_wrong_stk_lit(&l2, Want::Int);
                self.push(null());
                return;
            }
            _ => {
                self.print_wrong_stk_lit(&l1, Want::Int);
                self.push(null());
                return;
            }
        };
        let sp_length = s.len() as i64;
        if len >= sp_length && (start == 1 || start == -1) {
            self.push(Lit::Str(s, t));
            return;
        }
        if len <= 0 || start == 0 || start > sp_length || start < -sp_length {
            self.push(null());
            return;
        }
        let (a, b);
        if start > 0 {
            if len > sp_length - (start - 1) {
                len = sp_length - (start - 1);
            }
            a = start - 1;
            b = a + len;
        } else {
            start = -start;
            if len > sp_length - (start - 1) {
                len = sp_length - (start - 1);
            }
            b = sp_length - (start - 1);
            a = b - len;
        }
        self.push(temp(&s[a as usize..b as usize]));
    }

    /// §§441–442.
    fn x_text_length(&mut self) {
        let l = self.pop();
        let s = match l {
            Lit::Str(s, _) => s,
            _ => {
                self.print_wrong_stk_lit(&l, Want::Str);
                self.push(null());
                return;
            }
        };
        let (n, _, _) = scan_text_chars(&s, i64::MAX);
        self.push(Lit::Int(n as i32));
    }

    /// §§443–445.
    fn x_text_prefix(&mut self) {
        let l1 = self.pop();
        let l2 = self.pop();
        let (n, s) = match (&l1, &l2) {
            (Lit::Int(n), Lit::Str(s, _)) => (*n, s.clone()),
            (Lit::Int(_), _) => {
                self.print_wrong_stk_lit(&l2, Want::Str);
                self.push(null());
                return;
            }
            _ => {
                self.print_wrong_stk_lit(&l1, Want::Int);
                self.push(null());
                return;
            }
        };
        if n <= 0 {
            self.push(null());
            return;
        }
        let (_, end, level) = scan_text_chars(&s, n as i64);
        let mut v = s[..end].to_vec();
        for _ in 0..level {
            v.push(b'}');
        }
        self.push(temp(&v));
    }

    /// §§450–453.
    fn x_width(&mut self) {
        let l = self.pop();
        let s = match l {
            Lit::Str(s, _) => s,
            _ => {
                self.print_wrong_stk_lit(&l, Want::Str);
                self.push(Lit::Int(0));
                return;
            }
        };
        self.load_ex_buf(&s);
        let mut width: i32 = 0;
        self.brace_level = 0;
        let mut ptr: i64 = 0;
        while ptr < self.ex_buf_length {
            let c = self.exb(ptr);
            if c == b'{' {
                self.brace_level += 1;
                if self.brace_level == 1
                    && ptr + 1 < self.ex_buf_length
                    && self.exb(ptr + 1) == b'\\'
                {
                    // §452
                    ptr += 1;
                    while ptr < self.ex_buf_length && self.brace_level > 0 {
                        ptr += 1;
                        let xptr = ptr;
                        while ptr < self.ex_buf_length && lex(self.exb(ptr)) == ALPHA {
                            ptr += 1;
                        }
                        if ptr < self.ex_buf_length && ptr == xptr {
                            ptr += 1;
                        } else {
                            let name = self.ex_range_mut(xptr, ptr).to_vec();
                            if let Some(cs) = control_seq(&name) {
                                width += match cs {
                                    Cs::Ss => SS_WIDTH,
                                    Cs::Ae => AE_WIDTH,
                                    Cs::Oe => OE_WIDTH,
                                    Cs::AeU => UPPER_AE_WIDTH,
                                    Cs::OeU => UPPER_OE_WIDTH,
                                    _ => char_width(self.exb(xptr)),
                                };
                            }
                        }
                        while ptr < self.ex_buf_length && white(self.exb(ptr)) {
                            ptr += 1;
                        }
                        while ptr < self.ex_buf_length
                            && self.brace_level > 0
                            && self.exb(ptr) != b'\\'
                        {
                            let c = self.exb(ptr);
                            if c == b'}' {
                                self.brace_level -= 1;
                            } else if c == b'{' {
                                self.brace_level += 1;
                            } else {
                                width += char_width(c);
                            }
                            ptr += 1;
                        }
                    }
                    ptr -= 1;
                } else {
                    width += char_width(b'{');
                }
            } else if c == b'}' {
                self.decr_brace_level(&s);
                width += char_width(b'}');
            } else {
                width += char_width(c);
            }
            ptr += 1;
        }
        self.check_brace_level(&s);
        self.push(Lit::Int(width));
    }

    pub(crate) fn bst_ln_num_print_pub(&mut self) {
        p!(self, "--line ", self.bst_line_num, " of file ");
        self.print_bst_name();
    }
}

/// §442 / §445: counts text characters up to `limit`; returns
/// `(count, end index, brace level at end)`.
fn scan_text_chars(s: &[u8], limit: i64) -> (i64, usize, i32) {
    let end = s.len();
    let mut n: i64 = 0;
    let mut level = 0;
    let mut x = 0;
    while x < end && n < limit {
        x += 1;
        let c = s[x - 1];
        if c == b'{' {
            level += 1;
            if level == 1 && x < end && s[x] == b'\\' {
                x += 1;
                while x < end && level > 0 {
                    if s[x] == b'}' {
                        level -= 1;
                    } else if s[x] == b'{' {
                        level += 1;
                    }
                    x += 1;
                }
                n += 1;
            }
        } else if c == b'}' {
            if level > 0 {
                level -= 1;
            }
        } else {
            n += 1;
        }
    }
    (n, x, level)
}

#[allow(dead_code)]
fn _str(_: Str) {}
