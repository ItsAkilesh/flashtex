# mac-bibliography-kinds handoff

Agent / task / branch: `mac-bibliography-kinds` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / Gap 3 "explicit bibliography kind
persistence across native project close/reopen after original disk deletion" /
`agent/mac-bibliography-kinds/bibkinds` (base `origin/agent/mac-claude-a/mac-shell`
5bc3fc0f; helper contract read from origin/main 6472a5d2, read-only).

State: in progress (coverage audit done; implementation starting)

Owned paths: `apps/mac/Sources/FlashTeXMac/DocumentKinds.swift`,
`apps/mac/Tests/FlashTeXMacTests/DocumentKindsTests.swift`,
`coordination/mac-bibliography-kinds.md`, `coordination/agents/mac-bibliography-kinds.json`.
Small non-retained edit: `PreviewControllerClient.Config.bibliographyPaths`
(`PreviewControllerClient.swift`). Parent-retained files are NOT committed here;
their hooks are unified diffs in this handoff.

## Coverage audit (2026-09-12T13:35Z, mandatory first step)

Searched `apps/mac/Sources/FlashTeXMac/*`, `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`,
`coordination/*.md` for `bibliograph`, `document_kind(s)`, `bibliography_paths`,
`.bib`, `refs.bib`, `kind`:

- `apps/mac/Sources/FlashTeXMac/PreviewControllerClient.swift` `Config.json()`:
  emits `session_id/project_id/entry_path/project_root/private_ledger_root/
  store_paths/compiler_path/compiler_max_frame_bytes` only — NO `bibliography_paths`.
- `apps/mac/Sources/FlashTeXMac/ProjectDocuments.swift`: `open_document` payloads
  (lines 732, 1081) carry `path/source_versions/membership_generation` only — NO
  `document_kind`; `refreshSnapshot()` (line 1123) reads `source_versions` and
  `membership_generation` only — `document_kinds` is dropped. `detachDocument`
  (line 782) is typed-agnostic. Nothing persists kinds.
- `apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift` `attachController`
  (line 71, parent-retained): builds the Config without bibliography declarations,
  so a reopen re-imports a retained `.bib` ledger as `latex` (helper
  `Controller::open_with_bibliography` with an empty list).
- `apps/mac/Sources/FlashTeXMac/DocumentFiles*.swift`: no kind handling.
- `apps/mac/Tests/FlashTeXMacTests/NavigationTests.swift:743` only mentions a
  `thebibliography` environment as navigation fixture text (not kinds).
- `apps/mac/Tests/FlashTeXMacTests/ProjectDocumentsTests.swift`
  `testHelperRouteOpensWithExactSnapshotAndDetaches`,
  `testHelperSyncAttachesDocumentsOpenedBeforeTheController`,
  `testHelperRouteSavesThroughExportAndFlushesOnSwitch`: untyped open/detach with
  exact snapshot; no `document_kind`, no close/reopen, no disk deletion.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md`: zero matches.
- `docs/evidence/*.md`: zero matches.
- `coordination/mac-bibliography.md`: `crates/bibliography` parser lane (Rust data
  layer), unrelated to helper document kinds.
- Helper side (read-only, origin/main 6472a5d2 = local crate for kinds):
  `crates/preview-controller/docs/source-plans.md` states declarations "must be
  supplied again on reopen", "the application owns persistence of its startup
  configuration", "persist the bibliography paths from this [`document_kinds`]
  map ... do not reconstruct kinds from file extensions". Helper tests cover
  reopen-from-durable-source after disk deletion only when `bibliography_paths`
  is supplied. "No live native application adoption has been verified yet."

Conclusion: the gap is entirely uncovered on the Mac side. Uncovered part to
implement: Config `bibliography_paths`; explicit typed `open_document
{document_kind:"bibliography"}`; `document_kinds` adoption from `snapshot`;
application-owned persistence of the declared bibliography paths per project
(entry-keyed, next to the helper's private ledger root); reopen supplying them;
read-only kind indicator; real-helper close/delete/reopen test.

## Durable checkpoint

- Branch `agent/mac-bibliography-kinds/bibkinds` @ 5bc3fc0f (base), worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a0c5c1743911589cd`.
- Dirty files: this handoff, registration JSON.
- Consumed main SHA (read-only contract): 6472a5d2.
- Helpers: `/Users/jay3332/Projects/flashtex/crates/{compiler,preview-controller}/target/release/*`
  (contain `bibliography_paths`/`document_kinds` strings; built 09:30 local).
- Next commands: write `DocumentKinds.swift`, `DocumentKindsTests.swift`; edit
  `PreviewControllerClient.Config`; `cd apps/mac && swift build`; run
  `swift test --filter DocumentKindsTests` with helper env vars; commit; push.
- Decisions: never infer kind from extension/content; kind is only what the
  helper's `document_kinds` reports after an explicit declaration; persistence is
  keyed by entry path under `ShellModel.controllerLedgerRoot(for:)`.
- Staffing/billing: shared Claude Max quota with parent; no purchases.

Updated: 2026-09-12T13:36Z
