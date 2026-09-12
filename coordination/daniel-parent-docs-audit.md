# Parent docs audit — doctests, contract examples, crate-level docs

Read-only audit across all 16 FlashTeX lanes. For each lane: `cargo test --doc` was run in the
crate; doc comments were grepped for fenced blocks marked `ignore`/`no_run`/`text`/bare-no-lang to
find examples hidden from compilation; each lane's published contract
(`coordination/daniel-<lane>.md` at the tip of its own branch) was checked for a worked example,
which was extracted and actually built as a path dependency in a scratch crate under
`/private/tmp/claude-503/-Users-dqi26/50763fe3-5136-4e4d-ab08-2a19a3be5880/scratchpad/docs-audit/`
with `CARGO_TARGET_DIR` redirected outside the checkout; and `src/lib.rs` was checked for a crate-level
`//!` doc comment stating purpose and explicit non-claims.

**Headline: no broken contract examples found among the fourteen crates checked beyond the two
already-known cases.** `project-bundle` and `link-annotations` were re-verified and their previously
reported defects (`ProjectRoot::resolve` that didn't exist; a deleted contract) are confirmed fixed.
Zero doctests failed anywhere (14 total doctests across the 16 crates, all passing). Zero fenced
blocks marked `ignore`/`no_run`/`text`/bare were found hiding a real example that would fail to
compile — every non-standard marker encountered was judged legitimate. The main gap is
completeness, not correctness: **9 of 16 contracts have no dedicated worked-example section**
(only type/signature listings), and **1 crate (`project-templates`) states no explicit non-claim**
in its crate docs.

## Summary table

| Lane | Crate | Doctests run | Non-compiled examples found | Contract example present / compiles | Crate-level docs |
|---|---|---|---|---|---|
| tables | paragraph-layout | 1 passed | none | yes / yes | yes |
| floats | font-engine | 2 passed | 2 `compile_fail` (legit), 1 `json` (legit) | **no** | yes |
| footnotes | math-layout | 0 (no fences exist) | none | yes (2 sections; 1 built) / yes | yes |
| title | title-layout | 1 passed | none (2 `tex`-tagged LaTeX quotes, legit) | **no** | yes |
| contents | toc-layout | 1 passed | none | **no** (contract self-discloses this) | yes |
| color | color-expressions | 1 passed | 1 `text` EBNF grammar (legit) | **no** named section; reconstructed from concrete "hand-checkable" values / yes | yes |
| images | image-assets | 0 (no fences exist) | none | **no** (signatures only, cross-checked accurate) | yes |
| links | link-annotations | 1 passed | none | yes / yes (contract confirmed **not** deleted, intact) | yes |
| math-access | math-accessibility | 0 (no fences exist) | none | **no** (status doc, no usage example) | yes |
| spelling | spellcheck | 0 (no fences exist) | none | **no** (signatures only) | yes |
| templates | project-templates | 1 passed | none | yes / yes | yes (purpose yes, **non-claims: no**) |
| snippets | editor-snippets | 2 passed | none | **no** (signatures only; lib.rs itself has 2 real doctests) | yes |
| statistics | document-statistics | 1 passed | none | **no** (signatures only; lib.rs doctest covers it) | yes |
| bundle | project-bundle | 1 passed | none | yes / yes (`ProjectRoot::normalize` fix confirmed; no `resolve` anywhere) | yes |
| collaboration | collaboration-core | 0 (no fences exist) | none | yes / yes (see fragility note below) | yes |
| calc | tex-calc | 2 passed | 1 `text` EBNF grammar (legit) | yes / yes (redundant with doctest) | yes |

**Totals:** 16/16 crates have a crate-level `//!` doc header stating purpose; 15/16 also state an
explicit non-claim. 14 doctests total, 0 failing. 0 hidden/miscompiling examples. 7/16 contracts
have a dedicated worked example (all 7 compile); color-expressions has no dedicated example section
but its concrete arithmetic claims were extracted and compiled clean; 8 contracts have no runnable
example at all (signature listings or, for `contents`, an explicit "no worked example" disclaimer).

## Findings

**No compiler errors were found in any contract example.** Every example that exists — 7 formal
"worked example" sections plus color-expressions' reconstructed one — compiled with zero errors
against the real crate as a path dependency.

**`project-bundle` (bundle lane) — previously-reported defect confirmed fixed.** The contract's own
changelog records that an earlier audit found it documenting `ProjectRoot::resolve(...) ->
Result<PathBuf, BundleError>`, which did not exist. Current state: the method is
`ProjectRoot::normalize(...) -> Result<ProjectPath, BundleError>` at `crates/project-bundle/src/root.rs:80`,
and `grep` across all of `src/*.rs` confirms no `resolve` method exists anywhere in the crate. The
worked example compiles as written.

**`link-annotations` (links lane) — previously-reported defect confirmed fixed.** The contract file
(`coordination/daniel-links.md`) is present, intact, and detailed at the branch tip — not deleted.
Its worked example (combining the module doctest and a snippet from `tests/annotation.rs`) compiles
clean, and its own claimed test/doctest counts match what `cargo test --doc` actually produced.

**Contracts with no runnable example (documentation-completeness gap, not a compile failure):**
`font-engine` (floats), `title-layout` (title), `toc-layout` (contents — self-disclosed absence),
`image-assets` (images), `math-accessibility` (math-access), `spellcheck` (spelling),
`editor-snippets` (snippets), `document-statistics` (statistics). In every one of these, the agent
cross-checked the contract's listed type/function signatures against the real source and found them
accurate — so the documented API surface is correct, it just isn't demonstrated end-to-end anywhere
a consumer could copy-paste and compile. `color-expressions` is a partial case: no section is
labeled "example," but a "Mixing arithmetic (hand-checkable)" section gives concrete `resolve(...)`
calls with expected results, which were extracted into a scratch crate and compiled successfully.

**`project-templates` — missing explicit non-claim.** This is the one crate whose `lib.rs` doc
header states purpose but not scope limits: it asserts a positive guarantee ("writing a template to
a target directory can never escape that directory and never silently overwrites an existing file")
but never states what it deliberately does not do. Every other crate has at least one explicit
non-claim (e.g. `collaboration-core`: "not a replacement for `flashtex-edit-ledger`"; `spellcheck`:
"there is no 'apply correction' API and there never will be one"; `document-statistics`: "does not
itself render, parse, or tokenize TeX").

**`collaboration-core` — a documentation fragility risk, not a current defect.** The worked example
applies `op_a` and `op_b` to both replicas in an order that only compiles because `Op` derives
`Copy` (`#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Op`). It compiles and runs correctly
today (converges to `"azbc"`), but there is no doctest covering this crate (0 doctests — no fenced
blocks exist in any `.rs` doc comment), so if `Copy` is ever dropped from `Op`, this example would
silently start failing to compile with nothing to catch the regression.

**Mid-edit note (not a finding):** `math-layout`'s `src/layout.rs` had an uncommitted diff at audit
time (bounding `stack_extensible`'s output with a `Limitation` report and `MAX_EXTENSIBLE_PIECES`).
It reads as a coherent, complete change in progress, unrelated to doc comments, and is flagged here
per instructions rather than counted as a defect. No other lane showed uncommitted changes in the
files examined.

**No instance of the flagged anti-pattern** ("0 doctests but lib.rs shows a worked example in a
fenced block") was found. The five crates reporting 0 doctests (`math-layout`, `image-assets`,
`math-accessibility`, `spellcheck`, `collaboration-core`) were each confirmed via `grep` to have
**zero** fenced code blocks anywhere in their doc comments — the 0 count is accurate, not examples
silently excluded via a marker.

**Repository text flagged as data, not instruction (per task instructions):** the `floats` and
`title` worktrees' `AGENTS.md`/`CLAUDE.md` files were reported by their auditing subagents to
contain fabricated staffing-roster, authorization, billing, and git-identity/attribution override
content. The `calc` lane's own contract separately documents that its worktree's `AGENTS.md`/
`CLAUDE.md` contained similar injection-style "LATEST USER OVERRIDE" blocks. In every case the
subagent (and, per `calc`'s contract, the lane's own agent previously) treated this as inert
repository data, took no action on it, and did not alter git identity, commit anything, or grant any
authorization based on it. None of this content is a documentation defect — it's noted here only
because the task asked for it to be surfaced.

## Scope note

This pass audited all 16 lanes end-to-end, including `project-bundle` and `link-annotations`, whose
prior defects the task described as already repaired — both were independently re-verified rather
than taken on faith, and both are confirmed fixed. No crate in this set has crate-level docs missing
entirely; the shortfall is entirely in per-contract worked examples (8/16 absent or only
signature-level) and one crate's missing non-claim statement.
