# mac-a11y-panels evidence — keyboard traversal of the Settings, Durable History and Find in Project panels

Lane `mac-a11y-panels` (Claude Code subagent of `mac-claude-a`, mac-m1max-a),
branch `agent/mac-a11y-panels/a11y-panels` on top of `agent/mac-claude-a/mac-shell`
`72b29e8f` (search lane integrated). 2026-09-12, 1-min load 8–20 during the runs
(the machine had 12 lanes compiling minutes earlier, load 60–140).

- `panel-accessibility-tests.log` — `swift test --filter PanelAccessibilityTests`
  (3/3 pass) with the printed reading-order walk of every AppKit-backed
  control per panel: Settings 8 (pop-up, slider, 2 steppers, 3 switches,
  segmented), Durable History 1 (the stacks list; all buttons disabled
  without a controller), Find in Project 5 (literal field first responder on
  open, scope pop-up, match-limit stepper, replacement field). `nextValidKeyView`
  is nil for all of them in the never-key host (documented limitation).
- `narrow-filter-41-tests.log` — `CommandTableTests|PanelAccessibilityTests|EditorPreferencesTests|EditHistoryTests`:
  41 executed, 7 env-gated skips, 0 failures (`CommandTableTests` 8 incl. the
  new `testPanelFocusOrderMatchesThePanelSources`).

No screenshots: the panels are hosted off-screen in a test process that is
never activated (FLASHTEX_NO_ACTIVATE=1, no UI scripting); Accessibility
permission is not granted, so the AX API answers kAXErrorAPIDisabled (-25208).
