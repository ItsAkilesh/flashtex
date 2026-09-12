# mac-history handoff

- Updated UTC: 2026-09-12T11:13Z
- Agent / parent / machine alias: mac-history (Claude Code subagent) / mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: Commander lane (issue #2, 10:41Z): durable
  undo history UI on the helper's ledger. Gate: panel shows undo/redo stacks with
  meaningful labels and retention usage; explicit undo/redo guarded by the exact
  durable revision/sha; conflict → refuse and re-read; identical-id retry after
  an uncertain reply; documented behaviour across helper restart; VoiceOver labels;
  real-helper tests; full suite green. Owned: NEW
  `apps/mac/Sources/FlashTeXMac/EditHistoryPanel.swift`,
  `apps/mac/Tests/FlashTeXMacTests/EditHistoryTests.swift`, this handoff and
  `coordination/agents/mac-history.json`.
- Branch / code revision / main integrated through: `agent/mac-history/panel`
  from `origin/agent/mac-claude-a/mac-shell` f4c8aea; tip in the agent JSON.
  origin/main 796b982c merged in 05b247be (clean; no apps/mac changes on main; the merge message names 6e515bef, one of its commits)
  to pick up the helper's `history_status` document identity + limits (64829a0d).
- State: ready for integration (parent must apply one hook; see below).

## Ready behavior and evidence

`EditHistoryPanel.swift` (view + bounded client, no ledger/bridge code):

- `history_status {path}` → `EditHistory.Status`: undo/redo label stacks
  (oldest→newest; next step is the LAST element), `permanent_command_ids`,
  `history_bytes`; usage against the ledger limits 256 steps / 32 MiB / 4096 ids
  (`history.rs` MAX_HISTORY_*), warning from 80%, "full" when a recorded edit
  would be refused. The helper protocol exposes NO retention operation, so the
  panel can only warn early — stated in the warning text.
- Rows: consecutive `"Source edit"` steps collapse into "Typing run × N"
  (each step still one undo press; `steps` and `distance` are in the VoiceOver
  label); `"Capture <id>"` → "Capture insertion"; group labels verbatim;
  session annotations (`client.noteNextLabel("Reload from disk")` before an
  explicit reload) refine a "Source edit" the panel witnessed. The ledger
  itself labels typing and explicit reloads identically ("Source edit",
  `replace_document`), so a reload is only named when the shell announces it.
- `undo`/`redo`: fresh `mac-history-<uuid>` command id per action, expectation
  = the shell's current durable (revision, sha) for the active path; refused
  locally while a command is pending, the buffer is not yet durable (edit in
  flight / buffer ≠ durable text), or the stack is empty. Reply handling:
  `document_conflict` → clear, `document {path}` re-read (shell adoption),
  refresh; `command_id_conflict`/`history_empty`/`invalid_id`/other → clear +
  note; `history_full`/`history_ids_full` → capacity warning; helper exited /
  send failure / no reply within `replyTimeout` (10 s) → `uncertain`, Retry
  resends the SAME id (ledger answers `replayed_command:true`, never moves
  twice), Discard drops it and re-reads.
- Adoption: `ShellModel.controllerAdoptHistoryResult(doc, requestID:, payload:,
  issuedAtEditorRevision:)` (hook below): durable state recorded first, buffer
  takes the durable text when the editor revision has not moved since the
  command was issued, then the result is treated as the in-flight edit for that
  request id so the helper's follow-up preview binds to the new editor revision.
  If the user typed meanwhile, the buffer is left alone and the shell's normal
  path resubmits it on top of the durable result.
- Helper restart / `controllerAttached` flips: false → status unavailable,
  pending command → `uncertain`; true → refresh, uncertain command kept for an
  explicit Retry. Because the parent's attach policy resubmits a differing
  buffer as a new edit, a retry after relaunch returns either the replayed
  receipt (undo was applied before the exit; the current document — possibly
  the resubmitted buffer, one revision newer — is adopted and the note says so)
  or applies the move now. Nothing is retried automatically. Both branches are
  tested (`testHelperDetach…`, `testRetryAfterRelaunch…`).
- VoiceOver: every row (`accessibilityElement(children: .ignore)` + label
  "Typing run, 2 steps, 2 steps below the next undo. …"), Undo/Redo/Retry/
  Discard/Refresh, retention progress, capacity warning and status note carry
  labels and `history.*` identifiers.

Validation (all at tip; FLASHTEX_PREVIEW_CONTROLLER + FLASHTEX_COMPILER set to
the release binaries built in the main checkout at 06:27/05:43 local):

- `swift test --filter EditHistoryTests`: 12/12, repeated 8× (no flakes):
  wire decoding; failure codes; row collapsing + accessibility labels; retention
  maths; no-helper refusal; real helper: edit→undo→redo round trip with exact
  text checks (durable r2→r3→r4, buffer bytes equal, follow-up preview bound and
  not stale, `permanent_command_ids` 1→2); stale-revision refusal (r1 hash, and
  right hash/wrong revision) with re-read and no permanent id; identical retry
  after a lost reply (`replayed_command:true`, one move r3, one permanent id) plus
  command-id conflict for a different op under the same id; detach → relaunch →
  retry converges; applied-before-lost-reply → relaunch resubmits buffer r4 →
  retry replays and adopts r4; capacity: 256 raw edits fill retention, the
  shell's next edit is refused `history_full` and surfaced through
  `shellCapacityRefusal`, undo still works when full (entries only move).
- `history_status` on main ≥ 64829a0d also returns `document` identity (no
  text) and `limits`: usage is measured against the reported limits (built-in
  constants for an older helper) and a move is refused locally with a re-read
  when the stacks on screen describe a different revision than the shell's
  durable snapshot. Tested against BOTH helper generations: the worktree-built
  helper from the merged tree (identity guard + limits assertions run) and the
  06:27 binary in the main checkout (fallback paths; "predates 64829a0d" lines).
- Full `swift test` with all real binaries (helper, compiler, pdf, bridge,
  edit-ledger): 476 tests, 11 skipped (other lanes' helpers), 0 failures — at
  6b595ed (old helper) and at bfcc84b6 (merged tree, new helper).
- Window capture: `docs/evidence/mac-history/panel-seeded-ledger.jpg` (+ README,
  seed script): the Durable History window on a seeded ledger — undo 1, redo
  run ×2, 574 B, 2 permanent ids, durable r6 — launched with FLASHTEX_NO_ACTIVATE=1.

## Incomplete behavior / blockers / needs from others

- The panel is reachable only through the locally applied app diff (parent-retained
  `FlashTeXMacApp.swift`; diff below) until the parent integrates it.
- `apply_group` (grouped edits with a label) is not driven by the panel; the
  client's `issue(_:)` accepts explicit commands so a future group sender can
  reuse the id/retry discipline.
- No retention UI: the helper offers no retention operation (STDIO.md: "native
  retention UI is still required before history is full"). Needs a helper-side
  op; not this lane.
- The ledger cannot distinguish typing from explicit reloads (both "Source
  edit"); reload rows depend on the shell calling `noteNextLabel` (diff below).
- The text view's ⌘Z remains the in-memory editor undo; this panel is the
  durable one. Wiring ⌘Z to the ledger is a product decision for the parent.

## Interface changes / consumer actions (exact diffs for parent-retained files)

1. `apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift` — applied locally in
   commit 53e56a0 ("[parent diff, applied locally]"); cherry-pick or re-apply:
   add `func controllerAdoptHistoryResult(_ doc: [String: Any], requestID: String,
   payload: [String: Any], issuedAtEditorRevision: Int?)` before
   `controllerReleaseInFlight()` (see the commit; 24 lines, no other change).
2. `apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift` — applied locally in
   commits 2e60041e + bfcc84b6 ("[parent diff, applied locally]"):
   - Edit menu (`CommandGroup(after: .pasteboard)`), after "Nearby Companion…":
     `Button("Durable History…") { openWindow(id: EditHistoryPanel.windowID) }`
     (no key equivalent: CommandTableTests requires every shortcut item to be
     in mac-accessibility's AccessibilityCommand table; ⌥⌘Z suggested once
     that lane adds the entry).
   - Scenes: `Window("Durable History", id: EditHistoryPanel.windowID) {
     EditHistoryPanel().environment(model) }`
   - `FLASHTEX_OPEN_WINDOW` allow-list: add `EditHistoryPanel.windowID`.
3. Optional, `DocumentFiles.swift` reload path: before sending `reload`, call
   the panel client's `noteNextLabel("Reload from disk")` — requires the parent
   to own a shared `EditHistoryClient` (e.g. `model.history`) instead of the
   panel's `@State`; not applied here.

## Reviewed peer revisions / resulting adaptations

- origin/main 3ac69c3 (crates/preview-controller 2f2605d, STDIO.md history
  section; edit-ledger history.rs): wire shapes verified against the built
  helper by a raw JSON-Lines smoke run before writing Swift (labels, error
  codes `document_conflict` / `command_id_conflict` / `history_empty` /
  `history_full` / `invalid_id`, `replayed_command`, revision advance).
- origin/agent/mac-claude-a/mac-shell f4c8aea: reused `ControllerState.awaiting`
  per-request waiters and `applyDurableDocument` instead of adding any routing.

## Resources

- Pool: shared Claude Max 20x quota with parent mac-claude-a; no purchases, no
  paid network calls. Child allocations: none.
- Dirty files / unpushed work / running jobs: none after the final push (see
  agent JSON `code_revision`).

## Exact next action

Parent: cherry-pick 53e56a0 (hook), 2e60041e and bfcc84b6 (app window/menu) onto
`agent/mac-claude-a/mac-shell`, or re-apply the diffs above, then merge
`agent/mac-history/panel`. Follow-ups for this lane if kept staffed: group
edits (`apply_group`) through `issue(_:)`; a shared `model.history` client so
reloads are annotated; retention UI once the helper exposes it.

## Resume reading list

`crates/preview-controller/STDIO.md` (history section), `crates/edit-ledger/src/history.rs`,
`apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift`, this file.
