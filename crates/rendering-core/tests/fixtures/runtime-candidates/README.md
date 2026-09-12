# Actual runtime/helper candidate acceptance

Artifacts are copied unchanged from FT049 b9240b0 and helper8876279; manifest.json
pins exact bytes. The actual producer is65dbe7d with existing official LM12
font/TFM/license assets. Runtime JSONL retains original framing; helper fixtures
retain their direct producer reply and forwarded event/source states.

The requested raw sibling passes existing pipeline_frame pairing, then full
PipelineCff structural/source/font-byte validation and searchable PDF export.
Legacy/declined/failed outputs do not create candidates; attaching a sibling is
refused. No rawSHA, original GID, document revision or source metadata is repaired.

All three helper states pass the new opt-in helper_candidate binder and export
identical PDF bytes to their corresponding direct producer envelope through the
same existing backend. Editor source revisions1/2/3 remain distinct from compile
revisions2/3/4. V2 SourceSnapshot revisions intentionally use compile generation;
editor versions are checked separately against the caller's current controller
snapshot. This explicit translation is not mutation of producer metadata.

The authoritative CurrentHelper must come from the live controller, not from the
untrusted event. It includes session, project, request, compile and membership
generations plus exact editor versions/text. Bind checks outer candidate policy,
all identities, complete source membership and inner result/sibling correlation.
Existing immutable resource validation checks source hashes/lengths, geometry,
features, original GIDs and full font bytes. Export requires a fresh identical
controller snapshot again; caller must advance generation across A→B→A changes.
A caller replaying an obsolete CurrentHelper is not proving freshness.

Tests reject stale session/membership/compile/editor version/text, missing font,
wrong raw font SHA/glyph count/GID and malformed JSON. Bound objects expose no
native paint or source-navigation authority. No native switch is activated.
Fixture tests use existing owned LM12 bytes/license from sibling fixtures, with
manifest-profile adaptation through the established loader; wire bytes remain
unchanged. No parser/writer/subsetter is duplicated.

Run `cargo test --manifest-path crates/rendering-core/Cargo.toml --test
helper_candidate`. These are actual fixture transport-to-export checks plus
explicit mutated negative cases, not live Mac paint, typing latency, established
reference parity or a permission grant. Five earlier reference reports remain
separate and retain their original runner/source hashes.
