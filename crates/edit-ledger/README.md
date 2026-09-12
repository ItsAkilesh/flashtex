# Durable native edit ledger

Owner: Codex Astra product engineer. September 12, 2026. Status: reusable Rust
core and local JSON Lines executable, tested on Linux; Mac integration remains
with the Mac owner. This resolves the storage primitive missing from the source
review in [issue 2](https://github.com/flash-tex/flashtex/issues/2#issuecomment-5643795185).
Owned paths: `crates/edit-ledger` only. No native app or bridge files are changed.

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

Validation: 16 library tests and three subprocess tests pass on Linux. They
exercise UTF-8 interiors and bad ranges, all snapshot guards, persisted replay,
undo with durable deduplication, before/after-rename I/O failures, failed receipt
confirmation, corrupted/unreadable journals, competing handles, stale snapshots,
and an actual killed helper process after local commit but before bridge receipt.
Clippy passes with warnings denied. No Xcode/device test or physical power-loss
claim is made; durability relies on the host filesystem honoring sync and atomic
same-directory rename. Linux and macOS filesystems are the intended targets.

## Private JSON Lines helper

Launch `flashtex-edit-ledger --store /private/existing-parent/document-store`.
Use private stdin/stdout pipes from a serial background adapter. Each request is
one newline-terminated UTF-8 JSON object, at most 12 MiB, with a nonempty `id` of
at most 128 bytes. This is a local storage protocol, not a replacement for the
bridge's runtime envelopes. Responses preserve valid IDs and contain either
`payload` or structured `error: {code,message}`. Fatal framing/startup errors
terminate the helper. Request operations:

| Operation | Fields | Successful payload |
|---|---|---|
| `initialize` | `document: {project_id,path,revision,text,source_sha256}` | `document` |
| `status` | none | `document`, `pending_receipts` |
| `apply` | `edit: PreparedEdit` | durable `receipt`, current `document` |
| `replace_document` | `expected_revision`, `expected_sha256`, `text` | `document` |
| `confirm` | `receipt: {capture_id,edit_id,new_revision}` | `confirmed` |

`pending_receipts` entries include prepared edit, receipt, original document,
after-source hash and confirmation flag. They retain original source until the
matching bridge acknowledgement. The core does not erase recovery data merely
because a network request fails.

Bounds: document 8 MiB, replacement 64 KiB, store 128 MiB, 4096 applied IDs.
The ledger fails explicitly when full; it never silently evicts IDs. Unconfirmed
edits retain a complete original document, so timely receipt reconciliation
matters for storage size. No history compaction, encryption, native FFI, document
export UI or automatic bridge reconciliation is implemented here.

Resumption: branch `agent/mac-contract-review/edit-ledger`, worktree
`/tmp/flashtex-edit-ledger.pXf42s`, base main `1dd26c5`. No paid calls or provider
fallbacks; tests use only local processes and cached Rust dependencies. Next
consumer gate is native integration and fault testing with this store as the
authoritative document transaction.
