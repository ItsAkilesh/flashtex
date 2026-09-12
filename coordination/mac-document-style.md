# mac-document-style handoff

Agent / task / branch: mac-document-style (Claude Code subagent, parent
mac-claude-a, machine mac-m1max-a) / issue #2 dispatch: original Rust document
style model (`flashtex-document-style`) / `agent/mac-document-style/style-model`
State: ready for integration
Owned paths: `crates/document-style/**`, `coordination/mac-document-style.md`,
`coordination/agents/mac-document-style.json`
Main integrated through: 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863 (branch
base; origin/main c89ca86 reviewed, no overlap — see below)
Ready behavior:
- `Stylesheet::article(ClassOptions)`: letter/a4/a5/legal x 10pt/11pt/12pt;
  `page_layout()` reproduces pdflatex's \textwidth/\textheight/\oddsidemargin/
  \evensidemargin/\topmargin/\marginparwidth for all 12 combinations (exact,
  via \@settopoint truncation and integer line count).
- `Geometry` override with geometry.sty v5.9 completion rules; 10 configs
  verified against pdflatex (margin=1in on letter -> 469.755pt = 468bp).
- Size table from size1x.clo; \parindent 15/17/17.62482pt; \parskip 0+1pt;
  CM ex/em per size; \@startsection skips (in body ex), run-in \paragraph.
- Block style tree with inheritance (size/leading/indent/alignment/weight
  inherit; space_before/after/first-line indent do not); lists per level
  (\leftmargin, \labelwidth, \topsep, \parsep, \itemsep, \partopsep);
  StyleDelta overlay; hand-written JSON round-trip and deterministic export.
- Oracle test: heading and list baseline gaps measured from pdflatex
  (\pdfsavepos) at 10/11/12pt match the model to 1e-3pt.
Incomplete behavior: twocolumn/twoside/landscape, headers/footers, floats,
footnotes, display-math skips, geometry scale/ratio/lines/includemp; no
consumer wired (compiler untouched by design).
Interface changes and required consumer actions: none. New standalone crate;
runtime-v1 untouched. README proposes the compiler/PDF/preview adapter
(pt -> bp -> bp_2pow20 at emission) for negotiation.
Validation: `cd crates/document-style && cargo test` -> 18 + 1 + 1 doctest
pass; `cargo clippy --all-targets` 0 warnings; `cargo fmt --check` clean.
Oracle: BasicTeX /Library/TeX/texbin pdflatex (TeX Live 2026, pdfTeX 1.40.29);
probe documents reproduced in the README.
Needs from others: Commander/orchestrator to assign an FT number and publish
an assignment file (no `ack` was possible: none exists for this dispatch);
compiler lead (FT-002) to decide when to consume the crate; integration owner
to decide on a Cargo workspace.
Next action: await review; on request wire a compiler adapter behind a flag.
Peer revisions reviewed and adaptations:
- origin/main 254193c (c89ca86 + supervisor scripts only; through fb6b367 "integrate original Rust runtime,
  rendering, fonts"): crates/compiler/src/layout.rs constants unchanged
  (612x792, 72pt, PARAGRAPH_GAP_PT 6, headings 17/14); no crates/document-style
  on main; new docs/contracts/rendering-v2-proposal.md fixes bp_2pow20
  top-left coordinates -> README adapter note added. No code adaptation.
- issue #10 oracle findings (+14.75pt after headings, 6pt paragraph gap):
  confirmed by this crate's oracle test (article \parskip 0; \section after
  gap 11.88pt at 12pt).
Resource: allocation claude-mac20x-document-style (parent mac-claude-a's Max
20x quota); usage not observable from this session.
Updated: 2026-09-12T06:05Z
