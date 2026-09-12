# Native helper protocol v1

Run `flashtex-preview-controller CONFIG.json` as a local child. The JSON config
contains `session_id` (unique per launch), `project_id`, `entry_path`, `store_paths`
(1–256 initialized ledger directories), and optional `compiler_path` pointing to
the original FlashTeX compiler. Missing compiler leaves editing available. Ledger
source is authoritative; importing/exporting `.tex` and project membership remain
native adapter responsibilities.

Use dedicated native reader/writer workers; never synchronously wait for stdin,
stdout, fsync, compilation or replies on the UI thread. Input is JSON Lines with
`protocol_version:1`, configured `session_id`, a nonempty `id` up to 128 bytes,
`type`, and `payload`. A single actor serializes source operations while separate
threads drain pipes; compilation updates are polled independently every 2ms.

Admission is bounded: 1MiB input frames, 16 waiting operations, 8 output frames of
at most 16MiB each. Full input queues return a correlated `error` stating the
operation was not admitted. Drain responses before retrying that operation.
Output overflow/stall ends the helper with delivery uncertainty; recover source
and pending receipts before retrying writes. Normal stdin EOF drains queued
responses for at most two seconds, then exits; it does not promise pending compile
completion. Never interpret successful pipe writing as a durable edit receipt.

Responses use `type:ready`, `type:result`, `type:error`, or asynchronous
`type:update` with a null request ID. Result/error IDs correlate to requests.
Preview updates include compile request/revision, exact document `source_versions`,
compiler envelope and runtime/controller timings. Native code must compare its
current editor versions at paint time and suppress outdated updates. Native
keystroke/queue/paint timings remain additional measurements.

Operations:

- `document`: `{path}` returns authoritative document/hash/revision.
- `edit`: `{path, expected_revision, expected_sha256, text}` returns durable source
  and a separate `preview_error`; that error never means the edit was undone.
- `compile`, `restart`, `close`: empty payload. Restart uses the configured binary.
- `review`: `{edit: PreparedEdit}` checks current source and returns the exact edit
  plus an unpredictable session-local `approval_token`. At most 128 reviews remain.
- `apply_reviewed`: `{approval_token, user_approved:true}` is sent only from the
  actual user approval handler after displaying that exact edit. The helper cannot
  prove a human approved; automatic conversion must never manufacture this call.
  It returns a durable receipt/current source and separate preview status. Reusing
  the same token is idempotent through the durable edit ledger.
- `retire_review`: `{approval_token}` frees session-local review capacity.
- `recovery`: `{path}` returns pending applied transactions after uncertain delivery
  or restart. Tokens are not persisted, but source and receipts are.
- `confirm_receipt`: `{path, receipt}` only after exact bridge acknowledgement.

The helper has no provider access, automatic approval, native rendering or fallback
TeX engine. Tests use real durable stores and real helper subprocesses; the optional
original-compiler test also exercises streamed positioned results.
