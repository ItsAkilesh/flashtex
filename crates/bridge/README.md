# FlashTeX Rust capture bridge

Original Rust application layer for capture receipt, source context, Grok
transcription and reviewed UTF-8 anchored insertion. This is a separate process
from the compiler. See the authoritative [transfer contract](../../docs/contracts/transfer-v1.md).

```sh
cargo test --manifest-path crates/bridge/Cargo.toml
cargo run --manifest-path crates/bridge/Cargo.toml -- --store /private/app-data/captures
```

The private journal must be outside the repository. Grok is disabled by default;
`--enable-grok` plus an explicit `capture_convert` request enables one request
using `XAI_API_KEY` from the native credential adapter. `FLASHTEX_GROK_MODEL` can
select a model; the default is `grok-4.6`. No provider fallback or purchase occurs.
The Responses request uses a strict JSON schema, image input and `store:false`.
References: [xAI image understanding](https://docs.x.ai/developers/model-capabilities/images/understanding)
and [structured outputs](https://docs.x.ai/developers/model-capabilities/text/structured-outputs).

Implemented: atomic durable receipt, duplicate detection, decoded image bounds,
source-aware context limits, deterministic provider response checks, anchor
rebasing/invalidation, review requirement and idempotent insertion receipts.
Tests use a deterministic fake converter by default; they do not spend API
credits. One successful live round trip has been made against the real xAI
endpoint, on a synthetic rendered-math image, proving the request/response
plumbing (auth, strict schema, image upload, response parsing) works for that
clean case. Actual handwriting and photo-of-paper conditions (lighting, paper
texture, camera noise, real strokes) remain unproven. A gated, `#[ignore]`d
integration test (`tests/live_grok.rs`) makes further live checks cheap to
run for anyone holding an `XAI_API_KEY`; it never runs in a normal `cargo
test` and asserts only on response structure, never on exact output.

Outstanding: encrypted paired nearby networking, real native credential and
transaction adapters, actual compiler validation before review, real Pencil and
camera evidence, end-to-end crash/device tests. Do not describe the local protocol
as a completed secure Mac/iPad connection or as proof of LaTeX compatibility.
