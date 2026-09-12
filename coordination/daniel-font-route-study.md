# Font route study (lane `daniel-font-route-study`)

Read-only research at `origin/main` c7ff3f74, 2026-09-12 ~19:45Z. No code,
font files, or pins were changed. Local TeX Live 2026 was used only to locate
files and read metrics/licences (no downloads).

## Partial \mathbb notes for daniel-parent-b (handed over, not a recommendation)

Scope moved to `daniel-parent-b` mid-study. These are findings, not a ruling.

**Correct the premise first.** There are two producers, and both share the
parser in `crates/compiler` (render-pipeline vendors it), which is why HW1's
11 `\mathbb is not supported in math mode` diagnostics are identical on both
(`fixtures/real-world/hw1/gap-report-2026-09-12.md`). The diagnostic comes from
`crates/compiler/src/math.rs` `command_atom` (fallback arm, ~line 341), not
from the export guard.

- `flashtex-compiler`: layout on font-engine Core14 AFM metrics
  (`crates/compiler/src/layout.rs` `face`/`math_font`), export through
  `crates/pdf` runtime-v1 with base-14 `Times-Roman` + `Symbol` that are **not
  embedded** (`crates/pdf/src/lib.rs` header). The guard
  `every_math_symbol_has_a_decided_export_outcome` iterates `COMMAND_GLYPHS`
  only; the separate compile-time warning in `crates/compiler/src/protocol.rs`
  (~line 644, `export::unrepresentable`) would fire for U+211D etc.
- `flashtex-render` (`crates/render-pipeline`): TFM metrics (`lmmi/lmsy/lmex`
  via math-layout `CmMathMetrics`, `rm-lmr*` read from TFM) and painting from
  `latinmodern-math.otf` by original GID (`src/mathtex.rs`
  `TexMathMetrics::glyph`/`otf_gid`); exact export via
  `flashtex-pdf-exact from-v2` (fonts matched by sha256, GID-preserving CFF
  subset in `crates/pdf/src/cff.rs`). `TexMathMetrics::glyph` returns `None`
  for any char with no cm slot, so a parsed U+211D would be a missing glyph.

**Reference fonts.** `fixtures/real-world/hw1/reference-mactex2026.pdf` embeds
`QSLEFF+MSBM10` (Type 1 subset) for the double-struck letters, CMMI/CMSY/CMEX/CMR
for math, and cm-super `SFRM1095`/`SFBX*`/`SFTI1095` for text.

**Facts per option.**

