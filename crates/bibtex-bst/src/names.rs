//! `format.name$` (§§382–420).

use std::rc::Rc;

use crate::chars::{control_seq, lex, white, Cs, ALPHA, SEP};
use crate::engine::{p, Engine, Lit, R};
use crate::exec::Want;

impl<'a> Engine<'a> {
    fn nb(&self, i: i64) -> u8 {
        if i >= 0 && (i as usize) < self.sv_buffer.len() {
            self.sv_buffer[i as usize]
        } else {
            0
        }
    }

    fn nb_set(&mut self, i: i64, c: u8) {
        let i = i as usize;
        if i + 1 >= self.sv_buffer.len() {
            self.sv_buffer.resize(i + 20000, 0);
        }
        self.sv_buffer[i] = c;
    }

    fn nt(&self, i: i64) -> i64 {
        if i >= 0 && (i as usize) < self.name_tok.len() {
            self.name_tok[i as usize]
        } else {
            0
        }
    }

    fn nt_set(&mut self, i: i64, v: i64) {
        let i = i as usize;
        if i + 1 >= self.name_tok.len() {
            self.name_tok.resize(i + 20000, 0);
            self.name_sep_char.resize(i + 20000, 0);
        }
        self.name_tok[i] = v;
    }

    fn nsc(&self, i: i64) -> u8 {
        if i >= 0 && (i as usize) < self.name_sep_char.len() {
            self.name_sep_char[i as usize]
        } else {
            0
        }
    }

    fn nsc_set(&mut self, i: i64, c: u8) {
        let i = i as usize;
        if i + 1 >= self.name_sep_char.len() {
            self.name_tok.resize(i + 20000, 0);
            self.name_sep_char.resize(i + 20000, 0);
        }
        self.name_sep_char[i] = c;
    }

    fn append_ex(&mut self, c: u8) {
        self.exb_set(self.ex_buf_ptr, c);
        self.ex_buf_ptr += 1;
    }

    fn bst_ex_warn(&mut self, msg: &str) {
        p!(self, msg);
        self.bst_ex_warn_print();
    }

