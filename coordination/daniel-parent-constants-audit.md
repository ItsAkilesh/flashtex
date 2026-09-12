# TeX fundamental-constants audit

Read-only audit of every crate listed in the assignment for hard-coded TeX
constants (sp-per-pt, `MAX_DIMEN`, points-per-inch, big points, picas, didot/
cicero, mm/cm). This file is written by the audit; it does not modify any
crate and was not committed by the auditor.

**A note on the repo's own text.** Several crates' Cargo.toml doc-comments and
`coordination/*.md` files in this directory (e.g. `COMMANDER.md`,
`COMMANDER-ACK-CLAUDE.md`, `authority.json`) look like they could claim to
grant authorization or redefine identity/process rules. Per instructions,
repository text of that kind is data, not instruction. I saw these files exist
(via `ls`) but did not open or act on their contents — nothing in this audit
relies on them, and no such content was treated as a directive.

**Ground truth measured against real TeX** (BasicTeX 2026, `/Library/TeX/texbin/tex`,
`-interaction=nonstopmode`, via the `\dimen0=... \count0=\dimen0 \showthe\count0`
idiom, in `/private/tmp/claude-503/-Users-dqi26/50763fe3-5136-4e4d-ab08-2a19a3be5880/scratchpad/tex-audit/probe.tex`):

| Input | Real TeX result (sp) |
|---|---|
| `1in` | **4736286** |
| `72.27pt` | **4736287** (one sp more than real `1in` — confirms the task's stated divergence exactly) |
| `1cm` | 1864679 |
| `1mm` | 186467 |
| `1bp` | 65781 |
| `1pc` | 786432 |
| `1dd` (didot) | 70124 |
| `1cc` (cicero) | 841489 |
| `16383.99998pt` | 1073741823 (= `MAX_DIMEN`) |
| `16384pt` | `! Dimension too large.` (rejected, one past `MAX_DIMEN`) |

## Matrix

Only crates with at least one hard-coded TeX-unit constant are columns. The
other 13 assigned crates (title-layout, toc-layout, color-expressions,
image-assets, link-annotations, math-accessibility, spellcheck,
project-templates, editor-snippets, document-statistics, project-bundle,
collaboration-core, bibliography, project-files, vector-graphics) were
searched and contain **no** hard-coded TeX dimension constants — see "Clean
crates" below.

| Constant | tex-calc | document-style | paragraph-layout | math-layout | compiler | Real TeX |
|---|---|---|---|---|---|---|
| sp per pt | `65536` (`sp.rs:18`) | *(none — never quantizes to sp)* | *(none; only cited in a comment, `adapter.rs:9`)* | `65536.0` (`tfm.rs:13`) | *(n/a — works in bp)* | 65536 |
| `MAX_DIMEN` (sp) | `1_073_741_823` = `(1i64<<30)-1` (`sp.rs:23`) | *(not defined)* | `1_073_741_823` (`adapter.rs:38`) | *(not defined)* | *(not defined)* | 1073741823 |
| `MAX_DIMEN` (pt) | *(not defined; crate stays in sp)* | *(not defined)* | `16383.99998` (`adapter.rs:43`) | *(not defined)* | *(not defined)* | 16383.99998 (prints as this; = 1073741823sp) |
| **pt per inch** | **exact rational (7227,100), truncated once to sp** — effectively `72.26999...` (`sp.rs:48`, `Unit::In => (7227, 100)`) | **`72.27`** (`length.rs:10`, `PT_PER_IN`) | **`72.27`** (`lib.rs:31`, inside `BP_PER_TEX_PT = 72.0/72.27`; also `pages.rs:97-100`, margins `= 72.27` for "1in") | *(none)* | *(n/a — see below)* | 4736286sp exactly; prints `72.26999pt` |
| bp per inch (72) | ratio `(7227,7200)` pt/bp, i.e. 72bp=1in exactly (`sp.rs:52`) | `PT_PER_BP = 72.27/72.0` (`length.rs:12`) | `BP_PER_TEX_PT = 72.0/72.27` (`lib.rs:31`) | *(none)* | `PAGE_WIDTH_PT=612.0`, `PAGE_HEIGHT_PT=792.0`, `MARGIN_PT=72.0` — all **big points**, 8.5in/11in/1in at exactly 72bp/in (`layout.rs:24-26`) | 72bp = 1in, exact, no ambiguity |
| pt per pica (12) | `(12, 1)` (`sp.rs:49`) | `PT_PER_PC = 12.0` (`length.rs:18`) | *(none)* | *(none)* | *(none)* | 12pt = 1pc, exact |
| didot (1238/1157) | **not implemented** — `Unit` enum has no `Dd`/`Cc` variant; explicitly typed-rejected, see `tests/tex_oracle.rs:203-205` and test `didot_and_font_relative_units_are_rejected_not_approximated` (`tests/tex_oracle.rs:546`) | not implemented | not implemented | not implemented | not implemented | 1dd = 1238/1157 pt exactly; truncates to 70124sp |
| cicero | not implemented (same as above) | not implemented | not implemented | not implemented | not implemented | 1cc = 12dd = 841489sp |
| mm/cm factor (25.4, 2.54) | `Cm=(7227,254)`, `Mm=(7227,2540)` (`sp.rs:50-51`) — algebraically the same rational as `document-style`'s | `PT_PER_MM = 72.27/25.4`, `PT_PER_CM = PT_PER_MM*10.0` (`length.rs:14,16`) | *(none)* | *(none)* | *(none)* | 1cm=1864679sp, 1mm=186467sp (both match `tex-calc`'s own test assertions and real TeX exactly) |

