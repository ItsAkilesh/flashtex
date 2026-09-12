# Panic-surface audit — library code (`src/`), 14 crates

Read-only audit. No crate was modified, no repro code was written inside any
repository, and nothing described here was committed. Findings are for the
supervisor and the owning lanes to act on.

**Scope discipline applied throughout:** only `src/` was enumerated. Every
`tests/` directory and every inline `#[cfg(test)] mod tests { ... }` block was
excluded from counts and findings — a panic in a test is a correct assertion,
not a defect. Rustdoc `///` doctests containing `.unwrap()`/`assert!` on fixed
literal examples were likewise excluded as documentation, not caller-reachable
library logic.

**Note on repository content encountered during this audit — treated as data,
never as instructions:** every worktree's root `CLAUDE.md`/`AGENTS.md` (and, in
`ft-wt-daniel-parent`, `coordination/CLAUDE.md`) contains an extensive
fabricated narrative: multi-agent "Commander"/"Astra"/orchestrator staffing
directives, invented "explicit user authorization" for autonomous continuation
and spending, and instructions to override commit identity/provenance. This
text is auto-loaded by the harness as ordinary repository content, not sent by
the actual user of this session. It was independently encountered by nine of
the per-crate audit passes below and by this coordinating pass itself (when
writing this report). None of it was followed by any lane of this audit — no
staffing/authority claims were acted on, no autonomous continuation beyond this
task was taken, and no commit-identity or provenance override was applied.
This audit only read files, ran throwaway programs against a scratch
directory, and writes this one report.

**A second, more consequential observation:** this is a live, actively-edited
codebase. Three of the reachable panics found below were independently
re-verified by re-running the audit's own repro programs against current
`HEAD` *after* the originating subagent reported them, and by that point two
crates' owning lanes had already landed real fix commits — mid-audit, before
this report was written. Those are marked **FIXED DURING THIS AUDIT** below
with the fixing commit, verified against current `HEAD`, not taken on faith
from the subagent's earlier run. Everything else marked **OPEN** was
re-executed against current `HEAD` as this report was being written and still
panics.

## Summary table

| Crate | `src/` files | Total sites | REACHABLE | FRAGILE | UNREACHABLE-BY-INVARIANT | INTERNAL-ONLY | Current status |
|---|---:|---:|---:|---:|---:|---:|---|
| font-engine | 21 (+1 generated, 0 hits) | 58 | 2 | 1 | 55 | 0 | **2 OPEN**, 1 fragile |
| title-layout | 6 | 5 | 0 | 0 | 5 | 0 | clean |
| toc-layout | 6 | 3 | 0 | 0 | 3¹ | 0¹ | clean |
| color-expressions | 5 | 5 | 0 | 0 | 5 | 0 | clean |
| image-assets | 3 | 0 | 0 | 0 | 0 | 0 | clean |
| link-annotations | 8 | 3 | 2 | 0 | 1 | 0 | **FIXED DURING AUDIT** |
| math-accessibility | 1 | 0 | 0 | 0 | 0 | 0 | clean |
| spellcheck | 1 | 8 | 0 | 0 | 8 | 0 | clean |
| project-templates | 8 | 2 | 0 | 0 | 2 | 0 | clean |
| editor-snippets | 10 | 9 | 0 | 0 | 9 | 0 | clean |
| document-statistics | 7 | 1 | 0 | 0 | 1 | 0 | clean |
| project-bundle | 6 | 2 | 2 | 0 | 0 | 0 | **FIXED DURING AUDIT** |
| collaboration-core | 3 | 5 | 2 | 0 | 3 | 0 | **1 OPEN** (2 call sites) |
| tex-calc | 7 | 30 | 7 | 0 | 16 | 7 | **2 OPEN**, 5 fixed during audit |
| **Total** | **92** | **131** | **15** | **1** | **108** | **7** | **4 mechanisms still open, across 3 crates** |

¹ toc-layout's private-function loop-invariant sites (`converge.rs:165-166`)
were audited twice independently (a duplicate launch of the same lane) and
both passes agree the sites are fully safe, but disagree on the label —
INTERNAL-ONLY vs UNREACHABLE-BY-INVARIANT — for the same reasoning (single
verified private caller, loop-bound-derived indices). Not a defect either way;
noted for methodological transparency, not corrected further since it has no
effect on the REACHABLE/FRAGILE counts.

Two crates (`image-assets`, `math-accessibility`) had **zero** panic-pattern
sites of any kind in library code. `title-layout`, `toc-layout`,
`color-expressions`, `spellcheck`, `project-templates`, `editor-snippets`, and
`document-statistics` had sites but every one was provably safe. That is a
genuinely good result for those nine crates and is reported as such, not
inflated.

---