    /// §382.
    pub(crate) fn x_format_name(&mut self) -> R<()> {
        let l1 = self.pop_lit();
        let l2 = self.pop_lit();
        let l3 = self.pop_lit();
        let (fmt, which, names) = match (&l1, &l2, &l3) {
            (Lit::Str(f, _), Lit::Int(n), Lit::Str(s, _)) => (f.clone(), *n, s.clone()),
            (Lit::Str(..), Lit::Int(_), _) => {
                self.wrong(&l3, Want::Str);
                self.push_lit(Lit::Str(Rc::from(&b""[..]), false));
                return Ok(());
            }
            (Lit::Str(..), _, _) => {
                self.wrong(&l2, Want::Int);
                self.push_lit(Lit::Str(Rc::from(&b""[..]), false));
                return Ok(());
            }
            _ => {
                self.wrong(&l1, Want::Str);
                self.push_lit(Lit::Str(Rc::from(&b""[..]), false));
                return Ok(());
            }
        };
        self.load_ex_buf(&names);
        // §383
        self.ex_buf_ptr = 0;
        let mut num_names = 0;
        while num_names < which && self.ex_buf_ptr < self.ex_buf_length {
            num_names += 1;
            self.ex_buf_xptr = self.ex_buf_ptr;
            self.name_scan_for_and(&names);
        }
        if self.ex_buf_ptr < self.ex_buf_length {
            self.ex_buf_ptr -= 4;
        }
        if num_names < which {
            if which == 1 {
                p!(self, "There is no name in \"");
            } else {
                p!(self, "There aren't ", which, " names in \"");
            }
            p!(self, names);
            self.bst_ex_warn("\"");
        }
        // §388 (the leading-junk loop is removed by bibtex.ch [388])
        while self.ex_buf_ptr > self.ex_buf_xptr {
            let c = self.exb(self.ex_buf_ptr - 1);
            let cl = lex(c);
            if cl == crate::chars::WHITE || cl == SEP {
                self.ex_buf_ptr -= 1;
            } else if c == b',' {
                p!(self, "Name ", which, " in \"", names, "\" has a comma at the end");
                self.bst_ex_warn_print();
                self.ex_buf_ptr -= 1;
            } else {
                break;
            }
        }
        // §387
        let mut name_bf_ptr: i64 = 0;
        let mut num_commas = 0;
        let mut comma1: i64 = 0;
        let mut comma2: i64 = 0;
        let mut num_tokens: i64 = 0;
        let mut token_starting = true;
        while self.ex_buf_xptr < self.ex_buf_ptr {
            let c = self.exb(self.ex_buf_xptr);
            match c {
                b',' => {
                    // §389
                    if num_commas == 2 {
                        p!(self, "Too many commas in name ", which, " of \"", names, "\"");
                        self.bst_ex_warn_print();
                    } else {
                        num_commas += 1;
                        if num_commas == 1 {
                            comma1 = num_tokens;
                        } else {
                            comma2 = num_tokens;
                        }
                        self.nsc_set(num_tokens, b',');
                    }
                    self.ex_buf_xptr += 1;
                    token_starting = true;
                }
                b'{' => {
                    // §390
                    self.brace_level += 1;
                    if token_starting {
                        self.nt_set(num_tokens, name_bf_ptr);
                        num_tokens += 1;
                    }
                    self.nb_set(name_bf_ptr, c);
                    name_bf_ptr += 1;
                    self.ex_buf_xptr += 1;
                    while self.brace_level > 0 && self.ex_buf_xptr < self.ex_buf_ptr {
                        let c = self.exb(self.ex_buf_xptr);
                        if c == b'}' {
                            self.brace_level -= 1;
                        } else if c == b'{' {
                            self.brace_level += 1;
                        }
                        self.nb_set(name_bf_ptr, c);
                        name_bf_ptr += 1;
                        self.ex_buf_xptr += 1;
                    }
                    token_starting = false;
                }
                b'}' => {
                    // §391
                    if token_starting {
                        self.nt_set(num_tokens, name_bf_ptr);
                        num_tokens += 1;
                    }
                    p!(self, "Name ", which, " of \"", names);
                    self.bst_ex_warn("\" isn't brace balanced");
                    self.ex_buf_xptr += 1;
                    token_starting = false;
                }
                _ => {
                    let cl = lex(c);
                    if cl == crate::chars::WHITE {
                        // §392
                        if !token_starting {
                            self.nsc_set(num_tokens, b' ');
                        }
                        self.ex_buf_xptr += 1;
                        token_starting = true;
                    } else if cl == SEP {
                        // §393
                        if !token_starting {
                            self.nsc_set(num_tokens, c);
                        }
                        self.ex_buf_xptr += 1;
                        token_starting = true;
                    } else {
                        // §394
                        if token_starting {
                            self.nt_set(num_tokens, name_bf_ptr);
                            num_tokens += 1;
                        }
                        self.nb_set(name_bf_ptr, c);
                        name_bf_ptr += 1;
                        self.ex_buf_xptr += 1;
                        token_starting = false;
                    }
                }
            }
        }
        self.nt_set(num_tokens, name_bf_ptr);
        // §395
        let (first_start, first_end, last_end, von_start, von_end, jr_end);
        if num_commas == 0 {
            first_start = 0;
            last_end = num_tokens;
            jr_end = last_end;
            // §396
            let mut vs: i64 = 0;
            let mut ve: i64 = 0;
            let mut found = false;
            while vs < last_end - 1 {
                if self.von_token_found(self.nt(vs), self.nt(vs + 1)) {
                    ve = self.von_name_ends(vs, last_end);
                    found = true;
                    break;
                }
                vs += 1;
            }
            if !found {
                while vs > 0 {
                    let c = self.nsc(vs);
                    if lex(c) != SEP || c == b'~' {
                        break;
                    }
                    vs -= 1;
                }
                ve = vs;
            }
            von_start = vs;
            von_end = ve;
            first_end = vs;
        } else {
            von_start = 0;
            last_end = comma1;
            jr_end = if num_commas == 1 { last_end } else { comma2 };
            first_start = jr_end;
            first_end = num_tokens;
            von_end = self.von_name_ends(von_start, last_end);
        }
        // §§402, 420
        self.load_ex_buf(&fmt);
        self.ex_buf_ptr = 0;
        let end = fmt.len() as i64;
        let fc = |i: i64| -> u8 {
            if i >= 0 && i < end { fmt[i as usize] } else { 0 }
        };
        let mut sbl: i32 = 0;
        let mut sp: i64 = 0;
        while sp < end {
            if fc(sp) == b'{' {
                sbl += 1;
                sp += 1;
                // §403
                let mut sp_xptr1 = sp;
                let mut alpha_found = false;
                let mut double_letter = false;
                let mut end_of_group = false;
                let mut to_be_written = true;
                let (mut cur_token, mut last_token) = (0i64, 0i64);
                while !end_of_group && sp < end {
                    let c = fc(sp);
                    if lex(c) == ALPHA {
                        sp += 1;
                        // §405
                        if alpha_found {
                            self.brace_lvl_one_letters_complaint(&fmt);
                            to_be_written = false;
                        } else {
                            let next = fc(sp);
                            let pair = match c {
                                b'f' | b'F' => Some((first_start, first_end, b'f')),
                                b'v' | b'V' => Some((von_start, von_end, b'v')),
                                b'l' | b'L' => Some((von_end, last_end, b'l')),
                                b'j' | b'J' => Some((last_end, jr_end, b'j')),
                                _ => None,
                            };
                            match pair {
                                Some((a, b, letter)) => {
                                    cur_token = a;
                                    last_token = b;
                                    if cur_token == last_token {
                                        to_be_written = false;
                                    }
                                    if next == letter || next == letter - 32 {
                                        double_letter = true;
                                    }
                                }
                                None => {
                                    self.brace_lvl_one_letters_complaint(&fmt);
                                    to_be_written = false;
                                }
                            }
                            if double_letter {
                                sp += 1;
                            }
                        }
                        alpha_found = true;
                    } else if c == b'}' {
                        sbl -= 1;
                        sp += 1;
                        end_of_group = true;
                    } else if c == b'{' {
                        sbl += 1;
                        sp += 1;
                        while sbl > 1 && sp < end {
                            let c = fc(sp);
                            if c == b'}' {
                                sbl -= 1;
                            } else if c == b'{' {
                                sbl += 1;
                            }
                            sp += 1;
                        }
                    } else {
                        sp += 1;
                    }
                }
                if end_of_group && to_be_written {
                    // §411
                    self.ex_buf_xptr = self.ex_buf_ptr;
                    sp = sp_xptr1;
                    sbl = 1;
                    while sbl > 0 {
                        let c = fc(sp);
                        if lex(c) == ALPHA && sbl == 1 {
                            sp += 1;
                            // §412
                            if double_letter {
                                sp += 1;
                            }
                            let mut use_default = true;
                            let mut sp_xptr2 = sp;
                            if fc(sp) == b'{' {
                                use_default = false;
                                sbl += 1;
                                sp += 1;
                                sp_xptr1 = sp;
                                while sbl > 1 && sp < end {
                                    let c = fc(sp);
                                    if c == b'}' {
                                        sbl -= 1;
                                    } else if c == b'{' {
                                        sbl += 1;
                                    }
                                    sp += 1;
                                }
                                sp_xptr2 = sp - 1;
                            }
                            // §413
                            while cur_token < last_token {
                                let mut nbp = self.nt(cur_token);
                                let nbx = self.nt(cur_token + 1);
                                if double_letter {
                                    // §414
                                    while nbp < nbx {
                                        let c = self.nb(nbp);
                                        self.append_ex(c);
                                        nbp += 1;
                                    }
                                } else {
                                    // §415
                                    while nbp < nbx {
                                        let c = self.nb(nbp);
                                        if lex(c) == ALPHA {
                                            self.append_ex(c);
                                            break;
                                        } else if c == b'{'
                                            && nbp + 1 < nbx
                                            && self.nb(nbp + 1) == b'\\'
                                        {
                                            // §416
                                            self.append_ex(b'{');
                                            self.append_ex(b'\\');
                                            nbp += 2;
                                            let mut nml = 1;
                                            while nbp < nbx && nml > 0 {
                                                let c = self.nb(nbp);
                                                if c == b'}' {
                                                    nml -= 1;
                                                } else if c == b'{' {
                                                    nml += 1;
                                                }
                                                self.append_ex(c);
                                                nbp += 1;
                                            }
                                            break;
                                        }
                                        nbp += 1;
                                    }
                                }
                                cur_token += 1;
                                if cur_token < last_token {
                                    // §417
                                    if use_default {
                                        if !double_letter {
                                            self.append_ex(b'.');
                                        }
                                        let sepc = self.nsc(cur_token);
                                        if lex(sepc) == SEP {
                                            self.append_ex(sepc);
                                        } else if cur_token == last_token - 1
                                            || !self.enough_text_chars(3)
                                        {
                                            self.append_ex(b'~');
                                        } else {
                                            self.append_ex(b' ');
                                        }
                                    } else {
                                        let mut q = sp_xptr1;
                                        while q < sp_xptr2 {
                                            let c = fc(q);
                                            self.append_ex(c);
                                            q += 1;
                                        }
                                    }
                                }
                            }
                            if !use_default {
                                sp = sp_xptr2 + 1;
                            }
                        } else if c == b'}' {
                            sbl -= 1;
                            sp += 1;
                            if sbl > 0 {
                                self.append_ex(b'}');
                            }
                        } else if c == b'{' {
                            sbl += 1;
                            sp += 1;
                            self.append_ex(b'{');
                        } else {
                            self.append_ex(c);
                            sp += 1;
                        }
                    }
                    if self.ex_buf_ptr > 0 && self.exb(self.ex_buf_ptr - 1) == b'~' {
                        // §419
                        self.ex_buf_ptr -= 1;
                        if self.exb(self.ex_buf_ptr - 1) == b'~' {
                        } else if !self.enough_text_chars(3) {
                            self.ex_buf_ptr += 1;
                        } else {
                            self.append_ex(b' ');
                        }
                    }
                }
            } else if fc(sp) == b'}' {
                self.braces_unbalanced_complaint(&fmt);
                sp += 1;
            } else {
                let c = fc(sp);
                self.append_ex(c);
                sp += 1;
            }
        }
        if sbl > 0 {
            self.braces_unbalanced_complaint(&fmt);
        }
        self.ex_buf_length = self.ex_buf_ptr;
        let out = self.ex_string();
        self.push_lit(Lit::Str(Rc::from(&out[..]), true));
        Ok(())
    }