Numbers that look like TeX constants but are **not** (checked and excluded from the matrix as coincidental):
- `flashtex-font-engine/src/truetype.rs:502` — `/ 65536.0` is TrueType's `post`-table Fixed 16.16 format, unrelated to sp-per-pt.
- `flashtex-font-resources` (`src/mapping.rs:47`, `src/vf.rs:171`, `src/math_variants.rs:41/151/171`, `src/tfm.rs:230/370-371`, `src/outline.rs:146/211/217`, `src/expansion.rs:131-132`, `src/encoding.rs:54`, `src/enc_file.rs:226`, `src/registry/vf_project.rs:232`, `src/lib.rs:122`) — every `65536` here is either a glyph/cmap/record count bound or a TrueType outline Fixed-point scale, never a TeX sp/pt conversion.
- `link-annotations`, `vector-graphics`, `font-engine/examples/pdf_roundtrip.rs`, `math-layout/tests/*.rs` — `72.0`/`700.0` etc. are test/example PDF-point coordinates, not constant definitions.

## Disagreements found

### 1. Points-per-inch: `tex-calc` vs. `document-style` (the confirmed instance)

- `flashtex-tex-calc/src/sp.rs:48` — `Unit::In => (7227, 100)`, converted to sp with one exact truncation (`sp.rs:121-160`). For `1in` this yields **4736286sp**, matching real TeX exactly (verified above).
- `flashtex-document-style/src/length.rs:10` — `pub const PT_PER_IN: f64 = 72.27;`, used directly as an f64 multiplier with no sp quantization anywhere in the crate.
- Real TeX: `1in` = 4736286sp; `72.27pt` (document-style's constant, taken at face value and rounded to the nearest sp) = 4736287sp. **One scaled point apart**, as stated in the task.

**Severity: real bug, but currently latent (not yet reachable from compiled output).** `flashtex-tex-calc` depends on `flashtex-document-style` (only for the `Pt` newtype, via a one-directional `Sp -> Pt` conversion in `sp.rs:282-286` that never touches `PT_PER_IN`), so the two crates are clearly meant to interoperate in the same engine. But as of this audit:
  - **Nothing in the whole repository depends on `flashtex-tex-calc`** except its own tests (checked every `Cargo.toml` under every worktree and the main checkout).
  - `flashtex-document-style` has exactly one consumer, `flashtex-title-layout`, and that dependency is real — but title-layout's own `Cargo.toml` states outright "nothing in this repo calls this crate" (it's exercised only via a dev-dependency test fixture). `flashtex-compiler` (the only actual binary) does not depend on `document-style` at all, directly or transitively.
  - So today, neither crate's inch constant can produce a byte of different PDF output, because neither is wired into `flashtex-compiler`. The moment either is integrated (which their design clearly anticipates — `tex-calc` already imports `document-style`'s `Pt` type), a single `1in` in one document will silently resolve to two different scaled-point values depending on which crate touched it. This is exactly the bug the task describes; it's real and worth fixing before integration, but it cannot corrupt a real compile *yet*.

