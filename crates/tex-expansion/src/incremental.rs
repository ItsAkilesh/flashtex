//! Incremental re-expansion for the IDE.
//!
//! `Engine` snapshots its whole assignment state (`expand::State`: the
//! save stack with every macro/catcode/register, the conditional stack,
//! counter reset lists, ...) at *safe points* -- moments where the input
//! stack holds nothing but the base lexer, which has just consumed an
//! end-of-line, and no prefix/argument/definition scan is in flight. A
//! snapshot taken with the lexer at byte `P` depends only on source bytes
//! `<= P`, so after an edit at byte `E > P` expansion can resume from it
//! over the edited buffer and produce exactly what a from-scratch
//! expansion would.
//!
//! Two things keep a keystroke cheap:
//!
//! 1. **Restart from the nearest earlier checkpoint** (taken every
//!    `checkpoint_interval` bytes of source at safe points), so the
//!    prefix of the output before it is reused verbatim.
//! 2. **Convergence with the old run.** While re-expanding, whenever the
//!    engine reaches a safe point at the shifted position of one of the
//!    previous run's checkpoints (past the edit), the two states are
//!    compared modulo the span shift. If they are equivalent, everything
//!    the previous run produced after that checkpoint is reused with its
//!    spans shifted, and re-expansion stops. In the common case (an edit
//!    inside one paragraph that does not change any macro) the work is
//!    bounded by the distance between two checkpoints, not by the size of
//!    the document.
//!
//! `tests/incremental_tests.rs` proves `edit()` == `expand_str()` over
//! random edits of the oracle corpus and the HW1/HW2 fixtures;
//! `examples/bench_incremental.rs` measures keystroke latency on a 500 KB
//! synthetic document.

use std::rc::Rc;

use crate::error::{Diagnostic, Limits};
use crate::expand::{Checkpoint, Engine, LabelRecord, State};
use crate::span::Span;
use crate::token::Token;

/// One text edit: replace bytes `[start, end)` of the current source with
/// `replacement`. Offsets are byte offsets on UTF-8 boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

/// What one `edit()` call had to do (for tests/benchmarks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditStats {
    /// Byte position of the checkpoint expansion restarted from (0 = the
    /// document start).
    pub restarted_from: usize,
    /// Byte position (in the new source) where the re-run converged with
    /// the previous run and its suffix was reused, if it did.
    pub converged_at: Option<usize>,
    /// Output tokens reused from before the restart checkpoint.
    pub prefix_reused: usize,
    /// Output tokens reused from the previous run after convergence.
    pub suffix_reused: usize,
    /// Output tokens produced by actually running the engine.
    pub tokens_expanded: usize,
    /// Expansion steps the engine performed for this edit.
    pub steps: u64,
}

pub struct IncrementalExpander {
    source: String,
    tokens: Vec<Token>,
    /// Invocation origin of each output token
    /// (`Engine::next_content_token_with_origin`), parallel to `tokens`.
    origins: Vec<Option<Span>>,
    /// Host configuration applied to the fresh engine before checkpoint 0
    /// (host commands, host prelude, state options). Everything it sets must
    /// live in the checkpointed assignment state.
    init: Option<Rc<dyn Fn(&mut Engine)>>,
    diagnostics: Vec<Diagnostic>,
    labels: Vec<LabelRecord>,
    checkpoints: Vec<Checkpoint>,
    limits: Limits,
    checkpoint_interval: usize,
}

impl IncrementalExpander {
    pub fn new(source: &str) -> Self {
        Self::with_options(source, Limits::default(), 2048)
    }

    /// `checkpoint_interval`: minimum number of source bytes between two
    /// checkpoints (a checkpoint costs one clone of the assignment state).
    pub fn with_options(source: &str, limits: Limits, checkpoint_interval: usize) -> Self {
        Self::build(source, limits, checkpoint_interval, None)
    }

    /// [`IncrementalExpander::with_options`], with `init` run on the fresh
    /// engine before anything is read (and before checkpoint 0). `init` must
    /// only change checkpointed state (`declare_host_command`,
    /// `run_host_prelude`, `set_emit_unbalanced_close`), since restored
    /// engines never see it again.
    pub fn with_host(source: &str, limits: Limits, checkpoint_interval: usize, init: Rc<dyn Fn(&mut Engine)>) -> Self {
        Self::build(source, limits, checkpoint_interval, Some(init))
    }

