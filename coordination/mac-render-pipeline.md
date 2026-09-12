# mac-render-pipeline handoff

Agent / task / branch: mac-render-pipeline (Claude Code subagent Opus, parent mac-claude-a,
mac-m1max-a) / unified render pipeline (`crates/render-pipeline`, worker `flashtex-render`) /
`agent/mac-render-pipeline/unified`
State: ready for integration review (crate + docs); siblings still vendored until they merge
Owned paths: `crates/render-pipeline/**`, `docs/proposals/rendering-abi.md`,
`coordination/mac-render-pipeline.md`, `coordination/agents/mac-render-pipeline.json`
Main integrated through: 7fea005b0611e2cc565ec38b9a705eff7cc4532c (reviewed; compiler
`9026d8a` vendored from it — the branch itself is not merged with main, it only carries the
crate and docs, so a merge is trivial: no shared files touched)

Ready behavior:
- `cargo build --release && cargo test --release` in `crates/render-pipeline`: 35 tests green
  (unit 19; golden v1 spans incl. multi-byte and `\input` documents 3; v2 envelope /
  v1 consistency / math rules / page-break determinism 5; LaTeX structure 6; CLI e2e 2).
- `flashtex-render`: runtime-v1 worker; `payload.layout_capabilities` negotiation per
  `docs/contracts/runtime-v1-layout-capabilities.md` (validated list, accepted subset echoed,
  typed `rule` items / `font` hints only when accepted, U+2500 legacy otherwise, identical
  output for the same accepted set); `Span.document` honoured in every item; fail-closed on
  unknown protocol version/type; `--v2` display list; `--pdf` via the pdf sibling's negotiated route.
- Default face Latin Modern by optical size; Times only when the document selects it.
  Fonts resolved from `FLASHTEX_FONT_DIRS`, `--font-dir`, `FLASHTEX_LM_DIR`, an app-bundle
  `Fonts` dir, MacTeX/BasicTeX 2025/2026 and Debian paths; missing faces are error diagnostics.
- TeX page builder (`pagebuild.rs`): penalty costs (club/widow 150, `\nobreak` after headings,
  `\predisplaypenalty`), `\topskip`, `\maxdepth`, per-block `\baselineskip`, `\nointerlineskip`,
  `\raggedbottom`, `\newpage`/`\clearpage`/`\pagebreak` (recovered from source).
- LaTeX structure on top of compiler 9026d8a: numbered sections (`secnumdepth`, `\quad`),
  `\ref`/`\pageref` (bounded 3 passes), `equation` numbers flush right, `\[` opener line
  (long/short display skips as TeX §1199), figure captions as paragraphs.
- Evidence (`crates/render-pipeline/docs/oracle-evidence.md`): 18 corpus fixtures vs fresh
  pdflatex renders from **MacTeX 2026 full (pdfTeX 1.40.29)** (oracle only): word origins
  within 0.01–0.06 pt dx on all text fixtures, every word on the same page, first baselines and
  section spacing exact; residuals are documented (PDFKit font-box dy 1.15 pt constant, ink weight,
  math parser gaps, lists unsupported). Per-fixture worker time 1.2–9.3 ms cold, 0.07–4.2 ms warm.
- All 18 `--v2` envelopes pass main's `rendering-core` `validate_display` once the font
  `format` token is accepted (single deviation: `opentype-cff` vs `static-truetype`).

Incomplete behavior:
- Lists (hanging indent), `\left`/`\right`, Greek/control-word math symbols (compiler math parser),
  hyphenation, figures, tables, footnotes, page numbers: reported as diagnostics/limitations.
- `--pdf` re-encodes text by character through the pdf sibling (no glyph-run API yet).
- Sibling crates are vendored pins (`vendor/VENDORING.md`): compiler main 7adb021/9026d8a,
  font-engine f418238, paragraph-layout 70209e2, math-layout db90047, pdf 4bd8c2e,
  document-style bfc980d. Repoint `Cargo.toml` at `../<name>` when they land on main.

Interface changes and required consumer actions: none to existing contracts. Requests to
siblings/compiler/schema are in `docs/proposals/rendering-abi.md` (schema: accept
`opentype-cff`; font-resources: `inspect_opentype_cff`; compiler: style scopes, gaps, preamble
facts, `\newpage` as block boundary, number only `equation`, math symbols; paragraph-layout:
penalty page builder; pdf: glyph-run entry point).

Validation: `cargo test --release` 35 passed at the handoff SHA; oracle harness run
2026-09-12T07:41Z (table + provenance in `docs/oracle-evidence.md`); rendering-core validation
against main 7fea005.

Attach to the Mac app:
`FLASHTEX_COMPILER=$REPO/crates/render-pipeline/target/release/flashtex-render` (fonts found in
MacTeX 2026 by default; set `FLASHTEX_FONT_DIRS=<lm dir>:<lm-math dir>` elsewhere).

Needs from others: schema/rendering-core decision on `opentype-cff`; compiler items above;
integration of the sibling branches so the vendor directory can go.
Next action: after integrator review, un-vendor siblings as they merge; lists and math symbols
once the compiler exposes them.
Peer revisions reviewed and adaptations: main 7fea005 (rendering-core/font-resources/schema:
deviation register written; compiler 9026d8a: numbering/labels/captions adopted, equation-count
bug worked around); a949f5b/cab39a2 layout capabilities: implemented and tested;
visual-oracle 63f0cf5 harness: used for evidence, its `--embed-font auto`/no-capabilities export
route noted; sibling tips f418238/70209e2/db90047/4bd8c2e: re-vendored, API changes adopted
(`LineBreakParams.hfuzz/hbadness`, `Stats.emergency_pass_used`, `Lines.diagnostics`,
pdf `CompileResult.capabilities`/`Item` enum).
Resource state: shared 20x Max quota on mac-m1max-a (usage not observable from a subagent).
Updated: 2026-09-12T08:05:00Z