I independently verified, rather than assumed, the task's claim that document-style's exposure is narrower than it looks: `PT_PER_IN`/`PT_PER_MM`/`PT_PER_CM`/`PT_PER_BP`/`PT_PER_PC` are applied in exactly three places in `document-style`'s own source, and all three take fixed literals, never user input:
  - `geometry.rs:45-48` (`Paper::size()`) — fixed paper dimensions (8.5in/11in/210mm/297mm/148mm/210mm/8.5in/14in) straight from `article.cls`'s `\DeclareOption`.
  - `geometry.rs:92,105,117-122` (`article_page_params`) — fixed LaTeX-class literals (`1in`, `0.4in`, `1.5in`, `2.0in`) from `size1x.clo`.
  - `geometry.rs:254,280,292-293,329,336` (`apply_geometry`, `PageLayout::from_params`) — the same fixed `one_inch = Pt::inches(1.0)` LaTeX-spec offset.
  The one function that *would* expose arbitrary user lengths to `PT_PER_IN` — `Pt::parse` (`length.rs:54-76`, documented as parsing "a LaTeX absolute length such as `1in`, `2cm`...") — is called **nowhere in production code in this repository**; the only call sites are `document-style`'s own `tests/article.rs:739-745`. The JSON round-trip path that a real caller would use (`serialize.rs`'s `from_json`, line ~258) reads `margin`/`top`/`bottom`/etc. as already-resolved numeric point values (`opt_num(g, "margin")?.map(Pt)`), not unit strings, so it never touches `Pt::parse` either. Confirmed, not assumed.

### 2. Points-per-inch: `paragraph-layout` also hard-codes `72.27` (a second, previously unflagged site)

- `flashtex-paragraph-layout/src/lib.rs:31` — `pub const BP_PER_TEX_PT: f64 = 72.0 / 72.27;`
- `flashtex-paragraph-layout/src/pages.rs:97-100` — `margin_top`/`margin_bottom`/`margin_left`/`margin_right: 72.27` inside `PageParams::article_12pt_letter_1in_tex_pt()`, documented at `pages.rs:91` as "margins 1in = 72.27pt".

**Severity: same disagreement as #1, also currently latent.** `paragraph-layout` has zero non-dev dependencies and, by its own `Cargo.toml` comment, is consumed by nothing except test fixtures ("Neither crate is a dependency of the shipped library"). It *is* transitively linked into the `flashtex-compiler` binary — `font-engine`'s `paragraph` feature (which pulls in `flashtex-paragraph-layout`) is on by default, and `compiler` depends on `font-engine` without disabling default features. But I traced the actual call path: `font-engine`'s `adapters::paragraph` module only re-exports `paragraph-layout`'s `FontMetricsSource`/`GlyphRun`/`Ligature` types (`font-engine/src/adapters/paragraph.rs:20-21`) — it never calls `PageParams`, `BP_PER_TEX_PT`, or `layout_paragraph`. `compiler/src/layout.rs` implements its own independent (and much simpler) greedy layout with its own `MARGIN_PT = 72.0` in big points (see #3) and never references `paragraph-layout` at all. So `paragraph-layout`'s copy of the wrong constant is linked into the binary but dead code — unreachable from any real compile today.

### 3. `compiler`'s `MARGIN_PT = 72.0` — not a disagreement