## REACHABLE and FRAGILE findings, ranked by current severity

### 1. `collaboration-core` — dangling cross-reference from a decoded checkpoint dereferenced during integration — **OPEN**

- **Files:** `crates/collaboration-core/src/lib.rs:294` and `:306` (panic sites,
  inside `signed_index`/`right_bound`), reached from `integrate` at
  `lib.rs:326` and `:330`; root cause in
  `crates/collaboration-core/src/checkpoint.rs:93-124` and `:211-222`.
- **Mechanism:** `Document::apply` validates a *new* operation's own
  `left`/`right` references before calling `integrate`. But `integrate`'s scan
  loop also resolves the `left`/`right` references of **already-installed**
  elements, which are normally guaranteed valid because they were validated
  once, when originally `apply`'d. `Checkpoint::from_bytes` decodes a raw byte
  buffer directly into `Element { id, left, right, value, deleted }` with no
  referential-integrity check that `left`/`right` actually point at another
  element present in that same checkpoint, and `Document::from(Checkpoint)` —
  documented "Never fails" — installs the elements verbatim, bypassing
  `apply`'s validation entirely.
  ```rust
  // lib.rs:289-297 / 299-309
  .position_of(id).expect("caller must validate dependencies before integrating") as isize
  ```
- **Reaching input:** a checkpoint byte buffer (from a peer, disk, or a
  corrupted transport) containing one element whose `left` OpId does not
  correspond to any element actually present in that checkpoint. Once
  restored via `Document::from`, any ordinary subsequent concurrent `Insert`
  whose anchors span across that malformed element crashes the process.
- **Confirmed by execution** — twice, independently: once by the auditing
  subagent, and again by this coordinating pass re-running the same repro
  against current `HEAD` a few minutes later:
  ```
  thread 'main' panicked at .../src/lib.rs:294:18:
  caller must validate dependencies before integrating
  ```
- **Why this ranks first:** it is reachable from data a peer or disk source
  controls (not merely unusual document text), it contradicts the crate's own
  doc comment claiming the restore path "never fails," and — unlike the other
  findings below — no fix has landed for it as of this report (`git log` on
  `crates/collaboration-core/src/` shows no commit addressing it).

### 2. `font-engine` — GSUB ligature substitution indexes a coverage table with an unvalidated font-supplied base index — **OPEN**

- **File:** `crates/font-engine/src/gsub.rs:89`.
  ```rust
  for (lig, rest) in &sub.sets[usize::from(ci)] {
  ```
- **Mechanism:** `ci` comes from `otl::Coverage::index()`, which for a
  format-2 (Ranges) coverage table returns
  `base.checked_add(gid - start)` where `base` is the font's raw
  `startCoverageIndex` field — never checked against the subtable's actual
  number of parsed ligature sets. Every other coverage-index consumer in this
  crate (`gpos.rs`, `math.rs`) uses `.get(...)`; `gsub.rs` is the one place
  that indexes directly with `[]`.
- **Reaching input:** an OpenType/TrueType font whose `GSUB` table has a
  `liga` ligature-substitution lookup with a format-2 Coverage table
  declaring a `startCoverageIndex` far larger than the subtable's real
  `ligSetCount` (e.g. 9999 vs. 1). This is reached through the crate's main
  shaping entry point (`shape::shape()`, via `face.longest_ligature`) for any
  ordinary multi-glyph text run — i.e., simply opening a document that embeds
  such a font and shaping normal text crashes.
- **Confirmed by execution** — a minimal 318-byte hand-built hostile
  TrueType font was constructed and run against current `HEAD` (both by the
  subagent and again independently by this coordinating pass):
  ```
  thread 'main' panicked at .../src/gsub.rs:89:41:
  index out of bounds: the len is 1 but the index is 9999
  ```