    fn build(source: &str, limits: Limits, checkpoint_interval: usize, init: Option<Rc<dyn Fn(&mut Engine)>>) -> Self {
        let mut me = IncrementalExpander {
            source: source.to_string(),
            tokens: Vec::new(),
            origins: Vec::new(),
            init,
            diagnostics: Vec::new(),
            labels: Vec::new(),
            checkpoints: Vec::new(),
            limits,
            checkpoint_interval: checkpoint_interval.max(1),
        };
        me.full_run();
        me
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Invocation origins, parallel to [`IncrementalExpander::tokens`].
    pub fn origins(&self) -> &[Option<Span>] {
        &self.origins
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn labels(&self) -> &[LabelRecord] {
        &self.labels
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    fn full_run(&mut self) {
        let src: Rc<str> = Rc::from(self.source.as_str());
        let mut engine = Engine::with_limits(&self.source, self.limits);
        if let Some(init) = self.init.clone() {
            init(&mut engine);
        }
        let _ = src;
        self.tokens.clear();
        self.origins.clear();
        self.diagnostics.clear();
        self.labels.clear();
        self.checkpoints.clear();
        self.checkpoints.push(engine.snapshot(0));
        let mut last_cp = 0usize;
        self.drive(&mut engine, &mut last_cp, None);
        self.diagnostics = engine.take_diagnostics();
        self.labels = engine.take_labels();
    }

    /// Run `engine` to the end (or until it converges with an old
    /// checkpoint, when `converge` is given), appending output tokens and
    /// recording new checkpoints. Returns the convergence point if any.
    fn drive(&mut self, engine: &mut Engine, last_cp: &mut usize, converge: Option<&mut Converge>) -> Option<usize> {
        let mut converge = converge;
        // The engine's diagnostics/labels vectors start empty after a
        // restore, so checkpoint counts must be offset to absolute
        // positions in `self.diagnostics`/`self.labels`.
        let diag_base = self.diagnostics.len();
        let label_base = self.labels.len();
        loop {
            let (tok, origin) = match engine.next_content_token_with_origin() {
                Some(t) => t,
                None => return None,
            };
            self.tokens.push(tok);
            self.origins.push(origin);
            if self.tokens.len() as u64 > self.limits.max_output_tokens {
                engine.push_diagnostic(Diagnostic::error("output token limit exceeded", Span::synthetic()));
                return None;
            }
            if let Some(pos) = engine.safe_point() {
                if let Some(c) = converge.as_deref_mut() {
                    if pos >= c.new_edit_end {
                        if let Some(old_idx) = c.candidate_at(pos) {
                            let old = &c.old_checkpoints[old_idx];
                            if states_equivalent(&old.state, engine.state(), c) && old.lex_state == engine.lex_state() {
                                return Some(old_idx);
                            }
                        }
                    }
                }
                if pos - *last_cp >= self.checkpoint_interval {
                    let mut cp = engine.snapshot(self.tokens.len());
                    cp.diag_len += diag_base;
                    cp.label_len += label_base;
                    self.checkpoints.push(cp);
                    *last_cp = pos;
                }
            }
        }
    }

    /// Apply one edit and bring `tokens()`/`diagnostics()`/`labels()` up
    /// to date, re-expanding as little as the checkpoints allow.
    pub fn edit(&mut self, edit: &Edit) -> EditStats {
        let Edit { start, end, replacement } = edit;
        let (start, end) = (*start, *end);
        assert!(start <= end && end <= self.source.len(), "edit range out of bounds");
        assert!(self.source.is_char_boundary(start) && self.source.is_char_boundary(end), "edit not on char boundary");
        let old_len = self.source.len();
        let delta = replacement.len() as isize - (end - start) as isize;
        self.source.replace_range(start..end, replacement);
        let new_edit_end = start + replacement.len();

        // 1. Nearest checkpoint strictly before the edit (the lexer at
        //    P has looked at byte P to end the previous token, so P must
        //    itself be unchanged: P < start). The initial checkpoint at
        //    0 has looked at nothing and is always usable.
        let cp_idx = self
            .checkpoints
            .iter()
            .rposition(|cp| cp.pos == 0 || cp.pos < start)
            .expect("checkpoint 0 always exists");
        let cp = self.checkpoints[cp_idx].clone();
        let old_checkpoints: Vec<Checkpoint> = self.checkpoints.drain(cp_idx + 1..).collect();
        let old_tokens: Vec<Token> = self.tokens.drain(cp.out_len..).collect();
        let old_origins: Vec<Option<Span>> = self.origins.drain(cp.out_len..).collect();
        let old_diags: Vec<Diagnostic> = self.diagnostics.drain(cp.diag_len..).collect();
        let old_labels: Vec<LabelRecord> = self.labels.drain(cp.label_len..).collect();
        let prefix_reused = self.tokens.len();

        // 2. Re-expand from it over the new buffer.
        let mut engine = Engine::restore(Rc::from(self.source.as_str()), &cp, self.limits);
        let steps_before = engine.steps();
        let mut last_cp = cp.pos;
        let mut conv = Converge {
            old_checkpoints,
            edit_start: start,
            old_edit_end: end,
            new_edit_end,
            delta,
            old_len,
        };
        let converged = self.drive(&mut engine, &mut last_cp, Some(&mut conv));
        let tokens_expanded = self.tokens.len() - prefix_reused;
        let mut stats = EditStats {
            restarted_from: cp.pos,
            converged_at: None,
            prefix_reused,
            suffix_reused: 0,
            tokens_expanded,
            steps: engine.steps() - steps_before,
        };
        let new_diags = engine.take_diagnostics();
        self.diagnostics.extend(new_diags);
        self.labels.extend(engine.take_labels());

        // 3. Splice the old suffix back in if we converged.
        if let Some(old_idx) = converged {
            let old_cp = &conv.old_checkpoints[old_idx];
            let base_out = old_cp.out_len - cp.out_len;
            let base_diag = old_cp.diag_len - cp.diag_len;
            let base_label = old_cp.label_len - cp.label_len;
            stats.converged_at = Some((old_cp.pos as isize + delta) as usize);
            stats.suffix_reused = old_tokens.len() - base_out;
            for (t, o) in old_tokens[base_out..].iter().zip(&old_origins[base_out..]) {
                self.tokens.push(Token::new(t.kind.clone(), conv.shift_span(t.span).unwrap_or(t.span)));
                self.origins.push(o.map(|o| conv.shift_span(o).unwrap_or(o)));
            }
            for d in &old_diags[base_diag..] {
                let mut d = d.clone();
                d.span = conv.shift_span(d.span).unwrap_or(d.span);
                self.diagnostics.push(d);
            }
            for l in &old_labels[base_label..] {
                let mut l = l.clone();
                l.span = conv.shift_span(l.span).unwrap_or(l.span);
                self.labels.push(l);
            }
            // The new run's own checkpoints up to the convergence point
            // were already recorded by `drive`; keep the old run's
            // checkpoints after it, shifted (their states are valid for
            // the new buffer since the runs are equivalent from here on).
            let out_offset = self.tokens.len() as isize - (old_cp.out_len as isize + stats.suffix_reused as isize);
            let diag_offset = self.diagnostics.len() as isize - (old_diags.len() as isize - base_diag as isize) - old_cp.diag_len as isize;
            let label_offset = self.labels.len() as isize - (old_labels.len() as isize - base_label as isize) - old_cp.label_len as isize;
            for old in conv.old_checkpoints.iter().skip(old_idx) {
                let mut shifted = match conv.shift_state(&old.state) {
                    Some(s) => Checkpoint {
                        pos: (old.pos as isize + delta) as usize,
                        lex_state: old.lex_state,
                        state: s,
                        steps: old.steps,
                        out_len: (old.out_len as isize + out_offset) as usize,
                        diag_len: (old.diag_len as isize + diag_offset) as usize,
                        label_len: (old.label_len as isize + label_offset) as usize,
                    },
                    None => continue,
                };
                if let Some(last) = self.checkpoints.last() {
                    if shifted.pos <= last.pos {
                        continue;
                    }
                }
                shifted.steps = old.steps;
                self.checkpoints.push(shifted);
            }
        }
        stats
    }
}

/// Bookkeeping for the convergence search during one edit.
pub(crate) struct Converge {
    old_checkpoints: Vec<Checkpoint>,
    pub edit_start: usize,
    pub old_edit_end: usize,
    pub new_edit_end: usize,
    pub delta: isize,
    old_len: usize,
}

impl Converge {
    /// Index of the old checkpoint whose shifted position is exactly
    /// `new_pos`, if any.
    fn candidate_at(&self, new_pos: usize) -> Option<usize> {
        let old_pos = new_pos as isize - self.delta;
        if old_pos < 0 || old_pos as usize <= self.old_edit_end.min(self.old_len) && old_pos as usize <= self.old_edit_end {
            // Old checkpoints inside/before the edited region cannot be
            // convergence points.
            if old_pos < self.old_edit_end as isize {
                return None;
            }
        }
        self.old_checkpoints.iter().position(|cp| cp.pos as isize == old_pos)
    }

    /// Map a span from the old buffer to the new one: unchanged before
    /// the edit, shifted after it, `None` if it touches the edited region.
    pub fn shift_span(&self, s: Span) -> Option<Span> {
        if s.is_synthetic() || s.source_id != 0 {
            return Some(s);
        }
        let (start, end) = (s.start as usize, s.end as usize);
        if end <= self.edit_start && start < self.edit_start {
            Some(s)
        } else if start >= self.old_edit_end {
            Some(Span::new(0, (start as isize + self.delta) as u32, (end as isize + self.delta) as u32))
        } else if start == end && start <= self.edit_start {
            Some(s)
        } else {
            None
        }
    }

    pub fn shift_state(&self, st: &State) -> Option<State> {
        st.map_spans(&|s| self.shift_span(s))
    }
}

fn states_equivalent(old: &State, new: &State, c: &Converge) -> bool {
    match c.shift_state(old) {
        Some(shifted) => &shifted == new,
        None => false,
    }
}
