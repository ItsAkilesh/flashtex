# mac-document-files handoff

- Updated UTC: 2026-09-12T09:40Z
- Agent / parent / machine alias: mac-document-files (Claude Code subagent) /
  mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: issue #2 dispatch lane "Consume rooted
  save/reload/status helper with explicit conflict state", follow-ups
  "Preserve dirty source across lost export/reload replies" and "Multi-file
  open/detach/reopen native acceptance" (no FT number / assignment file, so no
  `coord.py ack`). Gate: full `apps/mac` suite green with the real helper.
  Owned: `apps/mac/Sources/FlashTeXMac/DocumentFiles.swift`,
  `apps/mac/Sources/FlashTeXMac/DocumentFilesClient.swift`,
  `apps/mac/Tests/FlashTeXMacTests/DocumentFilesTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/fake_project_files.py`, the
  `DocumentFileTests`/`DirtyOpenTests` classes (unchanged), this handoff and
  `coordination/agents/mac-document-files.json`. Also added, outside the
  listed owned paths because the lane cannot be met without it (the crate had
  no wire protocol, only the README proposal):
  `crates/project-files/src/bin/flashtex-project-files.rs`,
  `crates/project-files/tests/helper_bin.rs`, README section "JSON Lines
  helper". Original owner `mac-project-files` is done/idle; parent to route.
