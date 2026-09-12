# Durable native edit ledger

Owner: Codex Astra product engineer. September 12, 2026. Status: reusable Rust
core and local JSON Lines executable, tested on Linux; Mac integration remains
with the Mac owner. This resolves the storage primitive missing from the source
review in [issue 2](https://github.com/flash-tex/flashtex/issues/2#issuecomment-5643795185).
Owned paths: `crates/edit-ledger` only. No native app or bridge files are changed.

Lock cleanup explicitly unlocks on Store drop. A Unix fork regression reproduced
`store_in_use` before this fix when a still-running child inherited the lock's
open-file description; the same test passes after the explicit unlock, without
waiting for child exec/exit. This addresses the mechanism investigated in issue 17.

The source document and all applied edit IDs live in **one** `document.json`.
An apply validates project, path, revision, SHA-256 of UTF-8 source, scalar-aligned
byte range, and removed text. It commits updated source and the receipt ledger
together: write a private temporary file, sync it, atomically rename it over the
record, then sync the directory. A receipt is returned only after that completes.
An exclusive file lock excludes another writer, including other processes.

Identical prepared-edit retries return the original receipt without changing
source. Reusing an edit ID with different fields or a capture ID with a new edit
ID is rejected. Ordinary edits and undo advance the durable document revision
without removing applied-ID tombstones. Reinitializing an existing store with
different source is refused. Corrupt or unreadable state fails closed.

The `PreparedEdit` fields exactly match transfer-v1 at bridge revision
`b5ca96bdaca634166a01023cd3321955f2bc5f70`: `capture_id`, `edit_id`, `project_id`,
`path`, `expected_revision`, `start_byte`, `end_byte`, `removed_text`,
`replacement`, `document_before_sha256`. The library is independent of the
bridge crate and makes no provider calls.

## Native integration sequence

1. Open one private store directory per document; its parent must already exist.
   Load `document()` as the authoritative source. A separately saved `.tex` file
   is an export of this state, not a replacement for crash recovery. Initialize
   only a new store; never initialize over existing durable source.
2. On restart, use `recovery()` in revision order. Ask the bridge `capture_status`
   before acting. If its matching receipt is absent, reopen the retained
   `document_before` snapshot, then send that transaction's `receipt` as
   `capture_applied`. Confirm locally only after an exact matching bridge
   acknowledgement. Transport errors must retain these entries unchanged.
   Finally synchronize the current durable document with the bridge.
3. After explicit user approval and bridge preparation, call `apply(edit)` on a
   serial background I/O queue. Adopt the returned durable document in one
   undoable editor operation. Send the returned receipt to the bridge; do not
   also send `document_edit` for the capture edit. A retry can return an old
   receipt and the already-advanced document: adopt current durable source,
   never insert the replacement a second time.
4. Route ordinary typing/undo through `replace_document` using its expected
   revision and source hash. Distinguish the programmatic adoption in step 3
   from an ordinary edit so it is not persisted or forwarded twice.
5. On persistence error, send no receipt. The live handle is poisoned because
   rename might already have completed. Drop it, reopen, and reconcile the
   complete on-disk state; do not trust the pre-error in-memory snapshot.

This crate supplies durable state, not AppKit undo or bridge transport. The Mac
must integrate the transaction before marking the source edit applied. Native
UI/session guards and responsiveness fixes identified in issue 2 are separate.

## Build and check

```sh
cargo test --manifest-path crates/edit-ledger/Cargo.toml --offline
cargo clippy --manifest-path crates/edit-ledger/Cargo.toml --all-targets --offline -- -D warnings
cargo build --manifest-path crates/edit-ledger/Cargo.toml --release --offline
```

Validation: 55 library tests and nine integration tests pass on Linux, plus an
explicit opt-in release benchmark. See [measured costs](benchmarks/README.md) for
5/50/500 KB persistent-disk results and exact restart checks. They
exercise UTF-8 interiors and bad ranges, all snapshot guards, persisted replay,
undo with durable deduplication, before/after-rename I/O failures, failed receipt
confirmation, corrupted/unreadable journals, competing handles, stale snapshots,
and an actual killed helper process after local commit but before bridge receipt.
Clippy passes with warnings denied. No Xcode/device test or physical power-loss
claim is made; durability relies on the host filesystem honoring sync and atomic
same-directory rename. Linux and macOS filesystems are the intended targets.

## Background service adapter

`service::BackgroundService::start` spawns a worker that opens and exclusively
owns the store. Native library callers use `try_submit(frame)` and
`PendingReply::try_recv()`; these methods never open/sync files or write pipes.
Reply serialization, JSON decoding, document hashing and filesystem transactions run
on the worker. Do not use the blocking `wait()` convenience on MainActor.
The executable hosts the same service; its stdin/stdout host may block, so a
native Process adapter must keep pipe I/O on its own serial background queue.
No Swift/FFI binding or native integration is claimed by this Rust crate.

Admission accepts exactly one newline-terminated request of at most 12 MiB.
Default capacity is four outstanding commands including undrained replies;
`busy` is returned immediately before accepting excess work. Each command has
one reply slot, so a stalled/dropped reply reader cannot block other admitted
commands. Dropping a reply does not cancel an accepted transaction: export and
reconcile durable state before retrying. Dropping the last service handle closes
admission; its worker finishes accepted work and releases the store lock.
Use `shutdown()` and poll `ShutdownWatch::is_stopped()` to prove cleanup is
finished before attaching another session to that directory. Every cloned handle
must close first; checking the flag never blocks on I/O.

Replies include ledger-local `session_id`, monotonic execution `sequence`,
`document_revision`, `document_sha256`, and `command_succeeded`. Native consumers
must compare the current session identity before updating UI and reject older
sequence/revision observations. These fields do not alter transfer-v1 bridge
messages. Startup lock/storage errors arrive as asynchronous request error events.

Reply size defaults to 16 MiB (configurable up to 128 MiB); oversized payloads are
omitted with `reply_too_large`. `command_succeeded: true` and the durable revision
still report a completed operation, so never interpret omitted output as rollback.
Use a sufficiently bounded background recovery consumer for large documents.
The library caps capacity at 16 and retains at most the configured outstanding
request/reply count internally. Caller-owned collected replies are the caller's
responsibility. Tests cover capacity, frames, omitted payloads, startup errors,
distinct sessions, and competing stale edits with revision-aware replies.
The multi-session stress fixture runs four simultaneous document services with
four writers each: 160 guarded groups persist exactly once under contention,
with explicit stale-revision retries and reopen checks. The checked-in
`tests/fixtures/native-undo-restart.json` drives a real helper process killed
after durable undo: restart recovers source, pending capture receipt and redo,
and duplicate capture/undo requests remain harmless. This exercises the adapter
protocol on Linux, not a native view or Xcode runtime.

## Durable grouped editing and undo/redo

`apply_group` accepts a unique `command_id`, exact `expected_revision` and
`expected_sha256`, a label of at most 256 bytes, and 1–64 source edits. Each
edit carries `start_byte`, `end_byte`, `removed_text`, and `replacement` against
the same original source snapshot. Ranges must be scalar-aligned and nonoverlapping;
ambiguous insertion boundaries are rejected. All replacements form one atomic
source transaction and one undo unit, applied from the last byte range backwards.

Capture insertion and `replace_document` also record durable history. `undo` and
`redo` accept a fresh `command_id` plus exact revision/hash guards, advance the
document revision, and move one whole history entry. They never clear capture
receipts or release applied IDs. Identical retries of group/undo/redo commands
return their original command revision without moving history again, even after
restart or payload retention. Reusing the command ID with different fields or
another operation fails. `history_status` exposes retained undo/redo labels and
payload size; source and stacks recover together from `document.json`.

`retain_history` requires the current recovery `snapshot_token`, explicit
`acknowledge_undo_redo_loss: true`, and `keep_latest_undo`/`keep_latest_redo` counts.
It discards only excess history payloads. Capture IDs, pending receipt snapshots,
and permanent group/undo/redo command IDs remain. New edits conventionally
invalidate the redo branch but never its command IDs. Limits are 256 retained
entries, 32 MiB of before/after text and 4096 permanent history command IDs.
Exceeding a bound returns an error before mutation; no oldest-entry pruning is
implicit. Stores with history use at least schema 3 and are rejected by older readers;
payload compaction preserves the newer schema instead of downgrading it.
Legacy stores gain history for subsequent edits, not invented historical undo.

## Atomic checkpoint rotation

`rotate_checkpoint` writes internal backups under the private store's
`checkpoints` directory. Explicit export and deletion acknowledgements are
required. Retention keeps 1–32 latest complete checkpoints within a caller-set
budget of at most 512 MiB; a single checkpoint that exceeds that budget is
rejected before pruning. Source, receipt IDs, pending snapshots and retained
history remain bundled in each backup. Rotation does not compact live IDs.

Each checkpoint and its checksummed index use file sync, atomic rename and
directory sync. Old backups are deleted only after the selected index is durable.
An interrupted first rotation's valid unindexed backup is indexed before cleanup;
subsequent retries reconcile unreferenced files. At steady state the retained
files fit the policy; publication can temporarily add one checkpoint plus bounded
index metadata. A reduced policy requires space for existing backups until safe
pruning completes. Corrupt indexes, wrong identities and nonregular archive paths
fail explicitly and never trigger automatic archive reset.

`checkpoint_status` returns metadata without source text, including interrupted
temporary files or unindexed generations. Status does not authorize deletion or
source import. `checkpoint_read` reads only a committed generation. All service
replies retain session, sequence and current document revision/hash; discard stale
sessions in the native adapter. Inline checkpoint reads/imports remain subject
to service frame/reply caps; use the Rust bounded encode/decode and import APIs
for larger local files. These are ledger-local fields, not transfer-v1 additions.

Linux tests inject failures before checkpoint publication, before index publication
and after index publication. A real helper test kills an owner with an unread
rotation request, reopens, rotates, restores into an explicitly approved empty
store and verifies source, receipt deduplication, pending recovery and undo history
after another restart. The kill's exact filesystem boundary is scheduler-dependent;
the injected unit tests cover deterministic boundaries. No native/device or power
loss validation is claimed.

## Internal checkpoint export and reviewed import

`export_checkpoint` requires `acknowledge_private_source_export: true`; its result
contains the full private durable source, history and permanent receipt/command
IDs. It is a version-1 internal backup envelope, bounded to the 128 MiB store cap
plus 4096 bytes of envelope metadata. SHA-256 covers version, timestamp, identity
and all state. This is an integrity checksum, not an authenticated signature.
New stores carry a persistent random 256-bit `store_id` under schema 4. Exporting
a legacy store durably assigns that identity before returning the checkpoint.

`plan_checkpoint_import` validates the checkpoint and explicit expected
`{store_id,project_id,path}`. It returns a target-directory-bound plan with source
comparison and conflicts. Applying requires approving that exact `plan_id` plus
the relevant `allow_initialize_empty` or `allow_same_source_metadata` flag.
Any changed target/checkpoint invalidates the reviewed plan.

Restore into an explicitly authorized **empty isolated store** to recover source.
A live store can import metadata only when its source and revision match exactly,
and all existing permanent IDs, receipt confirmations, command IDs and retained
undo/redo payloads are preserved. Older/divergent source or lost/newer receipts
produce a blocked plan; neither source nor ledger is overwritten. To recover from
a corrupt live record, leave that directory intact and restore into a new empty
one, then let the native owner review cutover. This API never reseeds or rolls
back a live store. Checkpoint backup is separate from opening/saving a native
project or exporting `.tex` files.

These commands and fields are ledger-local, not transfer-v1 additions. The native
adapter adopts returned durable source rather than independently applying a
second edit. AppKit grouping and keyboard actions remain native integration work.

## Private JSON Lines helper

Launch `flashtex-edit-ledger --store /private/existing-parent/document-store`.
Use private stdin/stdout pipes from a serial background adapter. Each request is
one newline-terminated UTF-8 JSON object, at most 12 MiB, with a nonempty `id` of
at most 128 bytes. This is a local storage protocol, not a replacement for the
bridge's runtime envelopes. Responses preserve valid IDs and contain either
`payload` or structured `error: {code,message}`. Fatal framing errors terminate
the helper; startup errors arrive as service error replies. Request operations:

| Operation | Fields | Successful payload |
|---|---|---|
| `initialize` | `document: {project_id,path,revision,text,source_sha256}` | `document` |
| `status` | none | `document`, `pending_receipts` |
| `apply` | `edit: PreparedEdit` | durable `receipt`, current `document` |
| `replace_document` | `expected_revision`, `expected_sha256`, `text` | `document` |
| `confirm` | `receipt: {capture_id,edit_id,new_revision}` | `confirmed` |
| `recovery_export` | none | `snapshot_token`, `current_document`, `pending_receipts` |
| `recovery_import` | `recovery: {snapshot_token,observations}` | `actions`, fresh `recovery` export |
| `compact` | `policy: {snapshot_token,acknowledge_permanent_id_retention,acknowledged_through_revision,keep_latest_confirmed}` | compaction counts, sizes and fresh token |
| `apply_group` | `group: {command_id,expected_revision,expected_sha256,label,edits}` | current document, original command revision, replay flag, undo/redo availability |
| `undo`, `redo` | `command: {command_id,expected_revision,expected_sha256}` | current document, original command revision, replay flag, undo/redo availability |
| `history_status` | none | undo/redo labels, permanent command count, payload bytes |
| `retain_history` | `policy: {snapshot_token,acknowledge_undo_redo_loss,keep_latest_undo,keep_latest_redo}` | retained counts, dropped payload count, permanent command count |
| `checkpoint_status` | none | store identity, current revision/hash, retained metadata/bytes, unindexed generations and interrupted-write flag |
| `checkpoint_rotate` | `authorization: {acknowledge_private_source_export}`, `policy: {acknowledge_checkpoint_deletion,keep_latest,max_total_bytes}` | created checkpoint, retained metadata, removed filenames |
| `checkpoint_read` | `generation` | validated checkpoint from the committed index |
| `checkpoint_export` | `authorization: {acknowledge_private_source_export}` | internal checkpoint envelope |
| `checkpoint_plan` | `checkpoint`, `expected_identity: {store_id,project_id,path}` | reviewed import plan or blocked plan with conflicts |
| `checkpoint_import` | `checkpoint`, `plan`, `authorization: {approve_plan_id,allow_initialize_empty,allow_same_source_metadata}` | durable document after approved import |

Recovery export/import is a ledger-local API; it does not add bridge wire fields.
After export, query the bridge for each pending capture. Import observations
tagged `status: applied` with the exact `receipt`, `status: prepared` with the
exact `edit`, or `status: unavailable` with `capture_id` and a bounded `reason`.
An applied observation durably confirms that receipt. A prepared observation
returns a `replay_receipt` action with original source and receipt; it does not
reapply or replace the current document. Unavailable observations return a retry
action and retain all recovery evidence. Any conflict rejects the entire batch.
If source or ledger changed during the bridge round trip, the snapshot token is
stale and import is refused. Re-export and query again. Imported data never
contains document text to overwrite the authoritative source.

Compaction is explicit and only removes full prepared-edit payloads already
acknowledged by the bridge, at or below the supplied revision cutoff, after
preserving the requested number of latest confirmed payloads. The acknowledgement
flag must be true and the snapshot token must still match. Every edit/capture ID,
original receipt, and SHA-256 binding of all prepared fields remains permanent.
An identical replay still returns the original receipt after compaction and undo;
a changed payload remains a conflict. Pending receipt snapshots are never
compacted. Compacted files use at least schema 2 so an older schema-1 reader refuses them
instead of ignoring retained IDs. This is payload compaction, never ID garbage
collection; the 4096-ID bound remains explicit and non-evicting.

`pending_receipts` entries include prepared edit, receipt, original document,
after-source hash and confirmation flag. They retain original source until the
matching bridge acknowledgement. The core does not erase recovery data merely
because a network request fails.

Bounds: document 8 MiB, replacement 64 KiB, store 128 MiB, 4096 applied IDs.
The ledger fails explicitly when full; it never silently evicts IDs. Unconfirmed
edits retain a complete original document, so timely receipt reconciliation
matters for storage size. No ID garbage collection, encryption, native FFI, document
export UI or automatic bridge reconciliation is implemented here.

Resumption: branch `agent/mac-contract-review/edit-ledger`, worktree
`/tmp/flashtex-edit-ledger.pXf42s`, base main `1dd26c5`. No paid calls or provider
fallbacks; tests use only local processes and cached Rust dependencies. Next
consumer gate is native integration and fault testing with this store as the
authoritative document transaction.
