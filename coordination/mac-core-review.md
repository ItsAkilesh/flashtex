# mac-core-review — adversarial review of the Mac shell edit → durable → preview pipeline

Lane: `mac-core-review` (Claude Code subagent, Fable; parent `mac-claude-a`, mac-m1max-a).
Scope: read-only review of the parent-retained core plus minimal, test-covered fixes for
reproduced defects. Never edits `crates/*`; helper defects are reported with reproductions.

## Durable checkpoint

- Branch: `agent/mac-core-review/pipeline` (from `origin/agent/mac-claude-a/mac-shell` @ 6fb77efd)
- Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a07346630f0c4534d`
- HEAD: see `git log -1`; dirty files: this file, `coordination/agents/mac-core-review.json`,
  new `apps/mac/Tests/FlashTeXMacTests/ControllerPipelineReviewTests.swift` (in progress)
- Helpers built (release): compiler, preview-controller, pdf, bridge, edit-ledger,
  project-files, assistant-context under `crates/<c>/target/release/`.
- Next commands: `swift build --build-tests` (running), then
  `swift test --filter ControllerPipelineReviewTests` with FLASHTEX_* exports.
- Load at start: 1-min 11.4 (other lanes running).

## Findings so far

1. **`failed`/`cancelled` helper updates never release the in-flight edit** (default
   hold-until-preview policy): `handleController(.update)` only handles `stale`/`discarded`,
   and its guard compares the helper's global compile *generation* (`compile_revision`)
   with the document's durable revision; `discarded` frames carry no `compile_revision`
   at all (crates/preview-controller/src/main.rs:383-394). After a compiler failure the
   in-flight edit stays set forever: every later keystroke is queued and never sent
   (typing stall until app restart). Reproduction in progress with the real helper and a
   compiler wrapper that dies on its second request.
2. (reported only, so far) `controllerSave`/`controllerFileStatus` continuations are dropped
   without resuming when `detachController()` resets `controllerState` while an
   `export`/`file_status` is awaiting (`.exited` fails them first; explicit detach does not).
3. (reported only, so far) `completionFetcher` is not reset on helper exit/relaunch; request ids
   restart at `pc-1` on the new client, so a stale outstanding completion id can swallow a
   `document`/`edit` reply of the relaunched helper (`.refused` → `return`).
