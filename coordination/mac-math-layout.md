# mac-math-layout handoff — FT-020 rev 3

- Agent / task / branch: `mac-math-layout` (Claude Code subagent, parent
  `mac-claude-a`, machine `mac-m1max-a`) / FT-020 rev 3 "integrate declared
  math visual cases and tune spacing, scripts, rules, roots and delimiters
  against pinned TeX oracles" / `agent/mac-math-layout/math-boxes`
- State: ready for integration (rev 3 acknowledged 2026-09-12T06:32Z and
  implemented; one acceptance item — raster overlays against a live oracle —
  could not be regenerated on this machine, see "Incomplete behavior").
- Owned paths: `crates/math-layout/**`, `coordination/mac-math-layout.md`,
  `coordination/agents/mac-math-layout.json`.
- Main integrated through: `462fb27` (merged into the branch at this
  checkpoint; main carries `crates/pdf` at `4bd8c2e`, the commit the gate
  pins, and nothing under `crates/math-layout`).
- Ready behavior:
  - Declared visual corpus: `crates/math-layout/fixtures/visual/` — 15 cases
    (`.tex` + `.meta.json`), seven oracle formulas A–G plus nested scripts,
    display `\int`, `\left[\frac{a}{b}\right]^2`, `\hat{\imath}+\vec{x}`,
    `\sum\limits` inline, `\prod` with scripts inline, mixed text/math line,
    nested fraction sum. `src/corpus.rs` declares the bodies (lookup, not a
    parser); `flashtex-math-corpus` serves them as a runtime-v1 compile server
    with `layout_capabilities: ["rules-v1","font-hints-v1"]` accepted (typed
    `rule` items per `docs/contracts/runtime-v1-layout-capabilities.md`, one
    `text` item per glyph with a font hint).
  - Structural gate: `tools/structural_gate.py --regress
    fixtures/visual/structural-baseline.json` with `fixtures/visual/thresholds.json`
    (0.01 bp per case, 0.05 bp placement, 0.001 bp regression tolerance).
    Pinned oracle `fixtures/visual/oracle-geometry.json` (pdfTeX 1.40.29,
    TeX Live 2026, run of 12 Sep 02:43, keyed by fixture SHA-256) is used when
    `pdflatex` is absent; `--oracle auto|pinned|pdflatex`, `--pin-oracle DIR`.
    Result: 15/15 pass, corpus max |Δ| 0.0055 bp, placement ≤ 0.0005 bp; a
    baseline regenerated from the pin is byte-identical to the committed one.
  - Preview/PDF box parity: `tools/box_parity.py` + `tools/coretext_boxes.swift`
    — the same compile_result drawn by the pinned `crates/pdf` (`Td`/`re`)
    and by a CoreText draw agree with it item by item: 15/15, PDF writer max
    |Δ| 0.00096 pt (3-decimal rounding), CoreText 0.00000 pt, tolerance 0.05 pt.
  - `tools/run_visual.sh`: exports the visual-oracle harness at `db18236`
    (read-only `git archive`; the branch tip `78b9a64` is an untested WIP and
    is not pinned), builds `crates/pdf` at `4bd8c2e`, runs the harness, the
    parity gate and the structural gate, writes `docs/visual-evidence/<stamp>/`.
    Latest: `docs/visual-evidence/20260912T070234Z`.
  - Tuning from corpus deltas (before → after): accent centring on `char_box`
    width incl. italic correction, 11-accents 0.9176 → 0.0055 bp; text
    operators keep the last italic correction (tex.web §752). Nothing else
    exceeded pdfTeX's own rounding, so nothing else was tuned.
  - Explicit unsupported table in `README.md` ("Unsupported / limitations").
- Incomplete behavior:
  - Raster overlays against the reference could NOT be regenerated: no
    `pdflatex` on `mac-m1max-a` (BasicTeX removed, MacTeX pending). The
    harness report records every engine as unavailable; `raster-thresholds.json`
    values remain declared placeholders until the first oracle-backed run.
    The harness's own export-vs-preview-equivalent row reads DIFFERENT by
    design at `db18236` (Times-only draw, typed rules skipped) — that is why
    `box_parity.py` is the parity gate.
  - Parity faces differ: PDF side embedded LMRoman10 from a local LM directory
    (`FLASHTEX_LM_DIR`); the CoreText side had no system Latin Modern and drew
    Times at the same origins. Origins/rules are gated; outlines are not.
  - Everything in the README's unsupported table (no `\mathchoice`, matrices,
    `\middle`, family switches, `\text`, phantoms, OpenType MathVariants, …).
- Interface changes and required consumer actions: none to existing crates
  or contracts. The corpus compiler consumes `rules-v1`/`font-hints-v1` as
  specified on main; `crates/pdf` `4bd8c2e` was pinned unmodified.
- Validation: `cargo test` 37 passed (10 unit incl. corpus lookup/placement,
  27 golden); `cargo clippy --all-targets` clean; `cargo fmt --check` clean;
  `tools/run_visual.sh` exit 0 (parity 0, structural 0, diff 0 with no
  reference); evidence directory committed.
- Needs from others: a TeX installation on this machine (MacTeX) to regenerate
  the raster references and set measured raster thresholds; FT-018 owner —
  OpenType `MathVariants` and `MathFontMetrics` on `Face` (unchanged request).
- Next action: on MacTeX arrival, `tools/run_visual.sh` with an oracle, set
  `raster-thresholds.json` from measured numbers with 10–20% headroom, and
  `--pin-oracle` to confirm the pin against a fresh pdflatex run.
- Peer revisions reviewed and adaptations:
  - `origin/main` `462fb27`: `crates/pdf` `4bd8c2e` merged (rules-v1 +
    font-hints-v1) — identical to the pinned writer, no adaptation; no
    contract change. Merged into the branch.
  - `origin/agent/mac-visual-oracle/reference-raster` `78b9a64` (WIP): diff.py
    gains projection-based registration and burned-in footers; untested, so the
    gate stays pinned at `db18236`. Not edited.
  - `origin/agent/mac-pdf/pdf-output` `4bd8c2e`: `FLASHTEX_LM_DIR` and hint
    resolution read from `crates/pdf/README.md` and `embed.rs`; run_visual.sh
    passes the variable through.
- Resource state: Claude Max 20x plan on `mac-m1max-a` (allocation
  `claude-mac20x-math-layout`); this resumed session ~50 min; quota/usage
  unknown from this session.
- Updated: 2026-09-12T07:15Z
