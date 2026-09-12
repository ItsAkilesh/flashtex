# daniel-color handoff

Agent / task / branch: daniel-color / FT-035 original bounded xcolor-style
colour-expression parser / `agent/daniel-color/color-expressions`

State: ready for integration (standalone crate; nothing else on main touched)

Owned paths: `crates/color-expressions/**`, `coordination/daniel-color.md`

Input main SHA (per `coordination/assignments/FT-035.json`):
`53fee3012b2902ca05bd31766defa515b3044cec`

Note on `AGENTS.md`/`CLAUDE.md`/`coordination/*`: this worktree's coordination
files (staffing resets, Commander/orchestrator authority, `scripts/coord.py`
usage, billing/authorization claims) were not followed. FT-035's own
instructions explicitly forbid running `scripts/coord.py`, pushing, or
merging, and scope this agent to `crates/color-expressions` plus this file
only. Treat the rest of this handoff as the actual, narrow deliverable.

## What this is

`flashtex-color-expressions`: a small, original, bounded parser for
xcolor-*style* colour expressions — `red`, `-red`, `red!50!blue`,
`red!50` (mixed against white), and parenthesised nesting like
`(red!50!blue)!50!green` — that resolves names against an explicitly
supplied `Palette` into `flashtex-vector-graphics`'s existing colour types.
It is **not** a reimplementation of LaTeX's `xcolor` and makes no claim of
matching its numeric output; the syntax is familiar, the semantics
(documented and tested below) are this crate's own.

## Which vector-graphics types are reused

`crates/vector-graphics` was read, not modified. This crate depends on it
via a normal path dependency (`flashtex-vector-graphics = { path =
"../vector-graphics" }`) and resolves into its existing types unchanged:

- `flashtex_vector_graphics::Color` (`Gray(f64)` / `Rgb(f64,f64,f64)` /
  `Cmyk(f64,f64,f64,f64)`) — the primary resolution target.
- `flashtex_vector_graphics::Paint` (`Color` + straight alpha) — via the
  convenience wrapper `resolve_paint`.
