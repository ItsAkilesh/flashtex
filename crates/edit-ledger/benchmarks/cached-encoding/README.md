# Bounded immutable history JSON cache

Same-process alternating-order comparison against the exact former borrowed-field
serializer. Six pairs each at5,10,20retained entries:18/18encoded histories are
byte-identical and the cached path is faster in every measured pair. At20entries,
20,876,157encoded bytes, medians are41.026ms cached and93.685ms original. This is
warm serialization only, not source-save, compiler or native paint latency.

The `cold_us` raw field records the first serialization at each growing size;
earlier entries may already be warm. It is not a fully cold-cache benchmark.
All source/history strings remain retained and all output bytes remain unchanged.
The cache adds memory: at most1MiB per entry and at most that entry's retained
source bytes plus512bytes. A valid history thus retains no more than32MiB+128KiB
of cached JSON text, in addition to its original data. Allocator overhead, temporary
buffers and separately held snapshots are outside that figure. Oversized or
heavily escaped encoding falls back to ordinary serialization without dropping
payloads. Failed cache construction is remembered and is not repeatedly attempted.

Caches are private, immutable and absent from stored JSON. Deserialization starts
cold; supplied cache fields are ignored and cannot bypass content validation.
The implementation uses serde_json's checked RawValue constructor after bounded
serialization, preserving the former compact JSON field order and escaping.

67ledger tests and strict Clippy pass: exact legacy serialization, JSONValue
conversion, escaped fallback, allocation limits, cold tamper refusal, backup,
import, undo/redo, receipts and uncertain I/O remain covered. Full53helper tests
including real original-compiler gates also pass serially, with their original
thresholds. Prior history checkpoint's timeouts remain in its own evidence.

Run `cargo test --release paired_cached_history_encoding -- --ignored --nocapture`
in this crate. The standalone stored-file rewrite and filesystem sync costs are
unchanged; actual-helper/native measurements remain required after integration.

## Actual-helper follow-up

Release helper2f2605d, same pinned e75741e compiler, unchanged521792-byte source,
20edits targeting30ms intervals, explicit15MiB compiler frame allowance:

| Policy | Final after last send | Last ACK lag | Total request handling | Last edit handling |
| --- | --- | --- | --- | --- |
| Default strict current | 866.97ms | 356.12ms | 393.07ms | 32.32ms |
| Negotiated historical | 754.36ms | 262.91ms | 438.89ms | 34.26ms |

Both runs retain40/40acknowledged source versions overall, two exact independent
clean final results and two exact killed-helper source reopens. Each delivered
one final current preview and no historical frames during typing. These are
separate single samples; they do not establish a stable causal speedup against
earlier host runs. They are below earlier measured delays but still above200ms.
Native painting is unmeasured. Cached JSON reduces repeated serialization work;
complete state persistence and compiler reply processing remain substantial.
