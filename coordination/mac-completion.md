# mac-completion handoff — revision-bound completion metadata

- Updated UTC: 2026-09-12T08:50Z
- Agent / parent / machine alias: `mac-completion` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: lane "Consume bounded revision-bound
  Rust completion metadata" with follow-ups "Cancellation and stale
  caret/context refusal" and "Keyboard responsiveness and acceptance tests"
  (dispatched by the parent from issue #2). Owned paths:
  `apps/mac/Sources/FlashTeXMac/Completion.swift`,
  `apps/mac/Tests/FlashTeXMacTests/CompletionTests.swift`, this handoff and
  `coordination/agents/mac-completion.json`.
- Branch / code revision / main integrated through:
  `agent/mac-completion/revision-bound` (from
  `origin/agent/mac-claude-a/mac-shell` at `6b43a3a`) / see
  `coordination/agents/mac-completion.json` `code_revision` / branch base
  contains main through `2fd3026`; main `ad9fec2` reviewed (coordination-only
  commits since the base).
- State: in progress (core lane and both follow-ups implemented and tested;
  parent-side wiring reported, not applied)
- Ready behavior and evidence:
  - What main provides (read on `origin/main` `ad9fec2`): runtime-v1
    `compile_result` has no completion vocabulary — only `revision` and
    diagnostics that name commands (`\X is not supported …`), references
    (`undefined reference 'key'`) and environments (`environment 'X' is not
    implemented; …`). The only documented completion metadata channel is the
    preview-controller helper's `complete` operation
    (`crates/preview-controller/STDIO.md`: `{source_versions, category
    label|citation|command, prefix, limit 1–100}` →
    `{source_versions, completions:[{name, definitions[≤100],
    occurrences[≤100], locations_truncated}]}`, backed by
    `crates/project-index`). The Mac app does not run that helper today.
  - `Completion.Metadata` (Completion.swift): one value per producer, carrying
    the editor `revision` it describes. `Metadata.from(compileResult)` extracts
    the diagnostics above; `Metadata.decodeProjectIndexReply(data, category:,
    editorRevision:, expectedSourceVersions:)` decodes the helper reply and
    refuses it when `source_versions` differ from the queried snapshot
    (`DecodeError.staleSourceVersions`). Bounds: 100 items per category,
    4096-byte names (project-index key bound), 4 MiB reply, 256 diagnostic
    entries, 512-character messages; `truncated` records any cap hit.
    `bound(to: revision)` is the only way to use metadata for a caret and it
    returns nil for any other revision or an unknown (nil) editor revision.
    `merged(with:)` combines producers only at the same revision.
  - Suggestions now take bound metadata: project-index labels join `\ref{`
    (with "defined in main.tex · N uses · revision R" details), a new
    `\cite{`/`\citep{`/… context lists `\bibitem` keys and project-index
    citation keys (unresolved ones say so), declared project commands join
    the `\` list, and compile diagnostics annotate unsupported commands,
    undefined references and unimplemented environments — all labelled with
    the revision they came from. An empty prefix right after `\ref{`,
    `\cite{`, `\begin{`, `\end{` lists everything.
  - `CompletionScheduler`: candidates are computed on a serial user-initiated
    queue and delivered with a main run-loop block (same head-of-loop pattern
    as `WorkerClient.deliver`) only when the job is still the current,
    uncancelled generation; `cancel()` on every caret move/text change; a
    newer request supersedes the pending one; the scan polls the cancellation
    flag between phases. `Statistics` (scheduled/delivered/refusedStale/
    cancelled) make the refusals observable in tests.
  - `CompletingTextView` owns a completion session and a non-activating
    `CompletionPopup` (NSPanel child window, `canBecomeKey == false`, so it
    never takes focus). ⌃Space or Esc open it (Esc handled explicitly:
    AppKit's own `cancelOperation:` → `complete:` binding did not fire in a
    non-key hosted window); ↓/↑ choose (wrapping); Return/Enter/Tab insert the
    chosen item over the token range as one undoable edit; Esc closes; typing
    and Delete narrow/widen the list and keep the chosen item when it
    survives; ←/→/Home/End/Page keys, mouse/programmatic caret moves,
    programmatic text replacement (`NSTextStorage.didProcessEditing`) and
    losing first responder close it and cancel in-flight work. Accepting is
    refused when the scheduler generation moved since the items were
    computed. `editorRevision` (set by the owner) binds metadata;
    `accept(projectIndex:)` refuses metadata older than the held one.
  - Measurements (this Mac, debug build, `swift test`): synchronous scan on
    `Samples/demo.tex` (5,909 B) on the main thread, average of 50: command
    0.024 ms, word 0.031 ms, environment 0.013 ms, reference 0.016 ms (bound
    < 2 ms asserted). Keystroke through the open list on demo.tex (AppKit
    insertion + text copy + enqueue): max 1.96 ms over five keys, off-main
    scan 0.88–0.98 ms. 1 MB buffer sync scan (pre-existing test): words
    8.5 ms, commands 5.4 ms — the popup path never runs that on main.
- Incomplete behavior / blockers / needs from others:
  - The app does not pass the editor revision to the text view yet, so in
    the running app `editorRevision` is nil and no compile-result metadata
    binds (candidates come from the document text alone; nothing stale is
    ever shown). Parent-retained diff below enables binding.
  - No live project-index channel: the Mac shell does not spawn
    `flashtex-preview-controller`; the decoder is exercised on the documented
    wire shape only. Wiring the helper (spawn, `snapshot`, `complete` per
    category, `accept(projectIndex:)` with the editor revision current at
    that snapshot) is ShellModel work for the parent/Commander.
  - Popup rows are keyboard-only (no click-to-accept yet); VoiceOver label
    "Completions" is set on the table but not verified with VoiceOver.
  - No app screenshot evidence: opening the list needs keystrokes into the
    app, which would require stealing focus or Accessibility (not granted).
- Interface changes / consumer actions:
  - `Completion.suggestions(in:caretUTF16:metadata:supported:cancelled:)` is
    the primary API; the `result:` overload remains and binds the result
    as-is (caller asserts it matches the text).
  - `Completion.Kind.citation`, `Token.Context.citation`, `Completion.bibitems`.
  - Exact diff needed in parent-retained files (not applied):
    `SourceEditorView.swift`: add `var editorRevision: Int?` to
    `SourceEditorView` and in `updateNSView` set
    `(tv as? CompletingTextView)?.editorRevision = editorRevision` next to
    the existing `compileResult = result` line. `ContentView.swift`: pass
    `editorRevision: model.editorRevision,` in the `SourceEditorView(...)`
    call after `result: model.result,`. (`ShellModel.editorRevision` already
    increments on every text change, and `result.revision` is the compiled
    revision, so binding is exact.)
- Reviewed peer revisions / resulting adaptations: `origin/main` `ad9fec2`
  (coordination dispatch commits only since the branch base); STDIO contract
  and project-index README read on main and reflected in the decoder shape,
  bounds and the "lexical, not TeX semantics" wording of details.
- Validation commands / results / artifact paths:
  `cd apps/mac && swift test --filter CompletionTests` → 19/19;
  `FLASHTEX_COMPILER=… FLASHTEX_PDF=… FLASHTEX_BRIDGE=… FLASHTEX_EDIT_LEDGER=…
  swift test` (release worker binaries from the main checkout) → 195 tests,
  0 failures, 31.5 s. Timing lines are printed by
  `testCandidateComputationOnDemoTexStaysUnderTwoMilliseconds` and
  `testKeystrokeThroughOpenListOnDemoTexDoesNotScanOnMain`.
- Exact deadline UTC / remaining time / integration reserve: per
  `coordination/PROJECT.md`; no stop condition other than user stop.
- ETA remaining, optimistic / likely / pessimistic / confidence: 20 / 40 /
  90 minutes for click-to-accept and evidence polish / medium.
- Resource pool / allocation ID / maximum: shared Claude Max 20x quota on
  mac-m1max-a via parent `mac-claude-a` / `claude-mac20x-completion` /
  unknown (parent-owned).
- Confirmed spend / estimated usage / in-flight reservation / remaining:
  unknown / one subagent session / none / unknown.
- Billing evidence / freshness / unknowns: none beyond the parent's report.
- Child tasks and their deducted allocations: none.
- Dirty files / unpushed work / running jobs: none after each checkpoint push.
- Decisions / failed approaches / linked findings:
  - AppKit's built-in completion popup was probed in a hosted non-key window:
    ↓ moved the caret, Return inserted a newline, Esc did not revert — the
    popup does not intercept `keyDown` delivered to the view. An owned session
    makes the keyboard behaviour deterministic and testable, and lets the list
    show kind/detail text.
  - Binding is exact-revision (`==`), not "not older": metadata newer than
    the text is not the text's either.
- Exact next action or command: parent applies the two-line diff above;
  optional click-to-accept; Commander integration of the branch.
- Resume reading list: this file, `apps/mac/Sources/FlashTeXMac/Completion.swift`
  header comments, `crates/preview-controller/STDIO.md` (complete/navigate),
  `crates/project-index/README.md`.
