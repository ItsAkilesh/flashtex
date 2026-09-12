# Grok live check 20260912T191209Z

- commit: 4b0fc3b6bde766e710d749c7519d3cabf7af506b
- key source: Keychain (value never recorded)
- model: grok-4.6
- helper: /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2b5f9da704bfbf4b/crates/assistant-context/target/release/flashtex-assistant-context (built --features grok)
- bridge: /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2b5f9da704bfbf4b/crates/bridge/target/release/flashtex-bridge
- swift test status: 0 (0 = all three live cases passed)

## probe

```json
{
  "endpoint" : "https:\/\/api.x.ai\/v1\/models",
  "key_accepted" : true,
  "key_source" : "Keychain",
  "models_listed" : [
    "grok-4.20-0309-non-reasoning",
    "grok-4.20-0309-reasoning",
    "grok-4.20-multi-agent-0309",
    "grok-4.3",
    "grok-4.5",
    "grok-4.6",
    "grok-build-0.1",
    "grok-imagine-image",
    "grok-imagine-image-2.0",
    "grok-imagine-image-quality",
    "grok-imagine-video",
    "grok-imagine-video-1.5"
  ],
  "outcome" : "connection OK (HTTP 200): the key was accepted",
  "recorded_utc" : "2026-09-12T19:13:25Z",
  "seconds" : 0.16738796234130859
}
```

## explanation

```json
{
  "applied" : false,
  "context_id" : "47acb1dfa9218bf01e4c326f66f5d146c22cc90278a8d7cab011954eb4742d79",
  "edits" : 1,
  "explanation_bytes" : 570,
  "helper" : "\/Users\/jay3332\/Projects\/flashtex\/.claude\/worktrees\/agent-a2b5f9da704bfbf4b\/crates\/assistant-context\/target\/release\/flashtex-assistant-context",
  "key_source" : "Keychain",
  "launches" : [
    "helper:probe",
    "helper:prepare",
    "grok:provider",
    "helper:review"
  ],
  "model" : "grok-4.6",
  "note" : "xAI response id\/model\/usage are not surfaced by the helper's provider session (helper request R4 in apps\/mac\/docs\/grok-live.md)",
  "outcome" : "ready",
  "recorded_utc" : "2026-09-12T19:13:25Z",
  "review_id" : "abd1e3b59186124c4e33a282dce56d90283dcae59e67102ef832b1fe96a71de7",
  "seconds" : 69.710759043693542,
  "session_argv_has_key" : false,
  "session_env_keys" : [
    "FLASHTEX_GROK_API_KEY",
    "HOME",
    "LANG",
    "PATH",
    "SHELL",
    "TMPDIR",
    "USER"
  ],
  "session_id" : "mac-explain-1-27cbf45d"
}
```

## conversion

```json
{
  "ambiguities" : 4,
  "bridge" : "\/Users\/jay3332\/Projects\/flashtex\/.claude\/worktrees\/agent-a2b5f9da704bfbf4b\/crates\/bridge\/target\/release\/flashtex-bridge",
  "bridge_env_has_key" : true,
  "bridge_env_sensitive_names_other_than_key" : [

  ],
  "capture_id" : "grok-live-cb1962b0",
  "compile_gate" : {
    "compiled" : true,
    "compiler" : "\/Users\/jay3332\/Projects\/flashtex\/.claude\/worktrees\/agent-a2b5f9da704bfbf4b\/crates\/compiler\/target\/release\/flashtex-compiler",
    "nearby_diagnostics" : 0,
    "new_diagnostics" : 0,
    "new_errors" : 0,
    "new_messages" : [

    ],
    "pages" : 1,
    "status" : "ok",
    "zero_errors" : true
  },
  "context_revision" : 2,
  "enable_grok" : true,
  "image_bytes" : 197602,
  "key_source" : "Keychain",
  "latex_bytes" : 68,
  "latex_commands" : [
    "\\alpha",
    "\\beta",
    "\\int",
    "\\leq"
  ],
  "latex_environments" : [

  ],
  "model" : "grok-4.20-0309-non-reasoning",
  "note" : "xAI response id\/usage are not surfaced by the bridge's capture_proposal (helper request R4 in apps\/mac\/docs\/grok-live.md)",
  "outcome" : "proposal",
  "recorded_utc" : "2026-09-12T19:12:14Z",
  "required_dependencies" : [

  ],
  "seconds" : 2.8244140148162842,
  "supported_features_count" : 21,
  "supported_features_sha" : "49e6eb43808ac8fccb08d620b23667403fcb8667"
}
```

xAI response ids and token usage are not surfaced by the helper/bridge yet (helper request R4 in apps/mac/docs/grok-live.md); the ids above are the Mac's session/request ids.

## Run notes (added by the lane after the run)

- Working tree: commit 4b0fc3b6 plus the uncommitted changes committed next as
  "mac: live Grok acceptance …" (CaptureFeatures.swift, capture model override,
  rendered capture, compile gate). Environment for this run:
  `FLASHTEX_GROK_MODEL=grok-4.6 FLASHTEX_GROK_CAPTURE_MODEL=grok-4.20-0309-non-reasoning
  FLASHTEX_ASSISTANT_TIMEOUT_S=110`; the "model:" line above is the explanation model.
- Result: probe HTTP 200 in 0.17 s; explanation READY through the real helper's
  `--provider-session` (grok-4.6, 69.7 s, 570-byte explanation, 1 reviewed edit,
  `applied:false`, key only in the session child's environment, argv clean);
  capture conversion PROPOSAL through the real bridge `--enable-grok`
  (grok-4.20-0309-non-reasoning, 2.8 s, 68 bytes of LaTeX, 4 ambiguities
  reported, commands `\alpha \beta \int \leq`, no environments) and the demo
  gate through our `flashtex-compiler`: status **ok, 0 diagnostics, 0 errors, 1 page**.
- Earlier runs today (evidence directories removed as superseded; numbers kept here):
  1. 18:58Z — explanation READY (grok-4.6, 53.5 s); conversion HTTP 400 in 0.44 s
     with the 1×1 protocol fixture PNG (xAI rejects it).
  2. 19:05Z — rendered 900×260 PNG: conversion hit the bridge's fixed 90 s
     timeout on grok-4.6; explanation expired at the helper's 60 s flight limit.
  3. 19:09Z — grok-4.20-0309-non-reasoning: conversion in 3.2 s but bare math
     (no `$`), compile gate 10 errors; explanation refused by the helper
     ("removed source differs": the non-reasoning model proposed an edit whose
     removed text did not match the source — validation worked as designed).
  4. 19:10Z — with the OUTPUT wrapping rule: conversion 2.2 s, gate 1 error
     (`\tau` — the model read the handwritten π as τ and used a symbol outside
     the supported list); explanation READY (grok-4.6, 60.2 s).
- Token usage and xAI response ids are not available through the helpers
  (request R4); no prompt or reply body was recorded anywhere.
