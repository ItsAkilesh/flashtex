# Optional metadata-only ordinary edit acknowledgement

An ordinary `edit` request may include `response_mode:"metadata"`. Omitting it or
using `"full"` retains the existing full Document response. Unknown/non-string
values reject before durable source mutation. This option does not apply to
capture approval, grouped edits, undo/redo or permanent command receipts.

A metadata result echoes `response_mode:"metadata"` and contains `document` with
`project_id`, `path`, durable `revision`, `source_sha256` and UTF-8 `byte_length`.
It omits `text`, and retains `preview_error` and `save_and_submit_ms`. A preview
failure still permits a successful durable edit acknowledgement. The response is
an acknowledgement of saved source, not evidence of current preview paint.

Native callers must retain the exact submitted source until reconciling its
acknowledgement. Verify session/request/project/path/revision/hash/byte count;
do not overwrite newer local typing when an older acknowledgement arrives. On a
lost reply, reopen/query the authoritative document and compare its identity before
retrying. Ordinary edits remain revision/hash guarded, not permanent-ID operations;
replaying the old expected revision rejects after a successful save. The existing
permanent-ID semantics of reviewed/grouped operations are unchanged.

Older helpers may ignore the optional request field and return the full Document.
Accept that existing response when valid; never resend a successful edit merely
because the metadata echo is missing. Adoption in the native app is still pending.

The ledger now exposes `replace_document_in_place` for this path, reusing the same
history recording and durable commit without cloning a response Document. The
existing `replace_document` API calls that path and clones the saved result for
legacy callers. The controller shares indexing/compile completion logic between
both modes. Source compilation still needs its normal input snapshot; this change
does not claim all source copying is eliminated.

Actual helper checks cover a500KB Unicode edit, metadata frame below1KB, exact
reopen/hash, premutation invalid-policy rejection, stale retry, default full reply,
and an unconsumed application ACK followed by kill/reopen/reconciliation. Existing
ledger history, receipt, checkpoint, crash and helper/compiler suites pass.

The isolated paired wire probe in `../benchmarks/edit-ack-metadata` encodes the same
500KB Unicode source: full reply500,288bytes versus metadata326bytes. Full encoding
0.250–0.574ms versus metadata0.00045–0.00190ms in ten alternating pairs. This is
synthetic response encoding only, not ledger fsync, indexing, native IO or paint.
