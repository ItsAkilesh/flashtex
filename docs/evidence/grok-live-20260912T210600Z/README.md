# Grok live check 20260912T210600Z — fast-model explanation attempt

Lane mac-grok-polish (branch agent/mac-grok-polish/fast-default, base 8d1ab8e2).
Goal: make `grok-4.20-0309-non-reasoning` the explanation default (grok-4.6 measured 55–70 s).

- key source: Keychain `tech.jay3332.flashtex.xai` (value never recorded, never in argv/JSON)
- helper: this worktree's `crates/assistant-context` built `--features grok`
- probe: `GET https://api.x.ai/v1/models` → **HTTP 200, key accepted, 0.27 s** (`probe.json`;
  the listing also names `grok-4.20-0309-reasoning`, untested here)
- explanation, model `grok-4.20-0309-non-reasoning`, provider bound 30 s, through the app's real
  flow (`ProposalPreview.explain` → helper `prepare` → `--provider-session` admit/poll):
  - attempt 1: **4.06 s** end to end, outcome `failed`:
    `helper refused poll: removed source differs` (`explanation-attempt1.json`)
  - attempt 2: **4.94 s**, same refusal (`explanation.json`)

## Finding

The fast model answers in ~4–5 s (the latency goal holds), but its proposed
edit does not pass the helper's source-bound gate: `removed_text` / byte
offsets do not match the shadow source (`crates/assistant-context/src/lib.rs`,
"removed source differs"). That gate is the safety boundary and stays; the
helper collapses the whole proposal, so no explanation text reaches the sheet.
`grok-4.6` passed the same flow live (grok-live-20260912T191209Z, 69.7 s).

Decision in this lane: `grok-4.6` remains the explanation default; the fast
model stays the capture default and is selectable in Preferences (labelled
"its edits may fail the helper's source check"). Follow-up for the
assistant-context owner: let a non-reasoning model's edits be located (resolve
`removed_text` against the source instead of trusting model byte offsets, or
return the explanation without the refused edits), then flip
`GrokCredential.defaultModel` to `GrokCredential.fastModel`.

No prompt or reply bodies were recorded. The HTTP class of the explanation
call itself is not surfaced by the helper's provider session (helper request
R4 in apps/mac/docs/grok-live.md); the probe above is the HTTP-class evidence.