- Refill lane (coordinator, after the parent's `controllerSave`): branch
  `agent/mac-document-files/reload` from mac-shell 40d53b7, origin/main
  02de116 merged cleanly. Ready: `prepareReload()` → `ReloadReview` (disk
  snapshot read through the rooted helper or directly; bytes before/after,
  +added/−removed lines as a line-multiset summary, dirty notice, route,
  pinned hash); `confirmReload(review, dirty:) async` imports exactly that
  snapshot — through the controller's `reload {expected_revision,
  expected_sha256, expected_disk_sha256, user_approved:true}` when attached
  and ready (document adopted inside the reply waiter: durable state, editor
  revision mapping, buffer via `updateActiveText`, baseline; never submitted
  back; refusals leave the buffer untouched), else a direct re-read refused
  if the hash moved. `reloadFromDiskReviewed` (no modal),
  `reloadFromDiskInteractive` (summary modal), `resolveConflictPanel` shows
  the summary and disables Reload when nothing is readable; sync
  `reloadFromDisk` is direct-only and refuses to bypass a controller.
  `refreshDiskStatus() async` maps onto `controllerFileStatus` (`file_status`)
  comparing the disk hash with the editor's own baseline (typed-but-unexported
  edits are not an external change; `matches_source` falls back to the durable
  hash since the parent client only exposes `disk_sha256`), else
  `checkDiskStatus()`; conflict/notice mapping shared. Evidence: full suite
  384 pass / 6 env-gated skips / 0 failures with real compiler, pdf, bridge,
  edit-ledger, preview-controller (built from merged main) and project-files;
  `DocumentFilesControllerTests` (real controller: status unchanged/modified/
  deleted, review pinned, refused after a later change, imported with preview
  bound, recoverable buffer, sync reload refused) and 2 new direct tests.
  Not applied (parent-owned): menu/activation wiring and a one-line
  `controllerFileStatus` hash fallback — exact diffs in the final report.
- Branch / code revision / main integrated through:
  `agent/mac-document-files/rooted-helper` (from
  `origin/agent/mac-claude-a/mac-shell` 6b43a3a) / see JSON `code_revision` /
  origin/main b1f158c merged cleanly (main does not touch `apps/mac` beyond
  the mac-shell base; `crates/project-files` identical on main).
- State: ready for integration.
- Ready behavior and evidence:
  - Helper binary `flashtex-project-files --root DIR` (protocol
    `project-files-v1`): `ping`, `read`, `status {expected_sha256}`,
    `save {expected: new|any|hex, force}` → `{outcome: saved|conflict}`.
    Conflicts are payloads, errors are `{code,message}` (`refused` for
    symlink/escape/lock/too-large, `invalid_path`, `invalid_request`, `io`,
    `directory_sync`, `invalid_utf8`, `unsupported_operation`,
    `line_too_long`). Symlinked root refused at startup.
  - `DocumentFilesClient.swift`: `ProjectFilesV1` models + `ProjectFilesClient`
    over `LineProcessClient` (same plumbing as bridge/ledger), `outstanding`
    counter, `locate()` via `FLASHTEX_PROJECT_FILES` / bundle /
    `crates/project-files/target/{release,debug}`.
  - `DocumentFiles.swift`: `DocumentFilesState` (`model.files`, @Observable,
    associated object so the extension file stays self-contained): backend
    (`.helper(url)` / `.direct(reason)`), `status` text, explicit `conflict`
    (`DocumentConflict{url, kind, ours, theirs, size, mtime, viaHelper,
    summary}`), `lastDiskState`, `helperTimeout` (10 s default), late-reply
    notes, exit/restart counters, `detachHelper()`. One helper process per
    project directory (symlinks resolved in the directory only); restarted on
    root change, binary change, exit, or unanswered requests.
  - Save = compare-and-replace against `sha256(savedText)`
    (`baselineSha256`); never-on-disk buffer → expected new file; Save As →
    any (panel confirmed). Modified externally / appeared where none was
    expected → `files.conflict`, nothing written, buffer kept, `saveTex()`
    false. Resolution only by `overwriteOnDisk()` (force), `reloadFromDisk(dirty:)`
    (dirty buffer requires `.discard`, kept in `recoverableBuffer`), or keep
    editing; `resolveConflictPanel()` / `saveTexInteractive()` for menus.
    External deletion: `checkDiskStatus()` reports `.deleted` as a notice
    (nothing to overwrite); Save recreates through an expected-new-file save,
    which still refuses (`alreadyExists`) if a file appeared meanwhile.
  - Fallback without a helper binary: direct Foundation I/O with best-effort
    unlocked hash check; `files.status`/`conflict.viaHelper == false` say so.
    A helper that exists but fails is never bypassed.
  - Lost replies: bounded main-thread wait (needed because `saveTex()` is
    synchronous for quit/open flows); hang → false after `helperTimeout`,
    process replaced on the next request; exit mid-request → immediate
    honest failure; garbage → protocol failure; in every case the buffer
    stays dirty. A reply after the wait is reconciled on the main actor: a
    receipt for exactly the sent text becomes the baseline (later edits stay
    dirty), a late conflict is surfaced. A failed open no longer consumes a
    `.discard` decision.
  - Multi-file: open a (root A) → save → open b (root B, helper rebound) →
    detach → external change on a → open a with `.saveFirst` (b saved via a
    fresh helper on B, a read via a fresh helper on A) → conflict on a does
    not leak to b → restore discarded buffer re-baselines on disk content.
- Incomplete behavior / blockers / needs from others:
  - UI wiring lives in parent-retained files (exact diffs in the final report
    and below): Save menu → `saveTexInteractive()`, a "Resolve On-Disk
    Conflict…" item, `checkDiskStatus()` on `didBecomeActive`, quit flow.
  - `BridgeSession.commitPendingTransaction` still exports the `.tex` with
    `String.write(atomically:)` (parent-retained); suggested hook below.
  - Helper wire protocol does not yet expose discover/poll/recovery.
  - No native screenshot evidence: the conflict UI is not wired yet; the
    acceptance is XCTest with real files and the real helper.
  - Evidence of the 2 s-poll "external change while idle" path is via
    `checkDiskStatus()`, not a timer (parent decides where to call it).
- Interface changes / consumer actions: new `files` API on `ShellModel`
  (`files.conflict/status/backend/policy/helperTimeout`, `checkDiskStatus()`,
  `overwriteOnDisk()`, `reloadFromDisk(dirty:)`, `resolveConflictPanel()`,
  `saveTexInteractive()`, `baselineSha256`). `saveTex()` semantics: returns
  false on conflict instead of overwriting. `savedText == nil` now also means
  "no file on disk" after `restoreDiscardedBuffer` of a vanished file.
- Reviewed peer revisions / resulting adaptations: origin/main b1f158c
  (merged; no overlap); `origin/agent/mac-claude-a/mac-shell` 6b43a3a base
  (`DocumentFiles.swift` replaced in place, `ShellModel*.swift` untouched);
  `crates/edit-ledger` helper + `EditLedgerClient` pattern reused for the
  client; `crates/project-files` README proposal step 2 implemented.
- Validation commands / results / artifact paths:
  - `cargo test --manifest-path crates/project-files/Cargo.toml`: 38 pass
    (3 new in `tests/helper_bin.rs`); `cargo clippy --all-targets -- -D
    warnings` clean; `cargo fmt --check` clean.
  - `cd apps/mac && FLASHTEX_COMPILER=… FLASHTEX_PDF=… FLASHTEX_BRIDGE=…
    FLASHTEX_EDIT_LEDGER=… FLASHTEX_PROJECT_FILES=<repo>/crates/project-files/
    target/release/flashtex-project-files swift test`: see JSON evidence
    (193 tests; DocumentFilesTests 8 pass, real helper 3, fake 4, direct 1).
- Exact deadline UTC / remaining time / integration reserve: no deadline
  (continuous improvement authorization); reserve not applicable.
- ETA remaining: 0 for this lane; parent wiring diffs are minutes.
- Resource pool / allocation ID / maximum: parent's Claude Max 20x quota /
  claude-mac20x-document-files / unknown (shared account).
- Confirmed spend / estimated usage / in-flight reservation / remaining:
  unknown (per-call usage not exposed); no purchases.
- Billing evidence / freshness / unknowns: none available to this subagent.
- Child tasks and their deducted allocations: none.
- Dirty files / unpushed work / running jobs: none after the final commit.
- Decisions / failed approaches / linked findings: deletion demoted from
  blocking conflict to notice+recreate because a parent-retained bridge test
  (`BridgeRecoveryTests.testExportFailureWithholdsReceiptWhileTheDurableCommitStands`)
  saves to a URL that never existed with `savedText` set, and because nothing
  can be overwritten in that case; the expected-new-file retry keeps the
  no-silent-overwrite guarantee. Synchronous bounded wait chosen over an async
  save because `saveTex()` answers quit/open decisions.
- Controller route assessment (coordinator question, mac-shell b973b89 +
  crates/preview-controller STDIO.md): the controller's `export` writes the
  *ledger's* durable source (needs `expected_revision`/`expected_sha256` of
  the durable document plus `expected_disk_sha256`), `file_status` rereads
  disk, `reload` imports an approved disk snapshot into the ledger, and
  `open_document`/`detach_document` change session membership. That is the
  right save/status/reload path *while a controller session is attached and
  ready*: ledger text and `.tex` then never diverge and reload lands in the
  durable undo history. It cannot replace the project-files client: the
  fixture route and the direct-worker route (the parent's measured 2 ms
  typing path) have no controller, and `PreviewControllerClient` delivers on
  the main queue, so a controller-routed save must be asynchronous whereas
  `saveTex()` is synchronous for the quit/open flows. Recommendation: keep
  `DocumentFilesState` as the single file-layer facade and add a controller
  backend behind it (exact diff in the final report); not applied here
  because `PreviewControllerClient.swift` / `ShellModel+Controller.swift` are
  parent-owned and not on this branch's base.
- Context checkpoint (docs/context-checkpoints.md): this subagent cannot read
  a usage percentage from its runtime; the harness token counter shows the
  session far below the 60% threshold, so no compaction was needed. This file
  is the durable checkpoint: branch/HEAD in the JSON record, worktree
  `.claude/worktrees/agent-a68504b2d38f36ab0`, no dirty files after the
  final commit, no running commands, no pending messages; no purchases;
  transferred crates untouched.
- Exact next action or command: parent applies the diffs from the final report
  and merges `agent/mac-document-files/rooted-helper` into mac-shell.
- Resume reading list: this file, `crates/project-files/README.md` ("JSON
  Lines helper"), `apps/mac/Sources/FlashTeXMac/DocumentFiles.swift` header
  comments, `DocumentFilesTests.swift`.
