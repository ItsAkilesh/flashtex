# `crates/tex-expansion` — adoption contract

**Owner:** kabir-claude (task KC-101). **Status:** standalone, not yet wired
into `crates/compiler` — this document proposes how another agent would do
that; I have not modified `crates/compiler` myself (other agents are
actively editing it).

## What this crate is

A faithful implementation of TeX's *expansion processor*: category codes
and tokenization (TeXbook ch. 7-8), the control-sequence table with
grouping/save-stack semantics (ch. 24), `\def`/`\edef`/`\gdef`/`\xdef` and
macro calling with delimited/undelimited/`#{`-delimited parameters (ch.
20), `\let`/`\futurelet`, `\expandafter`/`\noexpand`/`\csname`, TeX and
e-TeX conditionals, `\count`/`\dimen`/`\skip`/`\toks` registers with
`\advance`/`\multiply`/`\divide`/`\numexpr`/`\dimexpr`, and a LaTeX layer
(`\newcommand`/`\newenvironment`/counters/`\@ifnextchar`/`\@ifstar`/
`\@namedef`/`\@nameuse`).

Its public entry point (`src/lib.rs`):

```rust
pub fn expand_str(source: &str) -> ExpandResult; // { tokens: Vec<Token>, diagnostics: Vec<Diagnostic> }
pub struct Engine<'a> { ... } // lower-level: new(), run(), diagnostics(), set_mode(), set_font_metrics()
```

`Token` carries a `Span { source_id, start, end }` back into the original
source string for every token that came from real source bytes (tokens
synthesized purely by expansion use `Span::synthetic()`), which is what the
IDE needs to map preview glyphs back to exact source characters.

## What it deliberately is not

It does not typeset. Register/macro assignments *execute* as they are
encountered (interleaved with expansion, exactly like real TeX's main
control), so **the output token stream contains no `\def`/`\let`/register
assignments, no conditionals, and no bare grouping braces** — only content
(`Char`/`ActiveChar` tokens) and any control sequence this crate doesn't
recognize, left untouched for the typesetting layer (e.g. `\section`,
`\hskip`, font-switching commands, anything from a class/package we have
no model of). An unrecognized control sequence is *not* flagged as an
error — this crate only knows the TeX/e-TeX/LaTeX-kernel primitives it
implements, not the rest of the LaTeX universe, so treating "not ours" as
"undefined" would be wrong.

## Proposed adoption path for `crates/compiler`

1. **Add the dependency.** `crates/compiler/Cargo.toml`:
   `flashtex-tex-expansion = { path = "../tex-expansion" }`.

2. **Preprocessing stage, before the existing lexer/parser.** Today
   `crates/compiler/src/parser.rs` does its own bounded `\newcommand`
   substitution inline while parsing (see `define_macro` around
   parser.rs:1392, and the hardcoded `"newcommand"|"renewcommand"` match at
   parser.rs:811/374-375) — there is no `\def`, no conditionals, no
   registers at all. The proposed split:

   ```
   source text
     -> flashtex_tex_expansion::Engine::run()   // NEW: expansion stage
     -> Vec<Token>  (content tokens + span provenance)
     -> crates/compiler's existing lexer/parser, adapted to consume
        Vec<Token> instead of re-lexing raw characters
     -> layout (unchanged)
   ```

   Concretely: `crates/compiler/src/lexer.rs` currently turns source bytes
   into whatever token type `parser.rs` consumes. The adoption diff is to
   insert the expansion pass *before* that: run `expand_str` (or drive
   `Engine` incrementally, see "Incremental compilation" below) first, then
   feed its `Token` stream through a thin adapter that turns
   `flashtex_tex_expansion::TokenKind` into whatever `crates/compiler`'s
   parser expects (`Char` -> existing character token; `ControlSequence`
   -> existing control-sequence token; span carried through unchanged so
   existing diagnostic code keeps working).

