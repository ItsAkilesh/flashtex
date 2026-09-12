# mac-pdf handoff — FT-009 PDF output

- Updated UTC: 2026-09-12T06:50Z
- Agent / parent / machine alias: `mac-pdf` (Claude Code subagent; issue #2
  follow-up dispatched as worker `mac-pdf-unicode`, same agent and branch) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: FT-009 rev 1, "original Rust PDF output
  from runtime-v1 positioned pages". Serves acceptance gate 5 (a real exported
  document). Owned paths: `crates/pdf/`, `coordination/mac-pdf.md`.
- Branch / code revision / main integrated through:
  `agent/mac-pdf/pdf-output` / see commit / `25a92a9`
- State: ready for integration
- Ready behavior and evidence:
  - `crates/pdf` (`flashtex-pdf`, edition 2024, zero dependencies) writes PDF 1.4
    by hand from a runtime-v1 `compile_result`: one page object per page with
    the page's own `MediaBox`, one `BT … ET` per text item at
    `(x_pt, height_pt - baseline_y_pt)` with `/F1` Times-Roman (WinAnsi) and
    `/F2` Symbol runs switched inside the block, U+2500-only items drawn as
    `re f` fraction rules, xref table, trailer. Library API
    `render_pdf` / `render_envelope` returning `{ bytes, warnings }`; CLI
    `flashtex-pdf [in.json] --out out.pdf [--verify]`.
  - Issue #2 follow-up: opt-in TrueType embedding with subsetting, hand-written
    (`src/truetype.rs`, `src/embed.rs`, still zero dependencies): Type0 /
    CIDFontType2 / Identity-H / `CIDToGIDMap Identity` / `FontFile2` /
    ToUnicode CMap. Enabled by `RenderOptions.embed_font`, `--embed-font
    PATH|auto`, or `FLASHTEX_UNICODE_FONT`; `auto` falls back on macOS to
    Times New Roman then Arial Unicode (Apple system fonts; redistribution
    licence is the user's call, README says so). Characters the font also
    lacks stay `?` with warnings; CLI prints a `note: N warning(s)` summary,
    exit 0. Verified on this Mac with both system fonts: rasterised output
    shows Cyrillic/CJK/ℝ and composite glyphs; `?` only where warned.
  - Document face (defect found by coordinator in e3e5e1e fixed): the
    embedded font was only a gap-filler behind Times, so LM was embedded but
    unused. `RenderOptions.face` / `--default-face embedded|lm|times`; Latin
    Modern implies `embedded` (LM first, Symbol, then Times, `?` last),
    other fonts imply `times`. Verified: sample `Latin Modern naïve — café`
    uses only `/F3`; PDFKit selection width of "Latin Modern" at 12pt =
    72.552pt = LM advances (Times would be 66.324pt); raster is CM-style.
  - CFF OpenType (Latin Modern, `OTTO`) embedding: the whole `CFF ` table
    verbatim as `/FontFile3 /Subtype /CIDFontType0C` under CIDFontType0 +
    Identity-H + sparse `/W` + ToUnicode. `/Type1C` was rejected by
    CoreGraphics ("unsupported CIDFontType0 subtype"); `/OpenType` also
    works but is 50 KB larger. `--embed-font auto` now prefers Latin Modern
    (`FLASHTEX_LM_DIR`, TeX Live roots newest-first, Linux TeX dirs) before
    Times New Roman / Arial Unicode. Verified: sips rasterises with no
    CoreGraphics font error, PDFKit `page.string` returns the input text.
    One-line PDF with LM embedded: 63,228 bytes (CFF table 61,140). No CFF
    subsetting yet; non-CID-keyed CFF relies on CID = GID (CoreGraphics does
    this; CID-keyed conversion is the robust follow-up).
  - Corpus regression (`crates/pdf/docs/corpus-report.md`, runner
    `crates/pdf/scripts/corpus_report.py`): all 14 `tests/tex-corpus` cases
    through compiler origin/main 342e1e0 → flashtex-pdf: 14/14 verify and open
    in sips; 12/14 zero warnings with default fonts, the 2 CJK cases warn;
    Arial Unicode embedding gives 14/14 zero warnings. de1020c comparison
    column for the math case.
  - Native visual checks (`crates/pdf/docs/visual-checks.md`): three PDFs
    rasterised by macOS and described, defects included. New finding for the
    compiler/contract owners: with CJK embedded, `東京.` overlaps `After` in
    `unicode-literals` because the compiler assumes 0.5 em for glyphs it has
    no metrics for (`DEFAULT_ADVANCE_UNITS`); real ideographs are 1 em. The
    PDF writer places items where told and does not re-flow.
  - `cargo test`: 39/39 pass (19 unit, 20 integration). Covers fixture page count
    and MediaBox, a 2-page synthetic result, multiline baselines, WinAnsi
    encoding (`é` → `0xE9`), unrepresentable chars (`中`, `😀`, `ℝ`) → `?` +
    warning, delimiter escaping, unsupported item kinds, bad envelopes,
    determinism, the CLI, an independent xref self-check, and (macOS) `sips`
    opening the fixture PDF and reporting 612 × 792.
  - Issue #9 follow-up: the exact compiler (de1020c) output for
    `$\frac{a}{b}+\alpha+\sqrt{x}$` is checked in as
    `crates/pdf/tests/fixtures/math-compile-result.json`; the test asserts zero
    warnings, one `re f` rule 8.4 × 0.72pt at x=72 on baseline 84.72, `α` via
    `/F2` byte `0x61`, `√` via `/F2` byte `0xD6`, both fonts in `/Resources`.
    Rasterised by macOS the PDF shows a/b with a drawn bar, α, and √x, no `?`.
  - Manual on mac-m1max-a: `/usr/bin/sips -g pixelWidth -g pixelHeight -g format`
    on the generated fixture PDF → `pixelWidth: 612.000`, `pixelHeight: 792.000`,
    `format: pdf`, exit 0. A two-page Unicode sample rasterised to PNG showed the
    heading, three baselines, accents, escaped parentheses, `?` substitution,
    and page two on a white page.
  - `cargo build --release` and `cargo clippy --all-targets` clean.
- Incomplete behavior / blockers / needs from others:
  - Embedding is opt-in; default output remains base-14 Times-Roman (WinAnsi)
    and Symbol. No shaping, no colour emoji, no .ttc, no Flate, no CFF
    subsetting (whole CFF per document). Latin Modern Roman covers Latin only,
    so Greek/Cyrillic/CJK still warn under `auto`. Heading
    weight is not reproduced: the compiler sets headings in Times-Bold but
    runtime-v1 carries no font field, and per issue #9 bold is not inferred from
    size. Needs a runtime-v1 font/weight field (proposed to Commander).
  - Characters outside WinAnsi and Symbol become `?` with a warning. Symbol has
    no bold/italic and no stretchy delimiters; the radical has no overbar.
  - Text items and U+2500 rule items only; other kinds are skipped with a
    warning. No images or general paths. A real rule item type in runtime-v1
    would replace the U+2500 convention (Commander's call).
  - Word spacing in the PDF reflects the FT-002 compiler's placeholder glyph
    widths; this crate does not re-measure text.
  - Not wired into `crates/compiler` or `apps/mac` yet (`pdf_path` still null).
    Integration steps are in `crates/pdf/README.md`; wiring belongs to the
    compiler/Mac owners, not this task.
- Needs from others: compiler/contract owners to decide how the compiler
  learns embedded-font advances (or a width exchange in runtime-v1) so
  embedded wide glyphs do not collide; a font/weight field for headings.
- Interface changes / consumer actions: none. Consumes runtime-v1 `compile_result`
  unchanged. Relies on the compiler's convention that fraction rules are text
  items consisting only of U+2500 (0.5 em per dash, baseline at the bar's
  bottom edge, thickness 0.06/0.7 em of the item size); documented in README. Surfacing PDF warnings in the UI would need a contract addition
  (Commander's call); until then the worker can emit them as `warning` diagnostics.
- Reviewed peer revisions / resulting adaptations: `origin/main` 25a92a9 (branch
  base); `origin/agent/claude/compiler-foundation` 29221d8 `crates/compiler`
  README and `src/layout.rs` for the emitted item shape (one item per word,
  612×792, top-left origin, `round2` coordinates) — matched by writing absolute
  `Td` per item and formatting numbers to three decimals. Follow-up: compiler
  de1020c `src/math.rs` (fraction rule construction, `SCRIPT_SCALE` 0.7,
  `FRACTION_RULE_EM` 0.06, Greek/radical as Unicode text) and `src/metrics.rs`
  (500-unit fallback for non-Latin glyphs) plus GitHub issue #9 — adapted by
  rendering U+2500 runs as rules and adding the Symbol font.
- Validation commands / results / artifact paths:
  `cd crates/pdf && cargo test` (39 passed);
  `cargo run --bin flashtex-pdf -- in.json --out out.pdf --embed-font auto` (prints which font was embedded);
  `cargo run --bin flashtex-pdf -- tests/fixtures/math-compile-result.json --out math.pdf --verify` (exit 0, no warnings);
  `cargo run --bin flashtex-pdf -- --out out.pdf --verify < ../../protocol/fixtures/compile-result.json`;
  `/usr/bin/sips -g pixelWidth -g pixelHeight out.pdf`.
- Resource pool / allocation: parent `mac-claude-a`'s Claude Max allocation on
  mac-m1max-a; this subagent's usage is deducted from it. Exact token totals not
  measured here.
- Dirty files / unpushed work / running jobs: none after push.
- Decisions: hand-written JSON reader instead of serde so the crate builds
  offline with no dependencies, matching `crates/compiler`; no background fill
  so export is white regardless of preview theme; one `BT/ET` per item so the
  content stream is trivially verifiable; font runs switched with `Tf` inside
  one text object so the viewer's real base-14 advances position them and no
  width tables are needed in this crate.
- Exact next action: Commander/integrator merges `agent/mac-pdf/pdf-output`;
  compiler or Mac owner wires `pdf_path` per `crates/pdf/README.md`.