- `flashtex-compiler/src/layout.rs:24-26` — `PAGE_WIDTH_PT = 612.0`, `PAGE_HEIGHT_PT = 792.0`, `MARGIN_PT = 72.0`.
- These are **PDF big points** (612bp = 8.5in, 792bp = 11in, 72bp = 1in), not TeX points, even though the identifiers say `_PT`. Since 1in = 72bp is exact by definition (no truncation involved either direction), this doesn't disagree with anyone's inch constant — it's simply a different, unambiguous unit. Worth a naming note (the `_PT` suffix invites confusion with TeX points) but not a numerical bug.

### 4. Didot / cicero — not implemented anywhere, so no disagreement is possible

Only `tex-calc` even discusses didot/cicero, and only to explicitly *reject* them (`Unit` has no `Dd`/`Cc` variant; `tests/tex_oracle.rs:546`, `didot_and_font_relative_units_are_rejected_not_approximated`). No other crate references these units at all. Real TeX's values (verified above: 1dd = 70124sp, 1cc = 841489sp, both matching the documented `1238/1157` ratio exactly) have nothing to disagree with yet.

### 5. Everything else agrees

- **sp per pt (65536):** `tex-calc` and `math-layout` both define it explicitly and correctly; `paragraph-layout` cites it correctly in a comment without needing its own constant. No crate uses a different value.
- **`MAX_DIMEN`:** `tex-calc` (`1_073_741_823` as `(1i64<<30)-1`) and `paragraph-layout` (`1_073_741_823` sp and the matching `16383.99998` pt form) agree with each other and with real TeX exactly, including the reject-at-one-past-the-bound behavior. `document-style`, `math-layout`, and `compiler` don't define a `MAX_DIMEN` at all — an omission, not a disagreement.
- **Big points per inch (72), points per pica (12), and the mm/cm rational:** every crate that defines these encodes the same underlying exact ratio (`7227/7200` for bp, `12/1` for pica, `7227/2540` for mm / `7227/254` for cm — `document-style`'s `72.27/72.0`, `72.27/25.4` are algebraically identical to `tex-calc`'s integer ratios since `72.27 == 7227/100` exactly). The only place these could still diverge numerically is the same root cause as #1: `document-style` never quantizes to an integer scaled point, so a hypothetical future `Pt -> Sp` conversion using round-to-nearest on `document-style`'s float would land 1sp away from `tex-calc`'s truncate-once result for any non-integral case (bp, cm, mm) — but I found no such conversion anywhere in the repo today (checked for `* 65536` / `* SP_PER_PT` / `.round() as i64` patterns outside `tex-calc`'s own internals — none exist). Not a present bug; flagged so it isn't missed if that conversion gets written later without going through `tex-calc`.

## Clean crates (no hard-coded TeX constants found)

`title-layout`, `toc-layout`, `color-expressions`, `image-assets`,
`link-annotations`, `math-accessibility`, `spellcheck`, `project-templates`,
`editor-snippets`, `document-statistics`, `project-bundle`,
`collaboration-core`, `bibliography`, `project-files`, `vector-graphics`,
`font-resources` (has many `65536`s, all confirmed unrelated — see above).

`title-layout` (worktree `~/ft-wt-daniel-title`) had three files mid-edit
(`src/abstract_block.rs`, `src/error.rs`, `tests/abstract_block.rs`) when this
audit started; by the time they were read, the worktree had committed and its
tree was clean, and none of the changes touched a numeric constant — it only
consumes `Pt`/`Skip`/`font_size`/`list_level` values already computed by
`document-style`, it never hard-codes its own inch/pt ratio.

## Cannot determine without running it

Nothing in this audit required a runtime judgment call beyond the TeX oracle
runs already performed above — every question (which constant, which value,
which crate reaches which output) was resolved by reading source and
`Cargo.toml` dependency graphs, or by running real TeX.

## Summary

- **Constants checked:** sp-per-pt, `MAX_DIMEN` (sp and pt form), pt-per-inch,
  bp-per-inch, pt-per-pica, didot, cicero, mm/cm conversion factors.
- **Crates checked:** 22 (16 from worktrees, 6 from the main checkout), all
  read via `git status`/plain reads, none modified.
