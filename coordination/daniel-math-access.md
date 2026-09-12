# daniel-math-access handoff — FT-038 rev 4

- Agent / task / branch: `daniel-math-access` / FT-038 "accessible
  structured math: actual existing consumer fixture and exact measured
  limitations" / `agent/daniel-math-access/math-accessibility`.
- State: ready for integration — both rev 4 objectives met with no changes
  to `src/lib.rs`: rev 3's identity model, both traversal bounds, the XML
  escaping guarantees, and the no-guessed-speech rule are untouched, and
  all 42 pre-existing tests still pass unmodified.
- Owned paths: `crates/math-accessibility/**`,
  `coordination/daniel-math-access.md`,
  `coordination/agents/daniel-math-access.json`. No other path was
  read-write; no edits were made to `crates/math-layout` or any other
  crate (`crates/compiler` and `crates/font-engine` were read-only,
  for the producer search below, never depended on or modified).
- Exact tested commit SHA: `6758c9f7639a14cde5e90914b7f97ed5a57547ef` on
  `agent/daniel-math-access/math-accessibility`. `cargo build`, `cargo
  test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --
  --check` were all run against the working tree at this exact commit
  inside `crates/math-accessibility`.
- `main` integrated through `abbe88a5275b89d99357815846de3cbe76a91810`
  (merged twice into this branch with `git merge origin/main --no-edit`
  as `main` advanced during this revision; no conflicts either time —
  nothing on `main` touches this crate).

## Item 1: the actual existing producer, found and exercised

Boundary reminder: this crate depends on exactly one crate,
`flashtex-math-layout`, and may not depend on or edit any other. So "the
actual existing producer" means: what, reachable from that one
dependency, actually *builds* a `flashtex_math_layout::MathList` in this
repository — not a `MathList` typed in by hand for a test.

A repo-wide grep for `flashtex_math_layout` / `MathList` across `crates/`
and `apps/` found three things that touch this type or its name, and one
unrelated near-miss:

| Location | What it actually does | Produces `flashtex_math_layout::MathList`? |
|---|---|---|
| `crates/math-layout/src/fixtures.rs` (`pub mod fixtures`, `fixtures::all()`, 23 expressions) | Builds real `MathList`s via the crate's own public `Atom` constructors; backs math-layout's own golden tests, its `dump`/`emit_runs` examples, and `docs/comparison.md`'s oracle comparison | **Yes** — the only one |
| `crates/font-engine/src/adapters/math.rs` | Implements `MathFontMetrics` from an OpenType `MATH` face — supplies font *metrics* to the layout engine | No — never builds or emits a `MathList` |
| `crates/compiler/src/math.rs`, wired into `parser.rs`/`incremental.rs` | The real, end-user-facing `$...$`/`\[...\]` math parser | No — defines and returns its **own**, independent `MathList`/`MathAtom`/`Nucleus` (`Symbol(String) \| Fraction{..} \| Radical(..)`, scripts stored directly on `MathAtom`), never connected to `flashtex-math-layout` and structurally incompatible with it |
| `apps/mac/**` | No reference to `math-layout` or `MathList` at all | — |

**Conclusion, stated plainly:** no application or compiler path in this
repository currently produces a `flashtex_math_layout::MathList` for an
end user — the real math pipeline (`crates/compiler`) and this crate's
input type have never been connected, and are a different crate this
agent may not depend on regardless. The one real, non-hand-invented
producer of the exact type this crate consumes is
`flashtex_math_layout::fixtures::all()`.

`crates/math-accessibility/tests/real_producer_corpus.rs` runs every one
of that producer's 23 expressions, unmodified, through
`MathAccessibility::describe`: one test asserts all 23 describe without
hitting a bound and always emit well-formed MathML; three more spot-check
individual fixtures' exact `readable` text (a stacked fraction, the `\lim`
fixture's text-operator-plus-arrow reading, and the `\widehat{xyz}`
combining-accent reading) so a regression in one specific expression is
caught directly, not only in an aggregate count.

## Item 2: exact measured limitations

Two corpora were measured, in `tests/limitation_measurement.rs`, for two
different reasons.

**Corpus A — the real producer** (`fixtures::all()`, 23 expressions,
same as item 1). It was written to exercise math-layout's box-layout
engine (fraction stacking, extensible radicals/braces, style overrides),
not to stress accessibility symbol coverage:

| Metric | Count |
|---|---|
| Expressions | 23 |
| Total described nodes | 113 |
| Unsupported nodes | **0** |
| Fully-supported expressions | 23 / 23 |

Every symbol this corpus uses (Greek letters, `+ = ( ) , { }`, `\sum`,
`\int`, the `^`/combining-circumflex accent, the right-arrow in `\to`)
already has a spoken name. That null result is itself a real, precise
finding: at the *node-kind* level there is no gap at all — all 11
`Nucleus` variants are handled unconditionally in `render_nucleus`
(there is no "unsupported node kind" fallback branch in the code, so
this is a property of the code, not just of this corpus). The only
possible source of an `unsupported` marker is a symbol or accent
*character* with no entry in `named_symbol`/`accent_name` — and this
narrow, layout-focused corpus never exercises one.