3. **Remove the compiler's inline `\newcommand` handling.** Once macro
   expansion happens upstream, `parser.rs`'s `define_macro`/`PARSABLE`
   command list (parser.rs:374-375) and the `newcommand`/`renewcommand`
   match arm (parser.rs:811) should be deleted — by that point in the
   pipeline `\greet{world}` has *already* become `Hello, world!` as plain
   character tokens; the parser never sees a macro call at all. (I have
   **not** made this edit — it is squarely inside `crates/compiler`, owned
   by other in-flight agents per the task's ownership rules.)

4. **Diagnostics.** `flashtex_tex_expansion::Diagnostic` is a plain
   `{ severity, message, span }` struct — map it 1:1 onto whatever
   diagnostic type `crates/compiler/src/diagnostics.rs` uses; the spans
   are already in the same byte-offset space as the original source.

5. **Font metrics for `em`/`ex`.** `Engine::set_font_metrics` takes a
   `Box<dyn FontMetrics>` (`quad_sp()`, `x_height_sp()`); wire this to
   whatever `crates/font-engine`/`crates/font-resources` expose for the
   current font at the point expansion runs. Until that's wired,
   `DefaultFontMetrics` (10pt CM-like defaults) is used, which will be
   *wrong* for any document using `em`/`ex` in a different font — flagged
   here rather than silently shipped as correct.

6. **Mode-dependent conditionals.** `\ifvmode`/`\ifhmode`/`\ifmmode`/
   `\ifinner` need real typesetting-mode state, which this crate does not
   have (no typesetter). `Engine::set_mode(Mode)` is a manual override the
   host must call before/while expanding if a document actually uses these
   (rare in practice; most documents don't inspect their own mode). A
   fuller integration would thread mode changes from the compiler's layout
   stage back into the engine, which requires interleaving expansion with
   layout rather than doing it as one upfront pass — out of scope for this
   task; flagged as a real limitation for documents that use `\ifmmode`
   etc. structurally.

7. **Incremental compilation.** `crates/compiler/src/incremental.rs`
   exists and is tested against `\newcommand`/`\renewcommand` edits
   already (see its tests around line 829-845). `Engine` as built runs
   start-to-finish over a whole source string; it is **not** yet
   incremental. A real integration needs either (a) accept the
   full-reparse cost for now (macro expansion is fast — pure Rust, no I/O)
   and let `incremental.rs`'s existing diffing operate on the *output*
   token stream instead of raw source, or (b) extend `Engine` with a
   restart-from-checkpoint API (save/restore `Scopes` state at paragraph
   boundaries). I have not built (b); it's the main piece of follow-up
   work I'd flag for whoever adopts this.

## Known deviations from real TeX (found via the oracle corpus)

- **Bare grouping braces produce no token.** Verified against real TeX:
  `{\def\a{inner}}\a` written via `\write` (which is itself a
  `scan_toks`-style capture, *not* real document typesetting) still
  observably yields just the macro's expansion with no literal `{`/`}`
  characters once you account for where the braces actually are. Real
  TeX's main control does *not* append a character node for catcode-1/2
  tokens — it only does the grouping bookkeeping — so this crate matches
  that (swallows them after pushing/popping scope), which is the correct
  behavior for feeding a typesetter. The flip side: `\write`'s *own*
  argument-scanning is a different TeX subsystem (`scan_toks`) that *does*
  preserve nested braces literally as balanced delimiters — this is why
  the oracle generator (`tests/oracle/gen/cases.py`) cannot use `\write`
  to validate constructs like `#{` (brace-delimited last parameter) whose
  leftover brace group is meant to be processed at the top level; those
  are covered by Rust-only unit tests instead (see comments in both
  files).
- **`\let`-to-a-character tokens are substituted immediately.** `\let\a=b`
  then using `\a` produces the character `b` directly in this crate's
  output. Real TeX's `\write` would instead print the literal token `\a `
  (since such a token isn't "expandable" in the technical sense `\write`'s
  restricted scan checks for) — but real TeX's *main control*, when
  actually typesetting, treats that same token exactly as if `b` had been
  typed. Since this crate's job is to feed a typesetter, we match main
  control, not `\write`. (No oracle case exercises this specific
  distinction for that reason — see the comment above `let_to_char` in
  `tests/oracle/gen/cases.py`.)
- **`\fnsymbol`** renders ASCII approximations (`*`, `**`, ..., `#`, `##`,
  `###`) instead of real LaTeX's math-mode footnote symbols
  (asterisk-operator, dagger, double-dagger, ...), since those aren't
  representable as plain catcode-Other characters without a math/symbol
  font this crate has no access to.
- **Catcode assignments are grouped/restored correctly**, but there is
  **no font/family/other "current font" state** at all — this crate only
  models the macro-expansion layer, not `\font`/`\textfont` etc.
- **`\newcounter`'s `[within]` parent is recorded but not acted on** —
  stepping a parent counter does not yet reset its children (LaTeX's
  `@removefromreset`/`@addtoreset` machinery isn't modeled). Flagged, not
  fixed, for time reasons.
- **`\long`/`\outer` flags are parsed and stored on `MacroDef` but not
  enforced** (a non-`\long` macro should reject `\par` inside an argument;
  `\outer` macros should be rejected in certain contexts). Parsing is
  correct; enforcement is a follow-up.
- **e-TeX `\numexpr`/`\dimexpr`**: implemented with standard `+ - * /` and
  parentheses, matching e-TeX's round-to-nearest (ties away from zero)
  division semantics (verified against real `etex` via the oracle). Unary
  minus and other e-TeX expression forms beyond this are not implemented.

## Oracle corpus

`tests/oracle/gen/cases.py` drives real `tex`, `etex`, and `pdflatex`
(MacTeX, oracle-only per KC-101 — never a build/runtime dependency of this
crate or the product) to capture ground-truth output for each case, three
ways depending on what the case needs:

- **`"tex"`/`"etex"` mode**: wraps the case in `\immediate\write` and reads
  back the written file. Fast and exact, but `\write`'s argument scanning
  is itself a `scan_toks` context (see "Known deviations" above) — not
  usable for cases whose observable behavior depends on real top-level
  grouping, or on non-expandable-token identity.
- **`"latex-render"` mode**: typesets the case as ordinary `article` body
  content between two unique marker strings, renders to PDF via
  `pdflatex`, and extracts the text between the markers with `pdftotext`.
  Used for the LaTeX-kernel macros (`\newcommand`, counters,
  `\@ifnextchar`, ...) whose internals use `\let`/`\futurelet` and so
  cannot be captured through `\write`'s restricted scan.

Each case's `(setup, expr, expected)` is committed to
`tests/oracle/manifest.json`; `tests/oracle_tests.rs` reads that fixture
and asserts `expand_str(setup + expr)` matches `expected` **without ever
invoking TeX itself** at `cargo test` time. To add cases or regenerate
after an engine change that legitimately changes output, run (with
MacTeX/TeX Live's `tex`/`etex`/`pdflatex`/`pdftotext` on `PATH`):

```
python3 tests/oracle/gen/cases.py
```

**108 / 108 oracle cases pass** as of this commit, plus **46 Rust-only unit
tests** in `tests/expand_tests.rs` covering the same feature areas (plus
the two documented brace/let-to-char distinctions the oracle mechanism
itself can't validate). This is short of the ≥150-case target in the task
brief — see "What's not done" below.

## What's done vs. not done

**Done and oracle/unit-tested:** catcodes + tokenizer (incl. `^^`
notation, comments, `\par` from blank lines), `\def`/`\edef`/`\gdef`/
`\xdef` with delimited/undelimited/`#{` parameters, `\let`/`\futurelet`,
`\expandafter`/`\noexpand`/`\csname`/`\endcsname`, `\string`/`\number`/
`\romannumeral`/`\the`, grouping with `\global`/`\aftergroup`, `\catcode`/
`\makeatletter`/`\makeatother`, all listed TeX+e-TeX conditionals
including `\ifcase`/`\newif`/`\unless`, `\count`/`\dimen`/`\skip`/`\toks`
registers with the `...def` family and `\advance`/`\multiply`/`\divide`/
`\numexpr`/`\dimexpr`, and the LaTeX layer (`\newcommand`/`\renewcommand`/
`\providecommand` incl. `*` and optional-first-argument default,
`\newenvironment`/`\renewenvironment`, `\newcounter`/`\setcounter`/
`\addtocounter`/`\stepcounter`/`\refstepcounter`/`\value`/`\arabic`/
`\roman`/`\Roman`/`\alph`/`\Alph`/`\fnsymbol`, `\@ifnextchar`/`\@ifstar`/
`\@namedef`/`\@nameuse`). Resource limits (`Limits`: max expansion steps,
max output tokens) prevent infinite macro loops from hanging; a runaway
argument/group hits a diagnostic instead of panicking anywhere in the
engine (no `unwrap`/`expect`/panic on malformed input was intentionally
left in the hot paths — see error.rs).

**Not done** (see "Known deviations" for why, where applicable):
- `\long`/`\outer` enforcement (parsed, not enforced).
- Mode-aware `\ifvmode`/`\ifhmode`/`\ifmmode`/`\ifinner` (manual override
  only, no real typesetting-mode feed).
- `\newcounter`'s `[within]` reset propagation.
- Incremental/checkpointed expansion (whole-document pass only).
- The oracle corpus is 108 cases, not ≥150 — time-bounded; the highest
  syntactic-risk areas (delimited params, `#{`, grouping/`\global`
  interaction, all conditional forms, counters, optional-arg
  `\newcommand`) are covered, but plenty of TeXbook corner cases (e.g.
  deeper `\edef`+`\noexpand` interactions, `\aftergroup` ordering with
  multiple queued tokens, catcode-13 active-character macros beyond the
  one `~` case) are not yet exercised.
- No fuzzing/property testing against the resource limits — only the one
  direct case is verified (`infinite_macro_loop_terminates_with_diagnostic`
  in `tests/expand_tests.rs`: `\def\loop{\loop}\loop` terminates with a
  single diagnostic and no panic, rather than hanging), not a broader
  sweep of worst-case shapes (e.g. loops that also grow the output-token
  count, deeply nested group/conditional recursion, etc).