    fn brace_lvl_one_letters_complaint(&mut self, fmt: &[u8]) {
        p!(self, "The format string \"", fmt);
        self.bst_ex_warn("\" has an illegal brace-level-1 letter");
    }

    /// §§397–400.
    fn von_token_found(&mut self, mut nbp: i64, nbx: i64) -> bool {
        let mut nml = 0;
        while nbp < nbx {
            let c = self.nb(nbp);
            if c.is_ascii_uppercase() {
                return false;
            } else if c.is_ascii_lowercase() {
                return true;
            } else if c == b'{' {
                nml += 1;
                nbp += 1;
                if nbp + 2 < nbx && self.nb(nbp) == b'\\' {
                    // §398
                    nbp += 1;
                    let yptr = nbp;
                    while nbp < nbx && lex(self.nb(nbp)) == ALPHA {
                        nbp += 1;
                    }
                    let name: Vec<u8> = (yptr..nbp).map(|i| self.nb(i)).collect();
                    if let Some(cs) = control_seq(&name) {
                        // §399
                        return !matches!(cs, Cs::OeU | Cs::AeU | Cs::AaU | Cs::OU | Cs::LU);
                    }
                    while nbp < nbx && nml > 0 {
                        let c = self.nb(nbp);
                        if c.is_ascii_uppercase() {
                            return false;
                        } else if c.is_ascii_lowercase() {
                            return true;
                        } else if c == b'}' {
                            nml -= 1;
                        } else if c == b'{' {
                            nml += 1;
                        }
                        nbp += 1;
                    }
                    return false;
                } else {
                    // §400
                    while nml > 0 && nbp < nbx {
                        let c = self.nb(nbp);
                        if c == b'}' {
                            nml -= 1;
                        } else if c == b'{' {
                            nml += 1;
                        }
                        nbp += 1;
                    }
                }
            } else {
                nbp += 1;
            }
        }
        false
    }