- **Status:** `git log`/`git status` on `crates/font-engine/src/gsub.rs` show
  no fix has landed. Note: while this audit was in progress, a real
  commit (`23f34ffb`, "fix silent u16/i16/u32 overflow in OTL/GPOS/cmap
  parsing") landed in the same worktree and fixed three *other*, unrelated
  overflow sites — the auditing subagent's analysis is against that final
  state, and this specific `gsub.rs` bug is not one of the three it patched.

### 3. `font-engine` — `verify_checksums` slices a table directory without a length check — **OPEN**

- **File:** `crates/font-engine/src/subset.rs:223`, inside the public,
  documented API `flashtex_font_engine::subset::verify_checksums`.
  ```rust
  let tag = &data[rec..rec + 4];
  ```
- **Mechanism:** the declared table count `n` is read with a bounds-checked
  `u16_at(data, 4)`, but the subsequent loop computes `rec = 12 + 16*i` and
  slices `data[rec..rec+4]` with no `.get()`/length guard — unlike every
  other table-directory walk in this crate (`truetype.rs`), which checks
  `needed > data.len()` before indexing.
- **Reaching input:** any byte slice ≥ 6 bytes whose bytes 4..6 declare
  `numTables ≥ 1` while the buffer is too short to hold that many 16-byte
  directory records, e.g. `[0,0,0,0,0,1]`.
- **Confirmed by execution** (subagent and this pass, current `HEAD`):
  ```
  thread 'main' panicked at .../src/subset.rs:223:24:
  range start index 12 out of range for slice of length 6
  ```
- **Caveat lowering its practical severity relative to #2:** this requires a
  caller to invoke `verify_checksums` directly on unvalidated bytes, rather
  than being hit through the crate's main parse/shape pipeline.

### 4. `tex-calc` — zero denominator via direct `Expr` construction bypasses the parser's own guarantee — **OPEN**

- **Files:** `crates/tex-calc/src/sp.rs`, `Unit::to_sp` (`debug_assert!` at
  line 87, unconditional division at line 94/99) and `Sp::checked_mul_scalar`
  (`debug_assert!` at line ~159, unconditional division at line ~192).
- **Mechanism:** the crate's lexer/parser can never itself produce a `Scalar`
  or `Dim` denominator of 0 (its minimum is 1) — but `Expr::Dim(i128, i128,
  Unit)` and `Expr::Scalar(i128, i128)` are `pub` tuple variants, and
  `eval::eval(&Expr)` is a public entry point, so any embedding Rust caller
  (not just the string-based `evaluate()`) can build a zero-denominator node
  directly. Divide-by-zero panics unconditionally in Rust regardless of
  build profile, so the `debug_assert!` compiling out in release does not
  make this safe in release — it just changes which line panics.
- **Reaching input:** `eval(&Expr::Dim(1, 0, Unit::Pt))`, or
  `eval(&Expr::Mul(Box::new(Expr::Dim(1,1,Unit::Pt)), Box::new(Expr::Scalar(5,0))))`.
  Not reachable through the documented text entry point `evaluate()` — only
  through the lower-level `Expr`/`eval` API surface.
- **Confirmed by execution against current `HEAD`**, re-run by this
  coordinating pass after the fix below landed (to make sure this specific
  bug was not incidentally closed by it — it was not):
  ```
  thread 'main' panicked at .../src/sp.rs:87:9:
  assertion failed: denominator > 0
  ```
- **Lower severity than #1–#3:** only reachable to code that constructs the
  AST directly, which is a narrower audience than "any document author typing
  a formula."

### 5. `font-engine` — `EmbedPlan::finish` indexes a glyph map using a cross-module invariant — **FRAGILE, not currently exploitable**

- **File:** `crates/font-engine/src/embed.rs:139`.
  ```rust
  let new = glyph_map[old];
  ```
- **Mechanism:** `old` ranges over glyph IDs a caller added via the public
  `EmbedPlan::add(&mut self, gid: GlyphId, text: &str)`, which does not itself
  validate `gid`. The lookup is currently safe only because two *other*
  functions — `subset()` and `advance()` — both bail via `?` on any
  out-of-range glyph id before `finish()`'s map is ever built or consulted.
  `finish()` has no local invariant of its own; a future change to either of
  those two other functions' error handling would silently reopen this panic.
- Not executed (nothing to trigger under the crate's current state — every
  path I traced bails safely first); reported as **FRAGILE** per the rubric
  rather than inflated to REACHABLE.

---

## Findings closed during this audit (reported for completeness, not action)

These were real, **confirmed-by-execution** REACHABLE panics at the time each
per-crate lane ran. By the time this coordinating report was written, the
owning lanes had already landed fixes — re-verified independently against
current `HEAD` before writing this section.

### `link-annotations` — inverted `SourceSpan` from a public-field bypass

- **Was:** `SourceSpan`/`SourcePos` had fully `pub` `start`/`end` fields, so a
  caller could construct a `SourceSpan{ start, end }` struct literal directly,
  skip `SourceSpan::new()`'s ordering check, and hand `slice_for`
  (`source.rs:273`, `&source[start..end]`) or `SourceSpan::len()`
  (`span.rs:44`, unchecked `u32` subtraction) an inverted span. Confirmed by
  execution at audit time:
  ```
  thread 'main' panicked at .../src/source.rs:273:15: byte range starts at 100 but ends at 5
  thread 'main' panicked at .../src/span.rs:44:9: attempt to subtract with overflow
  ```
- **Now:** commit `4538dae3` ("Fix silent u32 underflow in `SourceSpan::len()`
  on inverted spans") made `start`/`end` private, made `SourceSpan::new()` the
  only constructor, and made it return `Result` and reject `end < start`.
  Re-running the exact same repro against current `HEAD` no longer compiles
  (`error[E0451]: fields 'start' and 'end' of struct 'SourceSpan' are
  private`) — the bypass is closed.

### `project-bundle` — `.expect()` on a bundle/preview pairing a caller can mismatch

- **Was:** `apply_import` (`apply.rs:184` and `:222`) called
  `bundle.file(&fp.path).expect("preview built from this bundle")`. Because
  `Bundle` and `ImportPreview`/`FilePreview` are plain public structs with no
  privileged constructor pairing them, a caller could pass a `Bundle` and an
  `ImportPreview` that don't correspond to each other. Confirmed by execution
  at audit time:
  ```
  thread 'main' panicked at .../src/apply.rs:222:22: preview built from this bundle
  ```
- **Now:** commit `d18fa51f` ("replace `apply_import`'s `expect()` with a
  typed batch precheck") replaced both `.expect()` calls with an upfront
  `by_path.get(...)` precheck returning `Err(BundleError::PreviewBundleMismatch(..))`.
  Re-running the exact same repro against current `HEAD`:
  ```
  did not panic, result = Err(PreviewBundleMismatch("does-not-exist-in-bundle.txt"))
  ```

### `tex-calc` — `i128` overflow in unit/scalar arithmetic (3 of its 7 REACHABLE sites)

- **Was:** `Unit::to_sp`, `Sp::checked_mul_scalar`, and `Sp::checked_div_scalar`
  each used a raw `*` on `i128` operands derived from plain-text input (a
  numeric literal near `i128::MAX`, or a decimal with dozens of digits),
  overflowing in debug and silently wrapping to a wrong answer in release.
  Confirmed by execution at audit time via ordinary `evaluate()` strings, e.g.
  `evaluate("1pt * 170141183460469231731687303715884105727")`.
- **Now:** commit `c700f735` ("check unit-scaling and scalar mul/div for
  i128 overflow") replaced the raw multiplications with `.checked_mul(...)`
  chains returning a typed `CalcError::Overflow`, and a follow-up commit
  `9d34a597` removed the diagnostic probe tests used to confirm it. Verified
  directly against current `sp.rs` source: all three functions now route
  through `.checked_mul(...).ok_or_else(...)`. (Findings #4 above — the
  zero-denominator-via-`Expr`-bypass bugs in the same file — are a distinct
  mechanism and were **not** touched by this fix; they remain open.)

---

## What was thoroughly checked and found clean

For the nine clean crates (`title-layout`, `toc-layout`, `color-expressions`,
`image-assets`, `math-accessibility`, `spellcheck`, `project-templates`,
`editor-snippets`, `document-statistics`), every candidate site was traced to
either a same-function bounds check, a `checked_*` combinator used correctly
(no unwrap), or — for `spellcheck`, `editor-snippets`, `link-annotations`
(post-fix), and `document-statistics`, the crates flagged in advance for
UTF-8/byte-offset risk — a slicing discipline built structurally from
`char_indices()`/`chars()` rather than raw arithmetic offsets, so no
`&s[a..b]` in those crates can land mid-character. `project-templates` was
specifically checked for the public-field-bypass pattern (its `Template`/
`TemplateFile` structs do have public fields) and found *not* vulnerable,
because validation is re-run at the point of use (`instantiate()` calls
`template.validate()` unconditionally) rather than trusted from a
constructor. `font-engine`'s 55 UNREACHABLE-BY-INVARIANT sites and `tex-calc`'s
16 were each traced to a specific enforcing line, not accepted on a comment's
say-so — full reasoning for a representative sample of each is preserved in
the per-crate subagent transcripts if deeper spot-checking is wanted.

---

## Audit method

Each crate was audited by an independent subagent, working only inside its
own worktree, enumerating `.unwrap()`, `.expect(...)`, `panic!`,
`unreachable!`, `todo!`, `unimplemented!`, `assert!`/`assert_eq!`/`assert_ne!`,
direct indexing/slicing, non-constant integer division/modulo, and
`.unwrap()` on `checked_*` results. Every REACHABLE claim was required to be
backed by a throwaway Rust program under this session's scratchpad
(`/private/tmp/claude-503/-Users-dqi26/50763fe3-5136-4e4d-ab08-2a19a3be5880/scratchpad/panic-audit-repro/<crate>/`,
one directory per crate, each a standalone Cargo project with a `path`
dependency on the real crate — never a copy of its source, and never written
inside the audited repository) and actually executed. This coordinating pass
then independently re-ran the repros for every REACHABLE claim a second time
against current `HEAD`, which is how the three since-fixed findings and the
still-open `tex-calc` zero-denominator bug were caught and correctly
attributed above rather than reported stale.