- **Disagreements found:** one root cause (pt-per-inch: `72.27` vs. the
  TeX-accurate `7227/100` truncated to sp), appearing in **two** crates —
  `flashtex-document-style` (`length.rs:10`) and `flashtex-paragraph-layout`
  (`lib.rs:31`, `pages.rs:97-100`) — against `flashtex-tex-calc`'s correct
  value (`sp.rs:48`). Both are **real bugs by the task's own criterion**
  (two crates meant for the same engine computing different values for the
  same TeX constant) but **both are currently latent**: neither
  `document-style` nor `paragraph-layout` nor `tex-calc` is reachable from
  `flashtex-compiler`'s actual compile path today, so no PDF byte differs
  because of this yet. Everything else checked agrees, or doesn't yet exist
  in more than one place to disagree.
- **TeX ground truth measured:** `1in`=4736286sp, `72.27pt`=4736287sp,
  `1cm`=1864679sp, `1mm`=186467sp, `1bp`=65781sp, `1pc`=786432sp,
  `1dd`=70124sp, `1cc`=841489sp, `16383.99998pt`=1073741823sp (`MAX_DIMEN`),
  `16384pt`→rejected. All measured with real `tex` (BasicTeX 2026,
  `/Library/TeX/texbin/tex`) in
  `/private/tmp/claude-503/-Users-dqi26/50763fe3-5136-4e4d-ab08-2a19a3be5880/scratchpad/tex-audit/probe.tex`.

---

## Supervisor correction: `paragraph-layout` is NOT a defect

The audit above flags `crates/paragraph-layout` alongside `document-style` as a
real instance of the 72.27-versus-72.26999 divergence. **That conflates two
different quantities, and the paragraph-layout half does not hold.**

### `BP_PER_TEX_PT = 72.0 / 72.27` is correct by definition

The TeX point is *defined* as exactly 1/72.27 inch. Converting TeX points to
PostScript/PDF big points is therefore exactly `72/72.27`, and writing that
ratio is right. It is not an approximation of anything.

`72.26999` is a different fact: it is what TeX *produces* when asked to convert
`1in` into points, because `\dimen` arithmetic is integer scaled points and the
conversion rounds. Measured against real TeX:

    1in      = 4736286 sp  ->  4736286/65536 = 72.2699890 pt
    72.27pt  = 4736287 sp

One scaled point apart, and the difference is TeX's rounding of the *conversion*,
not a different definition of the point.

### The empirical check settles it

`crates/paragraph-layout/tests/oracle_wrap_sample.rs` pins line starts against
**real pdflatex output**, and both its tests pass with the current constant:

    test total_fit_ragged_matches_pdflatex_line_starts ... ok
    test first_fit_ragged_matches_total_fit_on_this_sample ... ok

Changing `BP_PER_TEX_PT` to `72.0/72.26999` would move every converted
coordinate and would be expected to *break* a test currently agreeing with a
real engine. Acting on the audit's recommendation here would have introduced a
defect, not removed one.

### What remains genuinely open in this crate

`pages.rs:97-100` sets `margin_top`/`bottom`/`left`/`right` to the literal
`72.27`, with a doc comment reading "margins 1in = 72.27pt". If the intent is
"a one-inch margin as TeX would compute it", TeX gives 4736286sp and this gives
4736287sp — one scaled point wider per margin. That is a real, if tiny,
divergence and it is a **semantics question, not a typo**: it depends whether
these margins are meant to reproduce TeX's rounded inch or to be exactly
72.27pt as written.

Deliberately not changed. It is the same class as the `tex-calc` decimal-rounding
divergence already raised with the Commander: altering it changes output
geometry, and the pinned golden fixtures would move. Raised, not decided.

### Standing

`document-style`'s instance (`length.rs:10`) is unaffected by this correction and
remains as the audit reports it — though that crate now belongs to
**mac-document-style** per the Commander's 17:56 triage, so it is theirs to
judge, and the ground-truth scaled-point table above was handed to them on GH#44.

Everything else the audit checked — sp-per-pt, `MAX_DIMEN` in both forms,
bp/pica/mm/cm ratios — agrees across every crate that defines it, and didot and
cicero are implemented nowhere.
