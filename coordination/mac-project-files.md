# mac-project-files handoff

Agent / task / branch: mac-project-files (Claude Code subagent, parent
mac-claude-a, machine mac-m1max-a) / issue #2 dispatch "project file layer"
(no FT number, no assignment file yet, so no `coord.py ack` is possible) /
`agent/mac-project-files/graph`
State: in progress
Owned paths: `crates/project-files/**`, `coordination/mac-project-files.md`,
`coordination/agents/mac-project-files.json`
Main integrated through: 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863
Ready behavior: none yet
Incomplete behavior: everything (crate skeleton being written)
Interface changes and required consumer actions: none. The crate produces the
runtime-v1 `documents` list (`{path,text}`, entry first) and does not change
any contract. Hashes follow transfer-v1 (SHA-256 of UTF-8 source, hex).
Validation: none yet
Needs from others: assignment file / FT number from Commander for ack.
Next action: sha256 + path normalization + reference scanner + graph, then
atomic save, snapshot/diff, recovery journal, tests, README.
Peer revisions reviewed and adaptations:
- origin/main 1dd26c5: base. `crates/compiler/src/protocol.rs::path_is_safe`
  rejects absolute paths, drive prefixes and `..` segments; `ProjectPath`
  normalization here is at least as strict (also rejects backslashes, NUL,
  and `..` that escapes the root after normalization).
- origin/agent/mac-claude-a/mac-shell `DocumentFiles.swift`: single-file
  open/save (`String.write(atomically:)`, no hash/conflict check, entry is
  always `main.tex`). The native integration proposal in the crate README
  targets replacing this once the parent agrees on the integration point.
- issue #8 (FT-002 file-aware input): compiler resolves `\input` against the
  supplied document map with implicit `.tex`; this crate is the layer that
  builds that map from disk plus unsaved buffers.
Updated: 2026-09-12T05:41Z