    /// §401: returns `von_end`.
    fn von_name_ends(&mut self, von_start: i64, last_end: i64) -> i64 {
        let mut von_end = last_end - 1;
        while von_end > von_start {
            if self.von_token_found(self.nt(von_end - 1), self.nt(von_end)) {
                return von_end;
            }
            von_end -= 1;
        }
        von_end
    }

    /// §418.
    fn enough_text_chars(&mut self, enough: i64) -> bool {
        let mut num = 0;
        self.ex_buf_yptr = self.ex_buf_xptr;
        while self.ex_buf_yptr < self.ex_buf_ptr && num < enough {
            self.ex_buf_yptr += 1;
            let c = self.exb(self.ex_buf_yptr - 1);
            if c == b'{' {
                self.brace_level += 1;
                if self.brace_level == 1
                    && self.ex_buf_yptr < self.ex_buf_ptr
                    && self.exb(self.ex_buf_yptr) == b'\\'
                {
                    self.ex_buf_yptr += 1;
                    while self.ex_buf_yptr < self.ex_buf_ptr && self.brace_level > 0 {
                        let c = self.exb(self.ex_buf_yptr);
                        if c == b'}' {
                            self.brace_level -= 1;
                        } else if c == b'{' {
                            self.brace_level += 1;
                        }
                        self.ex_buf_yptr += 1;
                    }
                }
            } else if c == b'}' {
                self.brace_level -= 1;
            }
            num += 1;
        }
        num >= enough
    }
}

#[allow(dead_code)]
fn _w(c: u8) -> bool {
    white(c)
}
