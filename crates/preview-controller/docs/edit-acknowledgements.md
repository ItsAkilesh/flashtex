# Optional metadata-only ordinary edit acknowledgement

An ordinary `edit` request may include `response_mode:"metadata"`. Omitting it or
using `"full"` retains the existing full Document response. Unknown/non-string
values reject before durable source mutation. The grouped-edit extension is described below. This option does not apply to
capture approval replies. Permanent command semantics are unchanged.

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


## Grouped-edit metadata acknowledgement

`apply_group` also accepts the same top-level `response_mode:"metadata"`; the
existing `command` payload and its fingerprint do not change. The result echoes
that mode and returns `history` containing a metadata-only `document`, original
`command_revision`, `replayed_command`, `can_undo` and `can_redo`, plus the usual
preview error and save timing. Omitted/`full` mode retains the full history result.
Invalid modes reject before mutation or command-ID processing.

Exact retries use the same permanent command ID and identical command payload,
even after a lost reply/restart. A changed command under that ID rejects. Do not
mint a new ID merely because a reply was lost. The document in a replayed result
is the current durable document: after undo it can have revision3 while the old
command_revision remains2. That response does not mean the old edit was reapplied.
If current source has advanced, reconcile through the existing document/snapshot
query before applying any UI replacement or undo action.

Metadata projection borrows the existing HistoryResult document for indexing and
copies only metadata to the response. The ledger still returns its existing owned
HistoryResult and retains all ordinary history/permanent-ID records. This avoids
an extra controller response-source clone; it does not remove history snapshots or
claim zero-copy mutation. No new patch format, cap or source-approval shortcut is
introduced. Native adoption remains opt-in and has not been measured here.

An actual helper test starts with500KB of Unicode source, applies two byte-aligned
edits as one group, loses the application ACK, reopens and retries exactly. It
compares full and compact history flags/hash, rejects an ID conflict, undoes the
whole group and retries the old command again: command revision2/current revision3
and redo availability remain correct. The metadata response remains below1KB.

The grouped500KB synthetic wire probe is recorded separately in
`../benchmarks/group-ack-metadata/provenance.json`; it retains all history flags
and command revision. This measures reply serialization only, not native or
ledger performance.

## Undo and redo metadata acknowledgement

`undo` and `redo` accept the same response policy and compact `history` shape as
`apply_group`. Validation occurs before mutation. Their existing command payloads,
fingerprints, permanent IDs and full-response defaults are unchanged. The shared
controller adapter avoids its extra source clone while preserving the ledger's
owned history result and durable snapshots.

A retry of an old undo or redo returns its original command revision alongside the
current document identity and current undo/redo availability. It does not replay
the source change. Clients must reconcile a newer document rather than reconstruct
it from the old operation alone. Changing response mode does not change command
identity; changing command fields under an existing ID still rejects.

The real helper test exercises undo, redo with an unread application ACK, restart,
exact redo retry, a conflicting ID, a stale new undo, and both old commands after a
later ordinary edit. Full and compact replies agree on current hashes/history
flags; the 500KB redo response is below1KB and reopening preserves exact text.
This is protocol/recovery evidence, not native undo integration or paint timing.
