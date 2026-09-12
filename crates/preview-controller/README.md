# Durable editor preview controller

Pure Rust worker-layer integration of `edit-ledger`, `project-index` and
`document-runtime`. A native adapter owns one controller per project and calls it
on a worker thread: source persistence includes fsync and must not block the UI.

Pass initialized locked document stores and an explicitly configured original
compiler command into `Controller::new`. Construction recovers the index from
those durable documents; call `compile_current` for the initial preview.
`replace_document` accepts an expected revision/hash and the new source text.
Successful source persistence precedes index update and compilation. An `Ok`
`EditOutcome` always carries the durable document; `preview_error` reports a
subsequent cache/compiler failure without pretending the source save failed.
Storage uncertainty returns an error and requires dropping/reopening the ledger.

`poll` delivers only previews matching the current complete source version vector.
Before painting a retained event, native code must recheck `is_current_preview` on
the serialized controller worker: a newer edit may have arrived since delivery.
`close` cancels publication and further edits. Drop releases source store locks.
`restart` recreates disposable compiler/index state from durable source after a
compiler failure; it cannot repair a poisoned persistence handle in place.

This is not a cross-crate transaction. A crash between the durable edit and cache
update is recovered by rebuilding caches, with source remaining authoritative.
The controller does not write exported `.tex` files, invoke a provider, approve a
capture, implement UI undo, or render PDF. Store membership is fixed for this first
API; callers reopen with the intended set of initialized stores to change it.

`save_and_submit_ms` includes persistence/index/submission. Preview
`runtime_total_ms` includes the runtime queue and compiler/pipe/poll observation.
Neither measures native painting. Calling code must measure from the original
keystroke through the actual paint completion for the <200ms acceptance gate.

```sh
cargo test --manifest-path crates/preview-controller/Cargo.toml
FLASHTEX_TEST_COMPILER=/absolute/original/flashtex-compiler \
  cargo test --manifest-path crates/preview-controller/Cargo.toml -- --include-ignored
```

Synthetic tests exercise the actual durable ledger and index with a clearly fake
compiler. The opt-in original-compiler test additionally compiles persisted edited
source. Neither proves reference-PDF parity or full TeX/package compatibility.