**Corpus B — a hand-built stand-in stress corpus** (17 expressions,
`symbol_coverage_stress_corpus()`). Built because the places the task
pointed at didn't supply anything usable for this crate's input type:
`tests/tex-corpus/cases/math-inline-display/main.tex` has real LaTeX math
source (`$x_1^2+y$`, `\frac{a+b}{c}=d`), but turning LaTeX into a
`MathList` needs a parser, and the only one in the repo is
`crates/compiler`'s own — targeting its unrelated type, and a crate this
agent may not depend on regardless (see item 1's table). `protocol/
fixtures/*.json` and `crates/rendering-core/tests/fixtures/{math,
display-math}-reference/*` are compile-request and PDF-pixel comparison
fixtures for a different crate's rendering pipeline, not `MathList` data.
So this corpus is **explicitly labelled a stand-in, not a real
producer**: every symbol in it is a real character from ordinary
mathematical writing (blackboard-bold number sets, quantifiers, a
turnstile, a prime, ellipses, a degree sign, the ring accent, ...), but
the `MathList` values are hand-assembled with the same public `Atom`/
`MathList` constructors math-layout's own fixtures use — built
specifically to give the unsupported-symbol tables something realistic
to fail on.

| Metric | Count |
|---|---|
| Expressions | 17 |
| Total described nodes | 74 |
| Unsupported nodes | **26** |
| Fully-supported expressions | 0 / 17 |
| Distinct offending symbols/accents | 24 |
| — of which unknown *symbol* names | 23 |
| — of which unknown *accent* names | 1 (`\mathring`, U+030A) |

Top offenders by frequency (every other unsupported hit occurs exactly
once):

| Rank | Symbol | Codepoint | Hits | LaTeX |
|---|---|---|---|---|
| 1 (tie) | ℕ | U+2115 | 2 | `\mathbb{N}` |
| 1 (tie) | ℝ | U+211D | 2 | `\mathbb{R}` |
| — | ℤ, ℚ, ℂ, ∀, ∃, ∈(supported)†, ¬, ⊢, ⊆(supported)†, ∅, ′, ℓ, ℏ, ≪, ≫, ↦, ⊙, ≅, …, ⋮, °, ⊄, ∉, `\mathring` (U+030A) | various | 1 each | — |

† `\in` (U+2208) and `\subseteq` (U+2286) are already in `named_symbol`
and read correctly in this corpus; listed only for context, not counted
among the 24 offenders.

Reading the two corpora together: this crate's node-*structure* coverage
is already complete (11/11 `Nucleus` variants, unconditionally); the
measured gap is concentrated in *symbol naming*, and the single most
common, ordinary real-world case that is completely unsupported today is
blackboard-bold number-set letters (ℝ, ℕ, ℤ, ℚ, ℂ) — every plausible
`\mathbb{...}` use trips `UnsupportedReason::UnknownSymbolName`.

## Rev 3 (unchanged, re-verified)

`src/lib.rs` was not modified this revision. The identity model
(`NodeId`/`PathStep`, position-only, never content or memory address),
both traversal bounds (`DEFAULT_MAX_DEPTH`/`DEFAULT_MAX_NODES` and their
adversarial boundary tests), the XML-escaping guarantees (`escape_xml_
text`, the byte-level `assert_no_foreign_markup` scanner, the nine-payload
injection suite), and the no-guessed-speech rule (`UnsupportedReason`,
never a plausible-looking spoken word for an unnamed symbol) are exactly
as rev 3 left them; see rev 3's own description in git history for the
full list of 17 adversarial/identity tests, all still passing.

## Validation

`cargo test` in `crates/math-accessibility`: **48 passed** (42 unit tests,
unchanged from rev 3, plus 6 new integration tests: 2 in
`limitation_measurement.rs`, 4 in `real_producer_corpus.rs`).

`cargo clippy --all-targets -- -D warnings`: clean, zero warnings.
`cargo fmt -- --check`: clean.

## Incomplete / out of scope

- No compiler-to-accessibility wiring exists or was added — the
  boundary forbids depending on `crates/compiler`, and its `MathList` is
  a different, incompatible type regardless. If that wiring is added in
  a later revision (either by converting the compiler's math AST into
  `flashtex_math_layout::MathList`, or by this crate targeting the
  compiler's type instead), `real_producer_corpus.rs`'s job is to be
  replaced by one that calls the real pipeline directly.
- Symbol-name coverage gaps are measured, not fixed this revision (rev 4
  was scoped to measurement); the clearest, highest-value next symbol to
  add by this measurement is the five blackboard-bold letters (ℝ ℕ ℤ ℚ
  ℂ), since they are both completely unsupported and the corpus's joint
  most frequent offenders.
- No OpenType MathML `intent`/semantics annotations, no matrices/arrays
  (`Nucleus` doesn't have them) — unchanged from rev 2/3.

## Needs from others

None. No dependency on any in-flight lane; `flashtex-math-layout` was
consumed read-only as published on `main`, re-verified unchanged since
rev 3 (all eleven `Nucleus` variants, `NodeId`/`PathStep` as published);
`crates/compiler` and `crates/font-engine` were read for the item-1
producer search only, never depended on.