| | (a) AMS msbm10 Type 1 | (b/c) Latin Modern Math U+2115/2124/211A/211D | (b') STIX Two Math |
|---|---|---|---|
| Already in repo | No | **Yes**: `apps/mac/Fonts/latinmodern-math.otf` sha256 `6075562b…` = font-engine manifest `lm.math`; bundled by `make-app.sh` | No (TeX Live has it, 838 KB) |
| Glyphs present | N Z Q R in msbm10.pfb (34 694 B) | GIDs 3506/3518/3509/3510, plus U+1D538 block | GIDs 1230/1238/1233/1236 |
| Advance (em) N/Z/Q/R | 0.722/0.666/0.777/0.722 (msbm10.afm) | 0.722/0.667/**0.667/0.639** | 0.692/0.648/0.751/0.729 |
| Licence | SIL OFL 1.1, © AMS; `msbm5–10` are Reserved Font Names (`doc/fonts/amsfonts/OFL.txt` l.136–141). OFL-FAQ 1.1: embedded subsets in documents need no licence text | GUST Font License (already shipped with text `GUST-FONT-LICENSE.TXT`) | OFL 1.1 |
| font-engine support | **None** for Type 1 programs (README "Not implemented") | CFF OpenType load, cmap 12, MATH: yes; CFF subsetting no (engine), yes in `crates/pdf/src/cff.rs` | same as LM Math |
| crates/pdf support | `type1.rs` parse+subset, `exact.rs` `FontProgram::Type1` exist | runtime-v1 `embed.rs` embeds the whole CFF table (no subset; large); exact route subsets | same |
| Mac preview | Preview draws display-list glyphs by GID from bundled OTFs; a `.pfb` has no GIDs and would need a new resource kind (verify whether CoreText can still load Type 1 on current macOS; `crates/font-resources` has Type 1 outline decoding) | No new resource; face already loaded | new pinned file |
| Bundle (#36) | New pfb + `msbm5/7/10.tfm` into the pinned texmf tree, pins, OFL text; `make-app.sh` copies only pinned `*.otf` to `Fonts/` and refuses unpinned files | None (already bundled). Note `make-app.sh` on main already bundles a pinned texmf tree for GH36 though the issue is still open | new pin + OFL |

**Likely code paths** (not sized precisely): parser arm in
`crates/compiler/src/math.rs` `command_atom` (letter argument → Unicode
double-struck char; refuse non-letters) and re-vendoring into
`crates/render-pipeline/vendor/compiler`; render route
`crates/render-pipeline/src/mathtex.rs` `glyph`/`otf_gid` (a double-struck
family: metrics from msbm TFM for (a) or from LM Math hmtx + `src/cff.rs`
bounds for (b)); compiler route `crates/compiler/src/export.rs` (an embedded
outcome instead of Symbol-only), `layout.rs` `face` (a non-Core14 face), and
`crates/pdf` runtime-v1 embedding (whole-table CFF today).

**Falsifiers noticed.** (b) visual parity with the MSBM10 reference fails for Q
and R (0.110/0.083 em narrower, different design), which can move HW1 line
breaks; whether that is acceptable is the decision. (a) fails if the Mac preview
cannot draw a Type 1 program, or if the v2 display list's GID/sha256 font model
cannot carry a name-keyed Type 1 face without a schema change. For the compiler
route, any option fails the "no silent substitute" rule unless the protocol.rs
warning and export table gain a real embedded outcome; do not relax the guard.

## Question 2: body text face (decision brief)

**Tier-1 decision** for the Commander: which producer/face the Mac app's
default document route uses is outward-facing and touches pinned evidence.

### What text uses today, and why

- The HW1 Times-like rendering comes from **`flashtex-compiler`**, which the
  Mac app attaches by default when it is bundled
  (`apps/mac/Sources/FlashTeXMac/ShellModel.swift` ~l.366–377, "A compiler
  shipped inside the .app bundle attaches by default"). `flashtex-render` is
  only attached through File > Attach Render Pipeline
  (`attachDiscoveredRenderPipeline`). `PreviewFonts.face(forProducer:)`
  (`Fonts.swift` l.116) picks `.times` for any producer whose name lacks
  "render".
- `crates/compiler/src/layout.rs` `face()` measures with font-engine
  `Core14Face` (Adobe AFM widths + KPX kerning): Times-Roman body,
  Times-Bold headings, Times-Italic, Symbol for math symbols; math letters and
  digits are Times-Roman (`math_font`). `protocol.rs` `font_json`/
  `font_json_literal` report exactly those family names in `font-hints-v1`.
- Why: FT-002 rev 6 (commit 1fcb6823) replaced a hand-typed Core 14 table with
  font-engine so the compiler stays **file-free, offline and deterministic**,
  and matches its export: `crates/pdf` runtime-v1 writes base-14
  Times-Roman/Symbol, not embedded (`crates/pdf/src/lib.rs` header), and the
  export guard/warnings in `export.rs`/`protocol.rs` are defined against that
  repertoire. It was never a typographic choice; font-engine's README records
  the Commander's later policy that the pipeline default is Latin Modern.
- The reference is not LM either: `reference-mactex2026.pdf` embeds cm-super
  `SFRM1095`/`SFBX*`/`SFTI1095` (T1 Computer Modern at 11pt) and CMR/CMMI/CMSY
  for math. LM Roman is the metric/design stand-in the project already pins.
- Measured magnitude: on an English prose sample LM Roman 10 hmtx advances are
  **10.6 % wider** than Times-Roman AFM (58 683 vs 53 046 units, no kerning).
  Any switch changes line breaks and can change page count.

### What already exists for Latin Modern text

- `flashtex-render` (`crates/render-pipeline`) already typesets text in LM with
  **TFM metrics** (`ec-lmr*`/`ec-lmbx*`/`ec-lmri*`, the same metrics pdfLaTeX
  +`lmodern` uses), optical sizes (t1lmr.fd boundaries, `fonts.rs`
  `FontSet::latin_modern_file`), bold and italic, and paints from the OTFs by
  original GID. On HW1 it already emits LMRoman10-{Regular,Bold,Italic} and
  LMRoman8 (`crates/pdf/docs/export-fidelity-hw1.md`).
- Export: `flashtex-pdf-exact from-v2` subsets CFF by original GID
  (`crates/pdf/src/cff.rs`); no program is embedded whole.
- Assets: 22 LM Roman OTFs + `latinmodern-math.otf` + 29 rooted TFMs are
  tracked and hash-pinned in `apps/mac/Fonts` (`SUPPLEMENTARY-FACES.json`,
  `texmf/SUPPLEMENTARY-METRICS.json`), and `apps/mac/scripts/make-app.sh`
  bundles the rooted texmf tree (the GH36 fix is on main; the issue is still
  open on GitHub). GUST licence text ships alongside. No new font or licence is
  needed for either option below.
- font-engine: `manifest.rs` `pinned_latin_modern()` pins lmroman10
  regular/bold/italic/bolditalic, lmsans10, lmmono10, lm.math; CFF OpenType
  load, GPOS kern, GSUB liga work; engine-side CFF subsetting does not
  (`subset` returns Unsupported for CFF), and no TFM parsing.

### Options

**A. Make `flashtex-render` the Mac app's default producer (text already LM).**
- Change: default-attach branch in `ShellModel.swift` init (prefer
  `locateRenderPipeline()` when bundled; keep compiler as explicit fallback /
  `FLASHTEX_AUTOATTACH` override). `PreviewFonts` already switches to
  `.latinModern` by executable name. Small (one file, tens of lines) plus
  Mac tests that assume the compiler is the default.
- Prerequisite (the real cost): re-vendor `crates/render-pipeline/vendor/compiler`.
  Its PIN `49e6eb43` lacks 15 compiler commits on main, including `\text`
  (f3379df8), amsmath gather/align (3958ddeb), long implication (787bf7a2),
  center/quote (fab74578). The vendored `Nucleus` has only
  Symbol/Fraction/Radical; main adds Text/Space/Matrix, so
  `render-pipeline/src/typeset.rs` `convert_math*` and `incremental.rs` must be
  adapted before it builds. That is render-pipeline-owner work.
- Metrics source: TFM (closest to the reference). Subsetting: done (exact route).
  Bold/italic: done. Pinned compiler fixtures: **unaffected** (compiler
  unchanged). Render-pipeline evidence must be re-run after the re-vendor.
- Reversibility: high; a default flag, both producers stay bundled.

**B. Port `flashtex-compiler` layout from Times to LM Roman.**
- Metrics source, two sub-choices: (B1) generated static tables from the pinned
  OTF hmtx/GPOS (or `ec-lmr*` TFM) in font-engine, the `tools/gen_tables.py` →
  `generated.rs` pattern, keeping the compiler file-free; or (B2) runtime
  `PinnedFontSet::load` of the OTFs, which makes the compiler depend on font
  files and discovery (new failure mode, tests that cannot skip).
- Code paths: `crates/compiler/src/layout.rs` (`Font` is a re-export of
  `Core14`; `face`, `shape_text`, `word_space`, `math_font`, every
  `Font::TimesRoman/TimesBold` use), `protocol.rs` `font_json`/
  `font_json_literal` (family strings), `math.rs` (Text nucleus font),
  `export.rs` + `protocol.rs` export warnings (the base-14 guard no longer
  describes the export), font-engine (new LM text face type or generated
  tables, optical sizes for 12/14.4/17 pt headings), `crates/pdf` runtime-v1
  (`font-hints-v1` already maps `Latin Modern*` → lmroman10-* but embeds each
  used face's **whole CFF table**, ~250 KB for four faces, and its
  `embed.rs` `candidate_paths` never looks inside the .app bundle, so on a
  no-TeX Mac hints degrade to Times with a warning unless the shell passes
  `FLASHTEX_LM_DIR`), `Fonts.swift` `face(forProducer:)`. Math stays
  Times-Roman letters + Symbol unless ported too, giving mixed LM text / Times
  math. Estimate: 6–10 files across three crates plus the Mac shell, several
  hundred lines, plus fixture regeneration.
- Pinned fixtures: `crates/compiler/tests/pinned/{layout-capabilities,no-capabilities}.compile-results.jsonl`
  are compared as raw bytes; every `x_pt`, line break and font family string
  changes, so all 16 pinned results must be regenerated and re-reviewed. The
  16 `crates/compiler/fixtures/*.{legacy,negotiated}.json` (90 `x_pt`),
  `kerning.*` (AFM KPX-specific), and assertions in `tests/acceptance.rs`,
  `layout_capabilities.rs`, `corpus_gate.rs`, `references_and_figures.rs`
  change too; Mac `PreviewTextCacheTests` pins `.times` for the compiler.
  font-engine `tests/pinned.rs` is unaffected.
- Reversibility: medium-low; it rewrites pinned evidence and adds a third LM
  measurement path (font-engine Core14, render TFM, compiler LM) that can
  drift from render-pipeline's TFM layout, which is what FT-002 rev 6 set out
  to prevent.

**C. Keep Times on the compiler route; state it as the `times`-package route.**
- No change; the audit finding stays open. Fully reversible.

### What would falsify each

- **A** fails if, after the re-vendor, `flashtex-render` on HW1 reports more
  diagnostics than `flashtex-compiler` on main, or its page count stops matching
  the 3-page reference; if the bundled app with host TeX removed cannot load
  required TFMs (GH36 regression); if the durable preview-controller route
  (`crates/preview-controller` takes a generic `compiler_path`) or the edit
  ledger/incremental path does not accept `flashtex-render` as a drop-in
  runtime-v1 worker; or if typing latency on the render route exceeds the
  200 ms warm-edit budget the compiler meets (28 ms p95 in 1fcb6823).
- **B** fails if LM hmtx-based greedy layout does not visibly move HW1 closer to
  the reference (it is not TFM, and the reference is cm-super, not LM); if
  runtime-v1 export on a no-TeX Mac silently degrades hints to Times (the
  README says it warns, which would re-introduce the diagnostic class); if
  mixed LM text with Times/Symbol math is judged worse than all-Times; or if the
  compiler's warm-edit budget or file-free determinism cannot be kept (B2).
- **C** is falsified by the user's visual acceptance requiring a Computer
  Modern look on the default route.

### Recommendation

Option **A**, gated on re-vendoring the compiler into render-pipeline first.
It is the only route where LM text already uses the reference's own TFM
metrics, optical sizes, bold/italic, GID-preserving subset export and pinned,
bundled assets, so the change to the app is a small, reversible default switch
and no compiler pinned fixture moves. B spends several hundred lines to build a
second, less faithful LM layout inside the compiler and rewrites its raw-byte
evidence, while still leaving math in Times/Symbol. The key blocker for A is
the 15-commit vendor lag and the exhaustive `Nucleus` adapter in
`render-pipeline/src/typeset.rs`; until that lands and HW1 diagnostics on the
render route are no worse than on the compiler, keep C.
