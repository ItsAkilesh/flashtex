# mac-pdf handoff — exact export lane (issue #25) on top of FT-009

- Updated UTC: 2026-09-12T09:15Z
- Agent / parent / machine: `mac-pdf` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`. Context usage at this checkpoint: about 3%
  of the session budget (14.55 M of 15 M tokens remaining per the runtime
  counter), far below the 80% compaction threshold in the user policy.
- Lane: "Exact original glyph/font/rule PDF export against declared reference
  profile"; follow-ups "Bounded font subset identity and deterministic object
  serialization tests" and "Classify remaining raw-byte versus raster
  differences". Owned paths: `crates/pdf/`, `coordination/mac-pdf.md`,
  `coordination/agents/mac-pdf.json`. Transferred crates
  (`font-engine`, `paragraph-layout`, `math-layout`) untouched.
- Branch / HEAD / worktree: `agent/mac-pdf/exact-export` /
  `528d52b` (merge of origin/main `b1cf8b9`; product commits `e481f01`,
  `d25a647`, `b075468`, docs commit after) / worktree
  `.claude/worktrees/agent-a665565ecb0f11ace` on mac-m1max-a. Pushed.
- State: ready for integration (all three lane items done; see limits).
- Ready behavior and evidence:
  - `crates/pdf/src/exact.rs`: additive exact API for issue #25. `Decimal`
    validated PDF numeric token written verbatim (no f64 round trip);
    bounded typed operator set `Op` (q Q cm w J j d g G rg RG m l c h re S f
    f* n W W* BT ET Tf Td Tm Tj TJ) with `Op::rule`; `parse` validates
    verbatim content into the same set; `GlyphRun`/`PlacedGlyph` by original
    glyph id (Type0/Identity-H, two-byte codes = GIDs); `ExactFont::{CidCff,
    CidTrueType, Simple}` with programs embedded byte for byte (Type 1
    `FontFile` Length1/2/3, `FontFile3` CIDFontType0C/Type1C, `FontFile2`),
    descriptors with verbatim extras, `/Differences` encodings, ToUnicode
    and CIDSet carried verbatim, per-page `/Resources`; `render_exact`
    through the crate's single `Document` container, deterministic (no /ID,
    no dates). Errors name page and operator index; nothing is dropped.
  - `crates/pdf/src/cff.rs`: CFF parser + GID-preserving subsetter: output is
    CID-keyed (charset CID = original GID, one FD with the original Private
    DICT and local subrs verbatim, charstrings byte-identical). Refuses
    CID-keyed sources, Type 1 charstrings and `seac` glyphs explicitly;
    `ExactFont::cid_from_opentype` then embeds whole and reports it. Latin
    Modern: 821 glyphs, no seac; 9-glyph subset 22,557 B vs 61,140 B whole;
    CoreGraphics renders it (test + raster inspected).
  - `truetype.rs::subset_keep_gids` (glyph order preserved for
    `/CIDToGIDMap /Identity`); `inflate.rs` (RFC 1950/1951, bounded) and
    `sha256.rs`, both in-tree, zero dependencies kept.
  - `reader.rs` (bounded PDF object reader: xref/object streams, Flate),
    `compare.rs` (`reemit` any read PDF as an `ExactDocument`; `classify`
    with categories PageGeometry, ContentFormatting/Operands/Operators/
    Unsupported, FontProgram, FontMetadata, FontResources, ObjectLayout,
    Compression, DocumentIdentity), bin `flashtex-pdf-exact reemit|classify|dump`.
  - Reference-profile evidence (`crates/pdf/docs/exact-export-classification.md`,
    oracle only, MacTeX 2026 never in the product path): all 18 visual-corpus
    fixtures × {pdflatex times, pdflatex lmodern, xelatex Times New Roman,
    xelatex Latin Modern OTF} = 72 references, 92 pages, 234 embedded
    programs re-emitted through the exact API: content streams byte-identical
    on every page, every font program SHA-256-identical, no font metadata
    differences, CoreGraphics 144 dpi rasters pixel-identical on all 92
    pages; remaining differences exactly ObjectLayout (PDF 1.7 xref/object
    streams vs 1.4 table, object order), Compression (Flate vs none),
    DocumentIdentity (/ID, dates, producer). This establishes what the
    container preserves, not what FlashTeX's compiler emits.
  - Tests: `cd crates/pdf && cargo test` → 71 passed (35 unit, 11
    `tests/exact.rs`, 25 `tests/render.rs`); `cargo clippy --all-targets`
    clean; `cargo build --release` clean. `tests/exact.rs` covers:
    byte-identical output for the same input twice + xref self-check + no
    /ID/dates; verbatim decimals (`12.000`, `700.12345`, `216.00000000001`)
    and two-byte codes re-parsing to the same ops; CFF subset identity
    (CID = original GID, charstring bytes equal, deterministic, out-of-range
    and CID-keyed refused); codes outside the declared glyph set rejected;
    validation errors (q/Q, BT/ET, path state, undeclared font, page
    resources, `gs`/`sc`, exponents); pdfTeX-style Type 1 simple font round
    trip through the reader and reemit as a fixed point; PFB parsing;
    classifier categories; skipped-when-absent: Latin Modern CFF subset in
    CoreGraphics, pdflatex and xelatex oracle round trips asserting the
    three remaining categories. All ran (not skipped) on this Mac.
  - runtime-v1 route (`flashtex-pdf … --verify`) untouched: `tests/render.rs`
    25/25 unchanged.
- Incomplete behavior / limits (stated, not hidden):
  - The exact route is a library API plus the audit binary; nothing feeds it
    yet. rendering-core's `pdf_stream::PdfCommandStream` (branch
    `agent/commander-render-core/rendering-core`) emits operator bytes in
    exactly the bounded set (`q re W n rg m l c h f Q`); its `content_bytes`
    can be handed to `Content::Verbatim` today, and its glyph GIDs to
    `GlyphRun` once it exposes them per run. Not wired by me (not my path).
  - Alpha, shading, images, inline images, `gs`, `sc/scn` are outside the
    bounded set and error (as issue #25 requests). No writer-side Flate.
  - CFF subsetting keeps whole global/local subr INDEXes (bounded, simpler;
    sizes above). CID-keyed source CFFs and seac glyphs embed whole. Type 1
    programs are embedded whole by `FontProgram::type1_from_pfb` (pdfTeX
    subsets them); no fixture here exercises that since re-emit reuses
    pdfTeX's own subset bytes.
  - Parity was measured with CoreGraphics only.
- Needs from others: Commander to review/merge `agent/mac-pdf/exact-export`
  and close or update issue #25; rendering-core owner to route its stream
  through `Content::Verbatim` / `GlyphRun` and report anything the bounded
  set refuses.
- Interface changes / consumer actions: additive only. New public modules
  `exact`, `cff`, `reader`, `compare`, `inflate`, `sha256`; new bin
  `flashtex-pdf-exact`; `TrueTypeFont::subset_keep_gids`. No contract change.
- Reviewed peer revisions / adaptations: GitHub issue #25 and
  `crates/rendering-core/PDF-INTEGRATION.md` + `src/pdf_stream.rs` on
  `origin/agent/commander-render-core/rendering-core` (operator set and
  exact-decimal requirement adopted verbatim); `docs/contracts/
  rendering-v2-proposal.md` (original-GID → subset mapping, CID/GID
  namespaces: satisfied by CID = original GID); `tests/visual-corpus/harness/
  render_reference.sh` and `reference-profile.json` on
  `origin/agent/mac-visual-oracle/reference-raster` (preambles reproduced
  exactly for the oracle run; read only). origin/main `b1cf8b9` merged; no
  crates/pdf changes upstream.
- Validation commands: `cd crates/pdf && cargo test && cargo clippy
  --all-targets`; `cargo run --release --bin flashtex-pdf-exact -- classify
  REF.pdf OUT.pdf` (exit 0 iff content ops and programs identical).
- Resource pool / allocation: parent `mac-claude-a`'s Claude Max allocation
  on mac-m1max-a (shared quota, no purchases). No paid API calls.
- Dirty files / unpushed work / running jobs: none after this push.
- Exact next action: Commander integration of the branch; then rendering-core
  hand-off through the exact API (their side).

---

Previous handoff (FT-009 runtime-v1 route, still accurate for that route):

# mac-pdf handoff — FT-009 PDF output

- Updated UTC: 2026-09-12T07:40Z
- Agent / parent / machine alias: `mac-pdf` (Claude Code subagent; issue #2
  follow-up dispatched as worker `mac-pdf-unicode`, same agent and branch) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: FT-009 rev 1, "original Rust PDF output
  from runtime-v1 positioned pages". Serves acceptance gate 5 (a real exported
  document). Owned paths: `crates/pdf/`, `coordination/mac-pdf.md`.
- Branch / code revision / main integrated through:
  `agent/mac-pdf/pdf-output` / see commit / `0b7076a` (branch fast-forwarded
  onto main after the earlier PDF commits were integrated)
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
  - Layout capabilities (contract a949f5b): `payload.layout_capabilities`
    parsed; `rules-v1` typed rules drawn as `re f` from top-left geometry
    (validated: positive finite, ≤1e6); a `rule` without `rules-v1` and any
    unknown kind under negotiation are errors naming kind and source range;
    legacy route keeps warn-and-skip and the U+2500 approximation (documented
    as such); `font-hints-v1` resolves Latin Modern → per-face whole-CFF font
    objects (`/F4`…), Times → base-14 variants, anything else → substitution
    warning naming the replacement. Legacy fixtures byte-identical to the
    c0f3837/d7f2c3a-era outputs (cmp). Rasterised: four LM faces + Times-Bold
    + a typed fraction rule render natively; PDFKit extracts the text.
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
  - `cargo test`: 45/45 pass (20 unit, 25 integration). Covers fixture page count
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
- Interface changes / consumer actions: consumes runtime-v1 plus the
  additive layout-capabilities contract; library model is now
  `Item::{Text(TextItem), Rule(RuleItem)}` with `CompileResult.capabilities`. Relies on the compiler's convention that fraction rules are text
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
  `cd crates/pdf && cargo test` (45 passed);
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
