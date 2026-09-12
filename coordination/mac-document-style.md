# mac-document-style handoff

Agent / task / branch: mac-document-style (Claude Code subagent, parent
mac-claude-a, machine mac-m1max-a) / issue #2 dispatch: original Rust document
style model (`flashtex-document-style`) / `agent/mac-document-style/style-model`
State: in progress
Owned paths: `crates/document-style/**`, `coordination/mac-document-style.md`,
`coordination/agents/mac-document-style.json`
Main integrated through: 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863
Ready behavior: none yet (registration checkpoint)
Incomplete behavior: page geometry (article defaults + geometry overrides),
font size table, block style tree with inheritance, section/list spacing,
StyleDelta overlay, hand-written JSON, tests, README
Interface changes and required consumer actions: none. The crate is a
standalone library; runtime-v1 and crates/compiler are untouched. A future
compiler/PDF adapter is proposed in the crate README for negotiation.
Validation: class values probed with BasicTeX pdflatex (TeX Live 2026,
pdfTeX 1.40.29) at /Library/TeX/texbin; the probe log is reproduced in the
crate README.
Needs from others: no assignment file exists for this task yet, so no `ack`
was possible; Commander to publish one if the task gets an FT number.
Next action: implement geometry + size tables, then tests, then README.
Peer revisions reviewed and adaptations: origin/main 1dd26c5 (layout.rs
constants 612x792 / 72pt / PARAGRAPH_GAP_PT 6 / heading 17 & 14); issue #10
oracle findings (+14.75pt after headings, 6pt paragraph gap vs \parskip 0).
No adaptation needed: this crate models the article values those findings
call for; the compiler owner decides when to consume it.
Updated: 2026-09-12T05:40Z
