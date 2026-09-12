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
