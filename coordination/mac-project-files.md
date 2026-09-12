# mac-project-files handoff

Agent / task / branch: mac-project-files (Claude Code subagent, parent
mac-claude-a, machine mac-m1max-a) / issue #2 dispatch "project file layer"
(no FT number, no assignment file yet, so no `coord.py ack` is possible) /
`agent/mac-project-files/graph`
State: ready for integration (crate + follow-up 1 crash recovery + follow-up 2
integration proposal); awaiting parent agreement on the native integration
point before any `apps/mac` change.
Owned paths: `crates/project-files/**`, `coordination/mac-project-files.md`,
`coordination/agents/mac-project-files.json`
Main integrated through: 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863 (base);
reviewed 254193c175534e78bfd9e5206b4850b86123cbe6 — no overlap with owned
paths; merge deferred to integration (branch applies cleanly: only additions).
Ready behavior (`crates/project-files`, zero external crates, edition 2024):
- `ProjectPath` normalization (relative, `/`, no `..` escape, no
  absolute/drive/backslash/colon/NUL).
- Reference scanner with UTF-8 byte spans: `\input`, `\include`,
  `\bibliography` (per item), `\addbibresource`, `\includegraphics`, bare
  `\input name`; skips comments/`\verb`/verbatim envs; flags macro args.
- `ProjectGraph::discover(_with overlay)`: DFS in reference order, implicit
  extension rules against the root, diamond dedup, diagnostics for missing
  file (with `tried`), invalid/escaping path, symlink escape, cycle (chain),
  unresolvable macro argument, invalid UTF-8, read error, depth > 64; runtime-v1
  `documents()` (Tex only, entry first), `documents_including_bibliography()`,
  `compile_payload()`, `compile_envelope()`.
- Hand-written SHA-256 (FIPS vectors incl. 1M `a`, streaming chunkings).
- `RevisionTracker`: per-file revision by hash; deterministic project revision.
- `save_atomic`: temp+fsync+rename in-dir, dir fsync best effort, permissions
  preserved, `SaveReceipt{path,bytes,sha256,mtime}`; refuses on
  Modified/Deleted/AlreadyExists with ours/theirs/mtime/size unless `force`.
- `Snapshot`/`diff`: mtime+size gate, hash only when moved; Created/Modified/
  Deleted; `Conflict{ModifiedExternally|DeletedExternally|Both, local_dirty}`;
  `record_own_write`; `Poller` (no FSEvents dependency).
- `RecoveryJournal`: `<root>/.flashtex/recovery/<sha256(path)>.json`, atomic,
  hash-verified on read, list/load/check/restore_to_disk/discard with explicit
  conflict rules; malformed entries reported, never deleted.
- README: API, guarantees, non-guarantees (no cross-process locking; rename
  atomicity is the filesystem's; mtime granularity; not the compiler), native
  integration proposal.
Incomplete behavior: no JSON Lines/FFI adapter or binary yet (proposed in
README step 2, pending parent agreement); no `apps/mac` changes (parent-owned).
Interface changes and required consumer actions: none. Produces runtime-v1
`compile` payloads unchanged; hashes are transfer-v1 SHA-256 hex.
Validation (rustc 1.99.0-nightly, cargo 1.99.0-nightly, this Mac):
- `cargo test`: 35 tests pass (12 unit, 10 graph, 6 save/watch, 4 recovery,
  3 sha256), temp dirs only.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean. `RUSTDOCFLAGS=-D warnings cargo doc`: clean.
Needs from others: assignment file / FT number from Commander for ack;
parent decision on process-vs-FFI integration (README proposal).
Next action: on parent agreement, add the JSON adapter + binary; otherwise
extend scanner coverage (`\import`, `\subfile`) as a bounded follow-up.
Peer revisions reviewed and adaptations:
- origin/main 1dd26c5 (base), b873340, c89ca86, 254193c (scripts/claude_worker.py supervisor only): AGENTS/PROJECT/coord tooling,
  FT-021..026 assignments (none for this worker), and new crates
  `project-index`, `edit-ledger`, `document-runtime`, `rendering-core`,
  `font-resources`, `conversion-jobs`. None touch `crates/project-files`.
  Adaptation: README gained a "Relationship to sibling crates" section —
  `documents()` + file revisions feed `project-index::replace_document`;
  `edit-ledger` owns live text/receipts per document while this crate owns
  the `.tex`-side graph/export/external-change layer (the ledger does not
  watch the exported file); `RecoveryJournal` is redundant for ledger-backed
  documents and kept for non-ledger buffers.
- crates/compiler `protocol.rs::path_is_safe` on main: rejects absolute,
  drive prefix, `..` segments; `ProjectPath` is stricter (also backslash,
  colon, NUL, control chars, symlink escape at discovery).
- origin/agent/mac-claude-a/mac-shell `DocumentFiles.swift`: single-file
  open/save via `String.write(atomically:)`, no hash/conflict check, entry
  always `main.tex`. README proposal targets it; no change made there.
- issue #8 (FT-002 file-aware `\input`): compiler resolves against the
  supplied document map with implicit `.tex`; `documents()` supplies exactly
  that map (real paths, overlay text) so no concatenation/relabeling occurs.
Resource: allocation claude-mac20x-project-files (parent's Max 20x quota);
per-call usage not exposed; no purchases.
Updated: 2026-09-12T06:12Z
