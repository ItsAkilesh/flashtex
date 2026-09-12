# Experimental whole-result chunk reassembly

Status: consumer prototype only. Production runtime-v1 and native protocol are
unchanged. Compiler/native owners must agree on a negotiated capability and its
begin/end envelopes before any producer or reader activation.

The caller constructs an Assembly with a bound Request (project, revision, ID,
source snapshot), announced result bytes and page count. Each JSON chunk repeats
project/revision/ID, a zero-based sequence and a nonempty UTF-8 string segment of
the original complete result JSON. The prototype supports at most64MiB total,
1MiB serialized chunk and10,000 pages. The caller selects a positive deadline up
to60seconds. It checks frame size before parsing and cumulative decoded bytes
before append. No allocation is made from an announced total alone.

No page is exposed before explicit finish. Finish requires exact announced byte
count, current revision, unexpired deadline, full existing runtime validation and
exact announced page count. Missing/duplicate/out-of-order chunks, wrong identity,
oversize, stale revision or cancellation terminally reject and clear the partial
buffer. The caller must cancel on document/epoch changes even if it reuses a numeric
revision; production Controller generations must remain monotonic. Failed stream
recovery needs a newly bound assembly; do not splice retries into old data.

Proposed future begin/end messages must bind a unique stream ID and a full-result
content hash, with bounded announced lengths. A producer cannot publish partial
page sets as a completed revision. Timeout or compiler failure retains the last
completed UI preview with an explicit stale/error marker, never half old/half new
pages. A transport disconnect aborts the unfinished assembly. The prototype has no
network authentication or producer negotiation; it must not be directly enabled
as a production wire protocol.

`cargo run --release --manifest-path crates/document-runtime/Cargo.toml --example
chunk_reassembly -- /tmp/single-request.json < /tmp/full-result.json` executes an
offline exact comparison. On the pinned compiler9026d8a500KB fixture,10,135,609
result bytes containing301pages reassembled exactly in155chunks. Largest chunk:
76,346bytes. JSON-string escaping increased wire payload to11,765,953bytes.
Packing/reassembling/validating took116.86ms in one Linux release run, excluding
compiler time and native painting. Peak RSS was not measured. The benchmark also
holds original JSON for comparison, so its memory is not consumer-only memory.

This prototype removes the single-frame requirement, not full-result allocation:
it retains a bounded byte buffer and parses the full result on completion. Parsed
JSON memory exceeds encoded bytes. It does not provide per-page memory residency,
compression, deltas, earlier paint, or lower latency. Next design comparison should
use typed page/item chunks and/or immutable page references to avoid JSON-string
escaping and permit bounded page staging with atomic final publication. Exact
reassembly and stale suppression remain mandatory acceptance gates.
