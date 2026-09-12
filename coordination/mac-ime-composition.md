# mac-ime-composition handoff — CJK IME composition against the real preview controller

- Updated UTC: see `coordination/agents/mac-ime-composition.json` `updated_utc`
- Agent / parent / machine alias: `mac-ime-composition` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Gap 1 — CJK marked-text (IME) composition during background preview /
  helper restart, verified through the ACTUAL `flashtex-preview-controller` +
  `flashtex-compiler`. Owned paths: `apps/mac/Tests/FlashTeXMacTests/IMEHarness.swift`,
  `apps/mac/Tests/FlashTeXMacTests/IMECompositionTests.swift`, this handoff and
  `coordination/agents/mac-ime-composition.json`. Not touched: parent-retained
  files (`ShellModel*.swift`, `ContentView.swift`, `PreviewView.swift`,
  `FlashTeXMacApp.swift`, `SourceEditorView.swift`) and every Rust crate.
- Branch: `agent/mac-ime-composition/ime` from `origin/agent/mac-claude-a/mac-shell`
  `5bc3fc0f` (contains main `dda0b62`'s preview-controller).

## Coverage audit (mandatory first step, done 09:30–09:45Z)

Already covered BEFORE this lane (all with the fake python worker or no helper,
never the real preview controller):

- `apps/mac/Tests/FlashTeXMacTests/SourceEditorViewTests.swift`
  - `testCompositionReachesTheModelOnlyWhenCommitted` — Japanese steps
    か/かん/漢 never reach `model.activeText`/`editorRevision`, no compile per
    step (`TypingBench` recorder), caret reported at the composition start,
    commit `漢字` = one revision + one compile (fake worker), navigation
    selection deferred while marked text exists.
  - `testCompositionCancelDropsTheStepsAndClosesTheCompletionList` — IME
    cancel (`setMarkedText("")` + `unmarkText`) restores the committed text with
    no revision; `unmarkText` with marked text left commits once; dead-key
    (´ → é) under the completion list; `caretByte` after é.
  - `testCaretBytesAreExactAcrossComposedAndDecomposedCharacters` — UTF-8
    caret bytes across composed/decomposed characters (no IME, no helper).
- `apps/mac/Tests/FlashTeXMacTests/CompletionTests.swift` (~line 1057) —
  marked text (か) never triggers a snippet; a composition closes the list.
- `apps/mac/Tests/FlashTeXMacTests/EditorDiagnosticsTests.swift` (~line 180) —
  a mark under text typed right after marked text (mark geometry only).
- Durable-edit pipeline with the real helper, no IME involved:
  `PreviewControllerTests.testEditsBecomeDurableAndPreviewsBindToEditorRevisions`,
  `testControllerSaveExportsDurableSourceAndRefusesChangedDisk`;
  `EditHistoryTests.testEditUndoRedoRoundTripAdoptsExactDurableText`,
  `testHelperDetachMarksThePendingCommandUncertainAndRetryConvergesAfterReattach`
  (explicit detach/reattach, not a kill), `testRetryAfterRelaunchReplaysAnUndoAppliedBeforeTheLostReply`.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md` and
  `docs/evidence/*`: no IME/marked-text/CJK case (grep `IME|marked|composition|CJK|漢`
  only hits `typing-bench-…md` line 29, which states the bench has "no
  input-method composition").

NOT covered before this lane (implemented here):

- (a) marked text vs the REAL helper's `document` / `file_status` (durable
  revision, text and disk state unchanged until commit);
- (b) a real preview update (`update {kind: preview}`) landing while the view
  has marked text: marked range, caret and composition intact;
- (c) helper killed by pid (SIGKILL) mid-composition, reattached, commit becomes
  durable exactly once;
- (d) one `NSUndoManager` step removes the whole committed composition and
  `history_status` shows exactly one entry for it;
- multi-codepoint CJK (surrogate pair 𠮷, Hangul syllables) with UTF-8 byte
  checks (`caretByte`, `source_sha256` of the durable text).

Finding while auditing: the shell has NO automatic relaunch for the preview
controller (`ShellModel+Controller.swift` `.exited` → `controller = nil`,
status "helper exited"); only the direct worker (`ShellModel.scheduleWorkerRelaunch`)
and the bridge children (`BridgeSession`) auto-relaunch. So (c) is tested as
kill → observed exit → explicit `attachController` (what File > Attach does),
and the report names this as a limitation rather than claiming auto-relaunch.

## Ready behavior and evidence

- `apps/mac/Tests/FlashTeXMacTests/IMEHarness.swift` — hosts the real editor
  (`SourceEditorView` → `CompletingTextView`, `NSHostingView` in a never-key
  window) bound to a `ShellModel` attached to the real helper; `compose`/`commit`/
  `cancel` replay `NSTextInputClient`; `helperDocument`, `historyLabels`,
  `fileState` observe the helper through `controllerState.awaiting` without
  applying replies; `killHelper` SIGKILLs only a pid the harness launched.
- `apps/mac/Tests/FlashTeXMacTests/IMECompositionTests.swift` — 4 tests
  (a: never durable + d: one edit/one entry/one undo step + surrogate-pair bytes;
  b: preview mid-composition; c: kill-by-pid mid-composition + reattach, durable
  once; a': cancel). Japanese/Chinese/Korean incl. 𠮷 (U+20BB7).
- Evidence: `docs/evidence/mac-ime-composition-2026-09-12.md` + 3 run logs.
  Measured: 4/4 passed twice (runs 2, 3; 2.9 s each) under shared load 16–27,
  after run 1 corrected one assumption (AppKit leaves a no-op "Typing" undo
  action after an IME cancel). XCTSkip without env verified. `swift build
  --build-tests` clean for the new files.
- Not run: full `swift test` (parent's heavy-build window notice).
- No parent-retained file was changed: no diffs to apply.

## Durable checkpoint

- Branch `agent/mac-ime-composition/ime`, tests committed at `7062c55a`;
  base `5bc3fc0f`; main seen `ffe199d8`.
- Helpers used (main checkout release builds, sha256 in the evidence doc):
  `/Users/jay3332/Projects/flashtex/crates/preview-controller/target/release/flashtex-preview-controller`,
  `/Users/jay3332/Projects/flashtex/crates/compiler/target/release/flashtex-compiler`.
- Dirty files: none. State: ready for integration (test-only lane).
- Next command for the parent: merge the branch into mac-shell; at integration
  run `cd apps/mac && FLASHTEX_PREVIEW_CONTROLLER=… FLASHTEX_COMPILER=… swift test --filter IMECompositionTests` (load < 15).
- Staffing/billing: shared Claude Max quota with parent; no purchases.
