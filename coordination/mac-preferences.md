# mac-preferences handoff — persisted editor display preferences

- Updated UTC: see `coordination/agents/mac-preferences.json` `updated_utc`
- Agent / parent / machine alias: `mac-preferences` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: Commander lane (issue #2, 10:41Z)
  "persisted editor display preferences with validation and sensible
  defaults". Owned paths: `apps/mac/Sources/FlashTeXMac/EditorPreferences.swift`
  (model + settings view), `apps/mac/Tests/FlashTeXMacTests/EditorPreferencesTests.swift`,
  this handoff and `coordination/agents/mac-preferences.json`. Not touched:
  `SourceEditorView.swift` (active editor lane), `ShellModel*`, `ContentView`,
  `FlashTeXMacApp`, `Completion.swift` — the exact diffs the owners need are
  below and in the final report.
- Branch / code revision / main integrated through:
  `agent/mac-preferences/editor` from `origin/agent/mac-claude-a/mac-shell`
  `f4c8aea` / see the agents JSON `code_revision` / main as merged into
  mac-shell at that tip.
- State: ready for integration (model, settings view and tests done; the
  parent/editor wiring is a requested diff, not applied here).

## Ready behavior and evidence

- `EditorPreferences` (`@Observable @MainActor` singleton `.shared`, backed by
  `UserDefaults.standard`; `init(defaults:)` for a test suite). Typed, clamped
  properties with write-through persistence:
  - `fontFamily: String?` — nil is the system monospaced face. A set value is
    accepted only if the family is installed and its regular member
    `isFixedPitch` (`NSFontManager`); otherwise it becomes nil.
    `installedMonospacedFamilies()` lists the picker candidates.
  - `fontSize: Double` clamped to `8...36` (non-finite → default 13).
  - `lineWrapping: Bool` (default on).
  - `tabWidth: Int` clamped to `2...8` (default 4); `indentStyle`
    `.spaces | .tabs` (default spaces); `indentString` is what a Tab inserts.
  - `appearance: .system | .light | .dark` (default system) with
    `nsAppearance`, `colorScheme` and `prefersDarkPreview(systemIsDark:)`;
    `darkPreviewDefault` is the preview dark toggle's initial value.
  - `autoCloseBraces: Bool` (default on; consumed by the editor lane),
    `completionPopup: Bool` (default on; consumed by `CompletingTextView`).
  - `generation` increments on every actual change; `snapshot` reads all.
- Persistence: keys `FlashTeX.EditorPreferences.v1.<name>` plus
  `FlashTeX.EditorPreferences.schemaVersion`. `migrate(_:)` stamps an absent
  version (unversioned stray keys are ignored) and never downgrades a newer
  stamp. `load()` repairs absent/invalid values (wrong type, out of range,
  unknown enum case, non-monospaced family) to valid ones and writes them back;
  `lastLoadRepairs` records which keys.
- `apply(to: NSTextView)`: font (whole storage + typing attributes), tab
  interval (`defaultParagraphStyle`, typing attributes, existing text;
  attribute-only edit — no text-change notification, no undo step, temporary
  attributes/marks untouched), line wrapping (container tracking/size,
  horizontal resizability, scroller), appearance (on the enclosing scroll
  view). `observeApplying(to:)` re-applies after every change until the token
  is cancelled/released. The doc comment on the type has the property →
  NSTextView-update table.
- `EditorPreferencesView`: grouped `Form` for the macOS `Settings` scene —
  font family picker (system face + installed monospaced families), size
  slider + stepper with a live sample line, wrap toggle, tab width stepper,
  indent radio group, appearance segmented picker, auto-close and completion
  toggles, Restore Defaults. Standard focusable controls; every control has an
  accessibility label and the non-obvious ones a hint.
- Tests (`EditorPreferencesTests`, 17): defaults and stamped migration,
  documented ranges/keys, font size and tab width clamping (incl. NaN/inf and
  persisted clamped value), font family validation (Menlo accepted, Helvetica
  and an unknown family rejected), `resolveFont` fallback, generation
  counting, round trip through a temp suite via a second instance, invalid
  stored values repaired and written back, migration behaviour, reset,
  `apply(to:)` on a real `CompletingTextView` (font runs, tab interval in
  points = columns × space advance, wrapping on/off, appearance on the scroll
  view and effective appearance), no text change/undo/temporary-attribute
  side effects, apply without a scroll view, observation re-applies and stops
  after cancel, appearance mapping, hosted settings view.

## Requested diffs in files owned by others (not applied)

See the final report (identical text). Summary:
1. `FlashTeXMacApp.swift`: add `Settings { EditorPreferencesView() }` scene.
2. `ShellModel.swift`: `var darkPreview = EditorPreferences.shared.darkPreviewDefault`.
3. `SourceEditorView.swift`: replace the hard-coded font with
   `context.coordinator.preferencesToken = EditorPreferences.shared.observeApplying(to: tv)`
   in `makeNSView`; Tab key / auto-close read `indentString` / `autoCloseBraces`.
4. `Completion.swift` `requestCompletion()`: `guard EditorPreferences.shared.completionPopup else { return }`.

## Limitations

- The `Settings` scene, the editor wiring and the preview default are not
  active until the owners apply the diffs; nothing in the app reads
  `EditorPreferences` yet on this branch.
- Font family validation uses `NSFontManager`; a family installed after
  launch is accepted on the next set (the picker list is built on appear).
- No screenshots: the settings window cannot be shown without the app diff.
