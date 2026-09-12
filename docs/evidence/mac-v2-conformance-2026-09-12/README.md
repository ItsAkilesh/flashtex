# mac-v2-conformance evidence (2026-09-12, mac-m1max-a)

## D6 — real helper stdin bound (`helper-stdin-measurement.jsonl`)

Helper: `crates/preview-controller` built at 5997f372 (release), compiler
`crates/compiler` (release), launched by `measure_stdin.py` (scratchpad; a
fresh helper per case, file-backed project `main.tex`, no UI). Each case sends
ONE `edit` whose complete JSONL line (newline included) has the stated size,
then a `document` request to see whether the helper still answers.

| edit line bytes | helper reply | afterwards |
|---|---|---|
| 1,044,483 | `result` (durable revision 2) | answers `document` |
| 1,048,576 (exactly 1 MiB) | `result` (durable revision 2) | answers `document` |
| 1,048,577 (1 MiB + 1) | `{"id":null,"payload":{"message":"truncated or oversized input"},"protocol_version":1,"session_id":"d6","type":"error"}` | stdout EOF, process exits with status 0; the next write is a broken pipe |
| 1,048,580 / 1,048,581 | same error | same exit |
| 2,097,156 (≈2 MiB) | same error (the write itself already hits a broken pipe) | same exit |

Conclusion: the draft's "helper stdin is bounded at 1 MiB" is CORRECT
(`MAX_FRAME = 1024 * 1024`, `main.rs:30,189-201`: `read_until` capped at
MAX_FRAME+1, frame must be ≤ 1 MiB and newline-terminated, else `failure(...)`
then `break` — the reader thread ends and the helper shuts down). The Mac
client had no outgoing bound on this route (`PreviewControllerClient.send`);
`TransferV1.maxLineBytes` (12 MiB) belongs to the bridge contract and is not
the helper's bound. Fix: `PreviewControllerClient.maxRequestLineBytes = 1 MiB`
with a typed `RequestTooLarge` thrown before any byte is written (the shell's
`controllerSubmitEdit` already surfaces thrown errors as
`controllerStatus = "edit failed to send: …"`).
