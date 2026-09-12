# daniel-calc handoff

Agent / task / branch: `daniel-calc` / FT-045 "tex-calc: bounded TeX dimension
expression evaluator" / `agent/daniel-calc/tex-calc`

State: ready for integration

Owned paths: `crates/tex-calc/**`, `coordination/daniel-calc.md`. No other
crate was edited; `document-style` is consumed read-only via a `path`
dependency, never modified.

Exact tested commit SHA: `021cf9c247c16174c1b3e098d91d90bb0946fe10`
(`cd crates/tex-calc && cargo build && cargo test && cargo clippy
--all-targets -- -D warnings && cargo fmt --check`, all clean, rustc/cargo
1.98.1, at that exact commit).

## What document-style types are consumed

Read `crates/document-style/src/length.rs` and `geometry.rs` first. The
existing `flashtex_document_style::length::Pt` is an `f64` TeX point (72.27
per inch) — not exact, and not what this lane needed. Per the assignment,
this crate does **not** invent a parallel dimension representation for its
own arithmetic: internally every dimension is an exact `i64` count of scaled
points (`Sp`, `1pt = 65536sp`), because `Pt`'s `f64` cannot give reproducible
`\dimexpr`-style arithmetic or a hard `max_dimen` bound. `Pt` is consumed
only as the **interchange type at the crate boundary**:

- `Sp::to_style_pt(self) -> flashtex_document_style::length::Pt` (lossy,
  `Sp -> Pt`).
- `Sp::try_from_style_pt(Pt) -> Result<Sp, CalcError>` (typed `Overflow` on
  out-of-range or non-finite input, truncating toward zero otherwise).

Nothing else from `document-style` (its parser, `Skip`, `geometry`, `fonts`,
`style` modules) is used or duplicated.

## Typed contract (public API)

Crate `flashtex-tex-calc` (`flashtex_tex_calc`), zero deps besides
`flashtex-document-style`.

```rust
pub struct Sp(pub i64);                 // exact scaled points
pub const SP_PER_PT: i64 = 65536;
pub const MAX_DIMEN_SP: i64 = (1i64 << 30) - 1; // 16383.99998pt, tex.web max_dimen

pub enum Unit { Pt, In, Pc, Cm, Mm, Bp, Sp }  // exact units only; em/ex/px/etc rejected

pub enum CalcError {
    Overflow,
    DivisionByZero,
    UnsupportedUnit(String),
    UndefinedLength(String),
    CyclicLength(String),
    ResolutionDepthExceeded,
    Parse { message: String, at: usize },
    TypeMismatch(String),
}

pub fn evaluate(source: &str) -> Result<Sp, CalcError>;   // parse + eval in one call
pub fn parse(source: &str) -> Result<Expr, CalcError>;
pub fn eval::eval(expr: &Expr) -> Result<Sp, CalcError>;
```

Every failure path returns one of these `CalcError` variants — no panics, no
silent approximation, no implicit zero on overflow/unsupported input.

### Expression language

```text
Body    := Stmt* Expr
Stmt    := "\setlength" "{" "\name" "}" "{" Expr "}" ";"
Expr    := Term (("+"|"-") Term)*
Term    := Unary (("*"|"/") Unary)*
Unary   := "-" Unary | Primary
Primary := Number [Unit] | "\name" | "{" Body "}" | "(" Expr ")"
```

`{ ... }` is TeX's own group semantics: `\setlength` inside a group binds a
name only until that group's closing `}`; a name bound in an inner group
shadows the same name in an outer scope and the outer binding is restored
(unmutated) once the inner group closes. Named lengths are resolved lazily
via "blackholed" thunks: forcing a thunk that is already being forced (a
self-reference or any cycle through other names) returns `CyclicLength`
instead of recursing; a `MAX_RESOLUTION_DEPTH = 200` backstops any other
runaway nesting with `ResolutionDepthExceeded`.

## Exactness: how conversions were verified

