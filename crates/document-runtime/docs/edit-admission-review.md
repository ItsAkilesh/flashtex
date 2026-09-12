# Operation-specific edit admission review

Read exact helper3014ade2 against runtime admission/event semantics. No workload
or production change. `CompileAdmission` is constructed from the operation-local
ID and generation only after runtime submission succeeds; full and metadata edit
paths preserve the same nullable pair. There is no fallback to an earlier
`submitted` field. Index/admission failure leaves the new source durable but has
no new receipt. Existing controller invalidation of old source currentness remains
separate from the runtime's still-active older request.

The held-child owner test explicitly observes startup before submitting two edits,
then matches `Superseded {id, by_id}` to their receipts. Its subsequent2048-byte
source under512-byte request framing fails admission, retains source and returns
None rather than the earlier successful receipt. These are reviewed owner tests,
not an independently repeated suite. Published docs correctly distinguish queued
admission from dispatch, completion, paint and source-action authority.

Runtime can admit successfully then immediately emit Failed(id) if writer delivery
fails. Pre-admission failures occur before queue/latest mutation; no event promises
a rejected ID. Superseded identifies both accepted old and replacement requests;
Cancelled/Failed identify accepted requests. Stale includes actual compile revision.
The helper's Discarded addition takes the matched Preview revision, not current
controller generation or editor revision. Full/metadata consumers can correlate
these outcomes using session plus receipt request ID without a new runtime field.

A caller must clear pending state on restart/session replacement and terminal
failure; an old receipt cannot establish source currentness. Unknown startup,
manual-compile or other-operation IDs must not be guessed from editor revisions.
Grouped/history/apply result schemas remain outside this change. No missing runtime
identity field was demonstrated for the explicitly scoped full/metadata edit path;
the owner's additional actual wire mapping gate is separate acceptance evidence.

## Grouped permanent retry followup

Pinned helperac0a860e extends only `apply_group` wire output, in full and metadata
modes. Both use the same operation-local admission path. The owner wire test holds
the child, applies a command at document revision2, advances current source to3,
and retries the permanent command: command_revision2/replayed remain durable
history, while a fresh compile ID/revision refers to current document3 and matches
the later preview. Neither the original command admission nor an intervening
submission is reused. Null admission on failure remains the existing path.
Undo/redo omit the new wire keys; an explicit undo key-absence assertion protects
that scope. No owner tests were repeated. The additional grouped encode-refusal
check is pending separately from this exact-SHA review. A minor stale STDIO line
claiming all group schemas unchanged was reported for documentation cleanup; the
new following paragraph and implementation clearly describe the additive change.