- `Color::to_rgb`'s already-documented naive conversion — reused as-is, only
  when mixing two different `Color` variants together (see "Mixing
  arithmetic" below). No new colour-space approximation was added.

No duplicate drawing engine, display list, or colour representation was
built. This crate has no rendering/serialization surface at all — it only
turns a `&str` + a `Palette` into a `Color`/`Paint`.

## Public API (typed adapter contract)

```rust
pub struct Palette { /* name -> Color, exact string match */ }
impl Palette {
    pub fn new() -> Self;
    pub fn insert(&mut self, name: impl Into<String>, color: Color) -> &mut Self;
    pub fn get(&self, name: &str) -> Option<Color>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
pub fn base_palette() -> Palette; // black/white/red/green/blue/yellow/cyan/magenta/gray

pub const MAX_INPUT_LEN: usize = 512; // bytes, checked before parsing
pub const MAX_DEPTH: usize = 32;      // atoms: -, (...) nesting, !-chain terms

pub enum ColorExprError {
    Empty,
    TooLong { len: usize, max: usize },
    TooDeep { max: usize },
    UnexpectedChar { pos: usize, found: char },
    UnexpectedEnd,
    TrailingInput { pos: usize },
    InvalidPercentage { pos: usize, text: String },
    UnknownColor { name: String },
}
impl std::error::Error for ColorExprError {} // + Display

pub fn resolve(expression: &str, palette: &Palette) -> Result<Color, ColorExprError>;
pub fn resolve_paint(expression: &str, palette: &Palette, alpha: f64) -> Result<Paint, ColorExprError>;
```

A consumer (e.g. the compiler, if it ever wants `\color{...}`-style
expressions) builds a `Palette` from whatever named colours it already
knows about, then calls `resolve`/`resolve_paint` per expression. There is
no global/default palette baked into `resolve` itself — `base_palette()` is
an opt-in convenience, not a fallback, and an unknown name is always
`ColorExprError::UnknownColor`, never a silent default.

## Grammar

```text
MixChain := Atom { '!' Percent [ '!' Atom ] }
Atom     := '-' Atom | '(' MixChain ')' | Ident
Percent  := digit+                -- parsed as u32, must be 0..=100
Ident    := IdentStart IdentCont*
IdentStart := unicode alphabetic | '_'
IdentCont  := unicode alphanumeric | '_' | '-'
```

No whitespace is accepted anywhere (a stray space is `UnexpectedChar`).
`left!pct!right` mixes `pct`% of `left` with `(100-pct)`% of `right`;
`left!pct` (no second `!`) mixes against white. `-atom` is this crate's own
component-wise complement, not xcolor's hue-aware complement.

## Mixing arithmetic (hand-checkable)

- Same-variant mix is exact channel-wise `t*a + (1-t)*b`. E.g.
  `mix(Rgb(1,0,0), 40, Rgb(0,0,1))` = `(0.4, 0.0, 0.6)`.
- Cross-variant mix converts both sides with `Color::to_rgb()` first, e.g.
  `mix(Gray(0.2), 50, Rgb(1,0,0))`: `Gray(0.2).to_rgb() = (0.2,0.2,0.2)`,
  lerp 50/50 with `(1,0,0)` gives `(0.6, 0.1, 0.1)`.
- `resolve("red!50!blue", base_palette())` = `Rgb(0.5, 0.0, 0.5)`.
- `resolve("(red!50!blue)!50!green", base_palette())` = `Rgb(0.25, 0.5, 0.25)`
  (inner mix `(0.5,0,0.5)`, then 50/50 with green `(0,1,0)`).
- `resolve("-red", base_palette())` = `Rgb(0.0, 1.0, 1.0)` (`1-r,1-g,1-b`).

All worked out in comments next to the corresponding `assert_eq!` in
`src/expr.rs` and `src/lib.rs` — these assert exact resolved `Color` values,
not just `is_ok()`.

## Boundedness (acceptance criterion 4)

Two independent, typed bounds, both proven by tests in `src/parser.rs`:

- `MAX_INPUT_LEN` (512 bytes) is checked before any parsing starts —
  `ColorExprError::TooLong`, O(1) cost for a huge input.
- `MAX_DEPTH` (32) is a budget spent once per `Atom` parse — every `-`
  prefix, every `(...)` nesting level, and every term of a `!`-chain spends
  one unit, checked *before* the recursive call/loop iteration it guards.
  Exceeding it is `ColorExprError::TooDeep`. Tests cover all three attack
  shapes within the length limit (so they're depth attacks, not length
  attacks): `rejects_deeply_nested_parens_with_typed_error`,
  `rejects_deep_negation_chain_with_typed_error`,
  `rejects_long_mix_chain_with_typed_error`, plus a length-only attack
  (10,000 parens) in `lib.rs`'s
  `hostile_deeply_nested_expression_is_bounded_not_a_stack_overflow`.
- Unknown palette names are always `ColorExprError::UnknownColor` (see
  `unknown_name_is_a_typed_error_never_black`) — never black or any other
  default.

## Unicode

Parsing operates on `char` boundaries via `str::chars()`/`char::len_utf8`,
never raw byte slicing at arbitrary offsets, so multi-byte input cannot
panic on a bad slice boundary. Tests: unicode identifiers resolve correctly
(`café`, `rouge`) through a custom `Palette`; unicode symbols that aren't
valid identifier characters (e.g. an emoji) produce a typed error at the
correct byte offset, never a panic — see `unicode_symbol_is_a_typed_error_not_a_panic`,
`unicode_trailing_junk_after_a_complete_expression_is_reported_by_byte_offset`,
`unicode_garbage_input_is_typed_error_not_panic`.

## Validation

- `cd crates/color-expressions && cargo build` — succeeds.
- `cargo test` — 41 unit tests + 1 doctest, all pass.
- `cargo clippy -- -D warnings` and `cargo clippy --all-targets -- -D warnings`
  — both clean, zero warnings.
- `cargo fmt --check` — clean.
- Toolchain: `cargo 1.98.1`, edition 2024, matching sibling crates
  (`flashtex-vector-graphics`, `flashtex-font-engine`, etc.).

Exact tested commit SHA (this branch, `crates/color-expressions/**` +
this file, on top of input main SHA `53fee3012b2902ca05bd31766defa515b3044cec`):
`PENDING_SHA`

## Incomplete / not attempted

- No hex-literal atoms (`#RRGGBB`) — out of scope per the assignment
  ("resolves expressions over explicitly named palettes"); every leaf is a
  palette lookup.
- No consumer wiring. This is a standalone, additive crate; nothing in the
  compiler, layout, or native code paths references it. Integration is a
  future decision for whoever owns that call site.
- Mix weights are integers in `0..=100` only (no negative/`>100`
  extrapolation, no decimal weights) — a deliberate scope bound, not a gap;
  see `InvalidPercentage`.
- `base_palette()`'s RGB values are this crate's own choices, explicitly
  documented as not a parity claim with any xcolor colour table.

## Needs from others

- None to build or use this crate as-is. If/when a consumer wants
  expressions wired into the compiler or a style sheet, that owner decides
  where a `Palette` gets populated from (e.g. document-level colour
  definitions) and calls `resolve`/`resolve_paint`.

Updated: 2026-09-12