Every unit ratio matches TeX's own internal table (tex.web `scan_dimen`):
`pt 1/1`, `in 7227/100`, `pc 12/1`, `cm 7227/254`, `mm 7227/2540`,
`bp 7227/7200`, `sp` identity. A literal's decimal text is parsed into an
exact `(numerator, denominator)` pair (never through `f64`), then converted
with `i128` rational arithmetic and **truncated toward zero**, matching real
TeX's well-known behavior (`\dimen0=1in \showthe\dimen0` prints `72.26999pt`,
not `72.27pt`). Hand-computed and independently cross-checked with
`fractions.Fraction` in Python (not the Rust implementation) and asserted in
`src/sp.rs` tests:

| input | exact sp (before truncation) | truncated sp |
|---|---|---|
| `1pt` | 65536 | 65536 |
| `1in` | 4736286.72 | 4736286 |
| `1pc` | 786432 | 786432 |
| `1cm` | 1864679.811... | 1864679 |
| `1mm` | 186467.981... | 186467 |
| `1bp` | 65781.76 | 65781 |
| `10.5cm` | 19579138.0157... | 19579138 |

Boundary: `16383.99999pt` truncates to exactly `MAX_DIMEN_SP = 1073741823`;
`16384pt` and `1073741824sp` both overflow; `1073741823sp` / `-1073741823sp`
(±`MAX`) succeed. Arithmetic overflow (`checked_add`/`sub`/`mul_scalar`) and
division by a zero scalar are checked and typed, never a panic.

## Incomplete / explicit restrictions (honest gaps)

- Only `pt, in, pc, cm, mm, bp, sp` are supported, per the assignment. `em`,
  `ex`, `dd`, `cc`, `px`, and everything else are `UnsupportedUnit`, not
  approximated.
- No dimension-by-dimension multiplication or division (`\dimexpr 10pt *
  2pt` or `10pt/2pt`) — both are `TypeMismatch`. Real TeX's `\dimexpr` does
  not support these either in a way that stays a dimension, so this is a
  deliberate restriction, not an oversight.
- A top-level expression that evaluates to a dimensionless scalar (e.g. just
  `2 + 2`) is `TypeMismatch`, not an implicit `0pt` or bare number.
- Truncation-toward-zero unit conversion matches real TeX's well-known
  quirks (verified above) but this crate does **not** replicate TeX's exact
  `xn_over_d`/remainder-correction bit pattern for every possible input;
  it uses one documented, deterministic `i128` rational truncation instead.
  This was not cross-checked against a running TeX/pdftex binary (none is
  used by this project), only against hand/Python-computed exact fractions.
- Control-sequence names accept any Unicode alphabetic character (tested
  with e.g. `\Länge`) but not digits or combining marks after the backslash,
  matching TeX's own letters-only control-word rule.

## Interface changes and required consumer actions

None — this is a new, standalone additive crate. No existing crate's files,
types, or public API were touched. A consumer that wants exact TeX dimension
math should depend on `flashtex-tex-calc` directly and cross at the boundary
via `Sp::to_style_pt` / `Sp::try_from_style_pt`; no changes to
`document-style` are required or were made.

## Validation

```
cd crates/tex-calc
cargo build                              # clean
cargo test                                # 58 unit/integration tests + 1 doctest, all pass
cargo clippy --all-targets -- -D warnings # clean
cargo fmt --check                         # clean
```

rustc/cargo 1.98.1 (from `rustup`), at commit
`021cf9c247c16174c1b3e098d91d90bb0946fe10` on
`agent/daniel-calc/tex-calc`.

## Peer revisions reviewed and adaptations

None. This crate has no dependency on any peer's in-flight work beyond the
already-merged `crates/document-style` (`Pt`, consumed as documented above).
No coordination scripts were run; no other agent's files were read or
touched beyond this document and `crates/tex-calc`.

## Next action

Await Commander/integration review of the typed contract above (in
particular: is a new crate with only a `Pt` boundary conversion — versus
`document-style` itself growing an exact-`Sp` type — the intended shape for
downstream layout consumers?). No further work planned on this lane
unless requested.
