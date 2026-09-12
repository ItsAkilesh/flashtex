# mac-pdf handoff — FT-009 PDF output

- Updated UTC: 2026-09-12T04:30Z
- Agent / parent / machine alias: `mac-pdf` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: FT-009 rev 1, "original Rust PDF output
  from runtime-v1 positioned pages". Serves acceptance gate 5 (a real exported
  document). Owned paths: `crates/pdf/`, `coordination/mac-pdf.md`.
- Branch / code revision / main integrated through:
  `agent/mac-pdf/pdf-output` / see commit / `25a92a9`
- State: ready for integration
- Ready behavior and evidence:
  - `crates/pdf` (`flashtex-pdf`, edition 2024, zero dependencies) writes PDF 1.4
    by hand from a runtime-v1 `compile_result`: one page object per page with
    the page's own `MediaBox`, one `BT … Tj … ET` per text item at
    `(x_pt, height_pt - baseline_y_pt)`, xref table, trailer. Library API
    `render_pdf` / `render_envelope` returning `{ bytes, warnings }`; CLI
    `flashtex-pdf [in.json] --out out.pdf [--verify]`.
  - `cargo test`: 20/20 pass (11 unit, 9 integration). Covers fixture page count
    and MediaBox, a 2-page synthetic result, multiline baselines, WinAnsi
    encoding (`é` → `0xE9`), unrepresentable chars (`∫`, `😀`) → `?` + warning,
    delimiter escaping, unsupported item kinds, bad envelopes, determinism, the
    CLI, an independent xref self-check, and (macOS) `sips` opening the fixture
    PDF and reporting 612 × 792.
  - Manual on mac-m1max-a: `/usr/bin/sips -g pixelWidth -g pixelHeight -g format`
    on the generated fixture PDF → `pixelWidth: 612.000`, `pixelHeight: 792.000`,
    `format: pdf`, exit 0. A two-page Unicode sample rasterised to PNG showed the
    heading, three baselines, accents, escaped parentheses, `?` substitution,
    and page two on a white page.
  - `cargo build --release` and `cargo clippy --all-targets` clean.
- Incomplete behavior / blockers / needs from others:
  - Base-14 Times-Roman with WinAnsiEncoding only; no font embedding, no bold/
    italic/math font selection. Non-WinAnsi characters become `?` with a warning.
  - Text items only; other kinds are skipped with a warning. No images/rules.
  - Word spacing in the PDF reflects the FT-002 compiler's placeholder glyph
    widths; this crate does not re-measure text.
  - Not wired into `crates/compiler` or `apps/mac` yet (`pdf_path` still null).
    Integration steps are in `crates/pdf/README.md`; wiring belongs to the
    compiler/Mac owners, not this task.
- Interface changes / consumer actions: none. Consumes runtime-v1 `compile_result`
  unchanged. Surfacing PDF warnings in the UI would need a contract addition
  (Commander's call); until then the worker can emit them as `warning` diagnostics.
- Reviewed peer revisions / resulting adaptations: `origin/main` 25a92a9 (branch
  base); `origin/agent/claude/compiler-foundation` 29221d8 `crates/compiler`
  README and `src/layout.rs` for the emitted item shape (one item per word,
  612×792, top-left origin, `round2` coordinates) — matched by writing absolute
  `Td` per item and formatting numbers to three decimals.
- Validation commands / results / artifact paths:
  `cd crates/pdf && cargo test` (20 passed);
  `cargo run --bin flashtex-pdf -- --out out.pdf --verify < ../../protocol/fixtures/compile-result.json`;
  `/usr/bin/sips -g pixelWidth -g pixelHeight out.pdf`.
- Resource pool / allocation: parent `mac-claude-a`'s Claude Max allocation on
  mac-m1max-a; this subagent's usage is deducted from it. Exact token totals not
  measured here.
- Dirty files / unpushed work / running jobs: none after push.
- Decisions: hand-written JSON reader instead of serde so the crate builds
  offline with no dependencies, matching `crates/compiler`; no background fill
  so export is white regardless of preview theme; one `BT/ET` per item so the
  content stream is trivially verifiable.
- Exact next action: Commander/integrator merges `agent/mac-pdf/pdf-output`;
  compiler or Mac owner wires `pdf_path` per `crates/pdf/README.md`.
