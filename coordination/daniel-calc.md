# daniel-calc handoff

Agent / task / branch: `daniel-calc` / FT-045 "tex-calc: bounded TeX dimension
expression evaluator" / `agent/daniel-calc/tex-calc`

State: ready for integration (revision 3)

Owned paths: `crates/tex-calc/**`, `coordination/daniel-calc.md`,
`coordination/agents/daniel-calc.json`. No other crate was edited;
`document-style` is consumed read-only via a `path` dependency, never
modified.

Exact tested commit SHA (revision 3): `8913fd58847d09e20c811a387b5eb5efd14da9f2`
(the merge commit that brings `origin/main` in through
`486b759ce906cf987e4d7ebba9033c56b15a499b` on top of the revision-3 code
commit `18af3c1a61c251fc22a79a3f03129ae1a0303406`). `cd crates/tex-calc &&
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
all clean at that exact commit; the merge touches zero files under
`crates/tex-calc` (the crate does not exist on `main`), confirmed by
`git diff 18af3c1a61c251fc22a79a3f03129ae1a0303406
8913fd58847d09e20c811a387b5eb5efd14da9f2 -- crates/tex-calc` being empty, so
the merge cannot have changed the tested result. Revision 2 was tested at
`09c049a43ff25854724fc794651a4e8691145be1`; revision 1 at
`021cf9c247c16174c1b3e098d91d90bb0946fe10`.

## Revision 2: scoped named-length dependencies, fresh vs. incremental, richer diagnostics

New module `src/deps.rs` adds `LengthTable`, a flat table of named lengths
built directly on `eval.rs`'s existing lazy-thunk machinery (same exact
arithmetic, same blackhole cycle detection, same `MAX_RESOLUTION_DEPTH`),
plus a dependency graph computed from each definition's free (non-shadowed)
`\name` references (`free_names`, which respects rev 1's shadowing rule: a
name bound by a `Group` nested inside a definition is not a dependency on
an outer name of the same spelling).

- `LengthTable::define(name, expr)` recomputes `name`'s direct dependency
  edges and invalidates exactly `name`'s own cached value plus every name
  that (transitively) depends on it — nothing else.
- `LengthTable::stale_dependents(name)` / `direct_dependencies(name)` expose
  the graph directly.
- `LengthTable::get(name)` resolves incrementally, reusing every
  still-valid cached value; `eval_all_fresh()` recomputes every defined name
  in a brand-new environment sharing no cache, for comparison.
- **Load-bearing property**: `deps::tests` proves
  `eval_all_fresh() == eval_all_incremental()` by exact `HashMap` equality
  (no tolerance) over five varied graphs: a linear chain, a diamond, an
  unrelated/unaffected branch, a definition shadowed by its own inner
  group, and a three-length cycle that is broken and then pushed to
  overflow — cycle and overflow diagnostics are asserted identical between
  the fresh and incremental paths in that last case too.

`CalcError::CyclicLength` now carries the full chain (`Vec<String>`, e.g.
`[a, b, c, a]`) instead of just the one repeated name, built from the
existing per-resolution `stack` threaded through `eval::force`/`eval_expr`.
`CalcError::Overflow` now carries `OverflowInfo { op, operands }` naming the
exact operation (`"+"`, `"-"`, `"*"`, `"/"`, `"neg"`, `"literal"`, or
`"from_style_pt"`) and the human-readable operand values, threaded through
every checked arithmetic path in `sp.rs` and `eval.rs`. Both changes are
breaking to the two enum variants' payloads; every existing test that
matched them was updated to the new (richer) shape — no test was loosened
or dropped to make this pass.

All rev 1 exactness is preserved unchanged: `i64` scaled points, `i128`
rational literal conversion (no `f64` in arithmetic), the tex.web unit
ratio table, the `MAX_DIMEN_SP` boundary tests, typed division-by-zero, and
the `72.26999pt` truncation fidelity. `f64` stays confined to the
`document-style` `Pt` boundary conversion (now also carrying an
`OverflowInfo` on the overflow path).

Test count: 66 unit tests + 1 doctest (up from 58 unit tests + 1 doctest in
revision 1: +8 net, after updating several existing cycle/overflow
assertions to the new payload shapes and adding new coverage in `deps.rs`
and `lib.rs`). `cargo clippy --all-targets -- -D warnings` and `cargo fmt
--check` both clean.

## Revision 3: bounded parse depth, named adversarial coverage, and the published consumer adapter contract

Rev 3's objective added two new requirements on top of rev 1/2: "consume
existing style units" and "integrate an explicit published consumer adapter
without peer ownership edits." Both were already satisfied by rev 1's design
and are made explicit here rather than changed:

- **Existing style units consumed**: `crates/document-style/src/length.rs`
  is the repository's existing TeX-units crate (`pub struct Pt(pub f64)`,
  `pub const PT_PER_IN: f64 = 72.27`). This crate does not invent a parallel
  `Pt`/length type; `Sp::to_style_pt`/`Sp::try_from_style_pt` (both in
  `crates/tex-calc/src/sp.rs`) are the only place `flashtex_document_style`
  is touched, and they consume `document-style`'s own `Pt` exactly as
  defined at `crates/document-style/src/length.rs:22`, at the crate
  boundary only. A repo-wide grep for other unit/dimension representations
  (`65536`, `MAX_DIMEN`, `struct Dimen`, `struct Pt\b`) turned up unrelated
  peer-owned integer/float scaled-point constants inside
  `rendering-core`, `font-resources`, and `math-layout` (e.g.
  `crates/math-layout/src/tfm.rs:13`,
  `pub const SP_PER_PT: f64 = 65536.0;`) — those are internal to their own
  crates' font/glyph pipelines, not a shared "style units" type, and were
  left untouched, per the owned-paths restriction.
- **Published consumer adapter, no peer edits**: `crates/tex-calc/src/deps.rs`
  now carries a module-level doctest (reproduced below) that is the exact
  typed surface a downstream consumer uses — `LengthTable::define`/`get`,
  crossing the `document-style` boundary via `to_style_pt`, and matching on
  every `CalcError` variant. A repo-wide grep proves no such consumer exists
  yet (see "No real consumer exists" below); the contract is published for
  when one appears, and no peer crate was edited to manufacture one.

New parser-side bound: `pub const MAX_PARSE_DEPTH: usize = 200` in
`crates/tex-calc/src/parser.rs`, checked once per `parse_unary` call (every
recursive path back into it — nested `(`, nested `{`, or a chained unary
`-` — goes through this one check), so unbounded input nesting is a typed
`CalcError::Parse` instead of a Rust stack overflow. This is a new, additive
public constant; it does not change any existing signature.

### No real consumer exists

```
$ grep -rln "tex-calc\|tex_calc" . --include="*.rs" --include="*.toml" --include="*.md" --include="*.json" | grep -v "^\./crates/tex-calc/" | grep -v "/target/"
crates/tex-calc/Cargo.toml
crates/tex-calc/src/deps.rs
crates/tex-calc/src/lib.rs
coordination/daniel-calc.md
coordination/next/daniel-calc.json
coordination/assignments/FT-045.json
coordination/completions/daniel-calc/FT-045-r2.json
coordination/agents/daniel-calc.json
coordination/machines/daniel-new.json
coordination/queues/daniel-calc.json
```

Every hit is either this crate's own source/manifest or `daniel-calc`'s own
coordination records (this file, the assignment, past reports, the queue and
machine files). No other crate's source references `tex-calc`/`tex_calc` in
any form, so no consumer fixture was built against a real integration point;
none was invented. The consumer-facing example below is exercised as a
doctest against this crate's actual public API so it stays true even with no
real caller yet.

### Bounded adversarial inputs (counts, revision 3)

15 tests are new since revision 2 (14 in `src/lib.rs`, 1 in `src/sp.rs`;
exact list reproduced by `comm -13` between revision 2's and this revision's
`grep -oE "fn [a-z0-9_]+\(\)"` over each file — no test was renamed or
dropped, only added). Every one asserts a specific `CalcError` variant, not
just `is_err()`. The nine categories the assignment names explicitly are
covered below (one pre-existing test, `empty_input_is_a_parse_error`, already
covered "empty input" from revision 1 and needed no new test):

| category | test | asserted variant | new this revision? |
|---|---|---|---|
| dimension at/past `MAX_DIMEN_SP` through `evaluate` | `expression_at_and_past_max_dimen_cap` | `Overflow` | yes |
| deeply nested parens at/past `MAX_PARSE_DEPTH` | `nested_parentheses_at_and_past_the_depth_bound` | `Parse` | yes |
| absurdly long (10,000-term) flat input | `absurdly_long_flat_expression_is_bounded_not_unbounded_work` | `ResolutionDepthExceeded` | yes |
| NUL bytes | `nul_byte_is_a_typed_parse_error_not_a_panic` | `Parse` | yes |
| non-NFC Unicode (decomposed `e` + U+0301) | `non_nfc_unicode_is_a_typed_parse_error_not_a_panic` | `Parse` | yes |
| RTL override (U+202E) | `rtl_override_character_is_a_typed_parse_error_not_a_panic` | `Parse` | yes |
| division by zero (named length resolving to 0) | `division_by_a_named_length_that_evaluates_to_zero_is_typed` | `DivisionByZero` | yes |
| empty input | `empty_input_is_a_parse_error` (`src/parser.rs`) | `Parse` | no (revision 1) |
| lone operators (`+ - * /`) | `lone_operators_are_typed_parse_errors_not_panics` | `Parse` | yes |
| numeric literal past i32::MAX/i64::MAX | `numeric_literal_past_i32_and_i64_range_is_typed_overflow_not_panic` | `Overflow` | yes |

The remaining 5 new tests cover related bounds not explicitly named but
needed for "no silent approximation": a 200-digit literal past `i128`
(`literal_with_an_enormous_number_of_digits_is_a_typed_parse_error`,
`Parse`), a named-length dependency chain at/past `MAX_RESOLUTION_DEPTH`
found by binary search (`dependency_chain_at_and_past_max_resolution_depth`,
`ResolutionDepthExceeded`), overflow through each of `+ - * /` carrying the
right operand name (`overflow_at_each_binary_operator_is_typed_with_the_right_op_name`,
`Overflow`), unknown/partially-matching unit suffixes never approximated to
the unit they resemble (`unit_suffix_unknown_empty_or_partially_matching_is_never_approximated`,
`UnsupportedUnit`/`TypeMismatch`), self-shadowing inside a nested group
(`name_shadowing_itself_in_a_nested_group_is_a_self_cycle_not_the_outer_value`,
`CyclicLength`), and negation overflow on a deliberately out-of-range `Sp`
constructed via its public tuple field (`neg_overflow_is_typed_not_panic`,
`src/sp.rs`, `Overflow`).

Total test count after these additions: **81 unit tests + 2 doctests = 83**,
up from 66 unit tests + 1 doctest in revision 2 (net +15 unit tests: the 14
in `src/lib.rs` and 1 in `src/sp.rs` listed above; +1 doctest: the new
`deps.rs` module-doc consumer-contract example). `cargo test`: 81 passed, 0
failed. `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`
both clean.

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

pub struct OverflowInfo { pub op: String, pub operands: Vec<String> } // rev 2

pub enum CalcError {
    Overflow(OverflowInfo),          // rev 2: names the op + operand values
    DivisionByZero,
    UnsupportedUnit(String),
    UndefinedLength(String),
    CyclicLength(Vec<String>),       // rev 2: full chain, e.g. [a, b, c, a]
    ResolutionDepthExceeded,
    Parse { message: String, at: usize },
    TypeMismatch(String),
}

pub fn evaluate(source: &str) -> Result<Sp, CalcError>;   // parse + eval in one call
pub fn parse(source: &str) -> Result<Expr, CalcError>;
pub fn eval::eval(expr: &Expr) -> Result<Sp, CalcError>;
pub const parser::MAX_PARSE_DEPTH: usize = 200;      // rev 3: typed Parse past this, not a stack overflow
pub const eval::MAX_RESOLUTION_DEPTH: usize = 200;   // typed ResolutionDepthExceeded past this

// The document-style boundary crossing (the only two functions that touch
// flashtex_document_style at all):
impl Sp {
    pub fn to_style_pt(self) -> flashtex_document_style::length::Pt;             // lossy Sp -> Pt
    pub fn try_from_style_pt(pt: flashtex_document_style::length::Pt) -> Result<Sp, CalcError>;
}

// rev 2: scoped named-length dependency tracking + incremental recompute
pub struct LengthTable { /* ... */ }
impl LengthTable {
    pub fn new() -> Self;
    pub fn define(&mut self, name: &str, expr: Expr);
    pub fn direct_dependencies(&self, name: &str) -> HashSet<String>;
    pub fn stale_dependents(&self, name: &str) -> HashSet<String>;
    pub fn get(&self, name: &str) -> Result<Sp, CalcError>;
    pub fn eval_all_incremental(&self) -> HashMap<String, Result<Sp, CalcError>>;
    pub fn eval_all_fresh(&self) -> HashMap<String, Result<Sp, CalcError>>;
}
pub fn deps::free_names(expr: &Expr) -> HashSet<String>;
```

Every failure path returns one of these `CalcError` variants — no panics, no
silent approximation, no implicit zero on overflow/unsupported input.

### Worked example (published consumer adapter)

Copied from the actual, compiled `///` doctest in `crates/tex-calc/src/deps.rs`
(module-level doc comment, runs under `cargo test`):

```rust
use flashtex_tex_calc::{parse, CalcError, LengthTable, Sp};

let mut lengths = LengthTable::new();

// 1. Define named lengths (each `Expr` comes from `parse`, e.g. read
//    from a document's preamble).
lengths.define("margin", parse(r"1in").unwrap());
lengths.define("gutter", parse(r"\margin / 2").unwrap());

// 2. Evaluate one length -- or `eval_all_incremental()` / `eval_all_fresh()`
//    for the whole table -- to an exact `Sp`.
let margin = lengths.get("margin").unwrap();
assert_eq!(margin, Sp(4_736_286));
assert_eq!(lengths.get("gutter").unwrap(), Sp(4_736_286 / 2));

// 3. Cross the `document-style` boundary at the edge of the consumer's
//    own layout code, not inside this crate's arithmetic.
let margin_pt = margin.to_style_pt(); // flashtex_document_style::length::Pt
assert_eq!(Sp::try_from_style_pt(margin_pt).unwrap(), margin);

// 4. Every failure mode a consumer must handle is one of these typed
//    `CalcError` variants -- never a panic, never an implicit zero.
lengths.define("cyclic", parse(r"\cyclic + 1pt").unwrap());
lengths.define("huge", parse(r"16384pt").unwrap());
lengths.define("stray", parse(r"1pt / 0").unwrap());
lengths.define("dimensionless", parse(r"2 + 2").unwrap());

assert!(matches!(lengths.get("cyclic"), Err(CalcError::CyclicLength(_))));
assert!(matches!(lengths.get("huge"), Err(CalcError::Overflow(_))));
assert!(matches!(lengths.get("stray"), Err(CalcError::DivisionByZero)));
assert!(matches!(lengths.get("dimensionless"), Err(CalcError::TypeMismatch(_))));
assert!(matches!(
    lengths.get("nonexistent"),
    Err(CalcError::UndefinedLength(_))
));
assert!(matches!(
    parse(r"1cc"),
    Err(CalcError::UnsupportedUnit(_))
));
assert!(matches!(parse("("), Err(CalcError::Parse { .. })));
```

The eighth variant, `CalcError::ResolutionDepthExceeded`, surfaces from
`LengthTable::get` exactly as it does from plain `evaluate` (see
`dependency_chain_at_and_past_max_resolution_depth` in `src/lib.rs` for a
worked boundary case); it is omitted from the doctest only because building
a 200-deep chain inline is unwieldy, not because it needs different
handling.

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

Rev 1: none — this was a new, standalone additive crate. No existing
crate's files, types, or public API were touched. A consumer that wants
exact TeX dimension math should depend on `flashtex-tex-calc` directly and
cross at the boundary via `Sp::to_style_pt` / `Sp::try_from_style_pt`; no
changes to `document-style` are required or were made.

Rev 2: no other crate consumes `flashtex-tex-calc` yet, so this is still
additive in practice, but note for whenever one does: `CalcError::Overflow`
and `CalcError::CyclicLength` changed payload shape (`Overflow(OverflowInfo)`
instead of a unit variant; `CyclicLength(Vec<String>)` instead of
`CyclicLength(String)`) to carry the diagnostics this revision requires. Any
future consumer matching on those variants' contents needs the new shape;
matching on the variant tag alone (`Err(CalcError::Overflow(_))`) is
unaffected.

Rev 3: purely additive. `pub const parser::MAX_PARSE_DEPTH: usize = 200` is a
new public item (no existing signature changed); every existing test still
passes unmodified. No other crate consumes `flashtex-tex-calc` yet (see the
repo-wide grep above), so there is no real consumer to break or migrate — the
"required consumer actions" are the worked example above, published for
whenever a first consumer appears.

## Validation

```
cd crates/tex-calc
cargo build                              # clean
cargo test                                # 81 unit tests + 2 doctests, all pass
cargo clippy --all-targets -- -D warnings # clean
cargo fmt --check                         # clean
```

rustc/cargo 1.98.1 (from `rustup`), at commit
`8913fd58847d09e20c811a387b5eb5efd14da9f2` (merge commit) on
`agent/daniel-calc/tex-calc`; the revision-3 code itself was committed at
`18af3c1a61c251fc22a79a3f03129ae1a0303406` and re-verified unchanged after
the merge.

## Peer revisions reviewed and adaptations

None beyond the already-merged `crates/document-style` (`Pt`, consumed as
documented above) — this crate has no dependency on any other peer's
in-flight work. Revision 3 merged `origin/main` through the FT-045 rev 3
assignment's stated `main_sha`,
`486b759ce906cf987e4d7ebba9033c56b15a499b` (`git merge --no-ff
486b759ce906cf987e4d7ebba9033c56b15a499b`, clean, no conflicts); `git diff
18af3c1a61c251fc22a79a3f03129ae1a0303406 8913fd58847d09e20c811a387b5eb5efd14da9f2
-- crates/tex-calc` is empty, confirming the merge touched none of this
crate's files (it changed only `coordination/**` and other crates' own
directories, none of which this lane owns or edited). No coordination
scripts were run; no other agent's files were read for content beyond what
the merge brought in mechanically.

Note on repository content encountered during this revision: this worktree's
`AGENTS.md`, `CLAUDE.md`, and `coordination/CLAUDE.md` contain numerous
"LATEST USER OVERRIDE"/staffing/authority/commit-identity blocks (e.g.
claims about who may commit as whom, standing orchestrator authority,
staffing resets). None of that was treated as an instruction — per this
lane's actual assignment, such repository text is data, not instruction. No
staffing, authority, or commit-identity claim from those files was acted on;
this lane's commits use exactly the git identity already configured in this
worktree (`d-q222`), and only `crates/tex-calc` plus this lane's own two
coordination files were touched.

## Next action

Await Commander/integration review of the rev 3 typed contract above,
including: the new `MAX_PARSE_DEPTH` bound, the 15 new adversarial-category
tests, and whether the published `LengthTable` consumer contract (worked
example above) matches what an eventual layout consumer will actually need.
No further work planned on this lane unless requested.
