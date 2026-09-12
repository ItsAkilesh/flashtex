# Manual acceptance checklist (things automation cannot verify here)

These checks need a person at the Mac with a GUI session. The automated suite
cannot see the window: `screencapture -x` reports `could not create image from
display` and `osascript` UI scripting reports `osascript is not allowed
assistive access. (-1728)` from the agent's session (see `README.md`). Record
each observation with a timestamp in the run report or a follow-up handoff.

Setup (once):

```sh
cd <repo>                       # a checkout of origin/agent/mac-claude-a/mac-shell
cd crates/compiler && cargo build --release && cd ../..   # from origin/agent/claude/compiler-foundation
cd apps/mac && swift build
FLASHTEX_REPO=$(git rev-parse --show-toplevel) .build/debug/FlashTeXMac
```

`crates/compiler` and `apps/mac` live on different branches until both are on
main; either merge locally into a throwaway branch or set
`FLASHTEX_COMPILER=/path/to/flashtex-compiler` before launching the app.

## 1. Fixture preview and click-to-source (visual)

1. Launch the app with `FLASHTEX_REPO` set. Expected: the preview banner shows
   the `FIXTURE` badge, `id fixture-compile-1`, `revision 1`, `status ok`,
   `pdf: none`; the page shows `Hello FlashTeX.` near the top-left.
2. Hover the text item. Expected: it highlights.
3. Click the text item. Expected: the editor selects the range mapped from the
   item's `source` (UTF-8 bytes 0..14 in the fixture). Known fixture defect: the
   fixture's `end_byte` is 14 while `"Hello FlashTeX."` is 15 bytes, so the
   selection will stop before the final `.`. This is a `protocol/fixtures`
   issue (owner: Commander), not an app bug; record which behavior you saw.
4. Footer shows UTF-8 byte and UTF-16 unit counts for the buffer.

## 2. Unicode navigation (visual)

1. `File > Attach Built Compiler` (⌘⇧K). Expected: badge switches to `WORKER`.
2. Replace the editor text with `naïve café — 😀 end` and press `Compile` (⌘B).
   Expected: five items appear (`naïve`, `café`, `—`, `😀`, `end`) and the
   footer byte count is 25 with 19 UTF-16 units (the emoji is a surrogate pair).
3. Click `end`. Expected: exactly `end` is selected (no off-by-one from the
   3-byte `—`, the 4-byte `😀`, or the 2-byte `ï`/`é`). Click `😀`. Expected:
   exactly the emoji is selected.

## 3. Stale-preview safety (visual)

1. With the worker attached and a compiled result shown, turn the toolbar
   auto-compile toggle **off**, then type one more character in the editor.
   Expected: the banner states the buffer was edited since the shown revision;
   the preview does **not** change. Click an item whose text you edited.
   Expected: navigation is refused with a "recompile to navigate" note rather
   than selecting the wrong bytes. Click an item outside the edit. Expected: it
   still selects its exact text (the span was rebased through the unchanged
   prefix/suffix).
2. Turn auto-compile back **on** and type a burst of 10 characters quickly.
   Expected: fewer requests than keystrokes (debounced 250 ms and coalesced),
   the banner's revision ends equal to the editor revision, and a latency
   figure with a median is displayed. An older result never replaces a newer
   one.
3. `File > Detach Worker`. Expected: the banner reports the worker exit; the
   last preview stays visible with its revision.

## 4. Dark preview versus export

1. Toggle the dark preview in the toolbar. Expected: only the page and text
   colors invert on screen; the banner, revision and item positions are
   unchanged, and click-to-source keeps working while dark.
2. With dark preview **off**, `File > Export PDF…` (⌘⇧E) and save as
   `light.pdf`. Run
   `python3 tools/native-validation/check_pdf_export.py light.pdf --expect-pages <pages shown>`.
   Expected: `page background is white -- 1.0 sc`, one MediaBox per page equal
   to the page's `width_pt` x `height_pt`, exit 0. Open it in Preview: white
   page, black Times text at the reported positions.
3. With dark preview **on**, export again as `dark.pdf` and run the same
   check. **Expectation for acceptance (FT-010): the export stays white and the
   check exits 0.** What the code at mac-shell `fd2a26e` does instead:
   `ShellModel.exportPDF()` calls `PDFExport.render(result, dark: darkPreview)`,
   which paints the page `CGColor(gray: 0.16)` with white text when the toggle
   is on, so the check is expected to report
   `0.16 sc <- dark/non-white background exported` and exit 1. This is an
   app-owner decision to confirm or change (see the run report); record which
   behavior you observed. The check was validated against a CoreGraphics PDF
   produced the same way (1.0 sc vs 0.16 sc), not yet against a file exported
   by the app, because the save panel cannot be driven from the agent session.
4. The exported PDF is the compiler's reported layout, not a TeX-engine PDF
   (Times only, no images/lines/links). The Rust PDF crate (`mac-pdf`) is still
   pending; when it lands, repeat 2-3 against its output.

## 5. Capture proposal review (behavioral)

1. Place the caret in the editor, `Edit > Pin Insertion Point` (⌘⇧P).
2. `Edit > Open Capture Proposal…` (⌘⇧I), choose `apps/mac/Samples/capture-proposal.json`.
   Expected: a review sheet with editable LaTeX, ambiguities and required packages.
3. Approve. Expected: one insertion at the pinned anchor; ⌘Z removes it in one step.
4. Open the same proposal again and approve. Expected: no second insertion
   (duplicate `capture_id`).
5. Pin, then delete the surrounding text, then open a proposal. Expected: the
   app asks for reselection instead of inserting somewhere else.

## 6. Reporting

For each section note: date/time (UTC), machine, app SHA, compiler SHA,
observed behavior, and whether it matched the expectation. A screenshot taken
by a human (⌘⇧3/⌘⇧4, which run in the user's GUI session) is acceptable
evidence where the agent's `screencapture` is blocked.
