# Immutable history sharing and validation reuse

The exact521792-byte source changes20times without reducing retained undo history.
The baseline is parent276847f plus the new cost benchmark; the candidate changes
only in-memory history sharing and content-validation reuse. Both preserve all
history payloads, permanent IDs and the existing JSON schema. Each benchmark
checks byte-exact serialization after reopening its own store.

At edit20 (21,398,252 stored bytes in each run):

| Phase | Baseline | Candidate |
| --- | --- | --- |
| State clone | 7.373ms | 0.193ms |
| State validation | 41.109ms | 2.195ms |
| State serialization | 47.905ms | 90.661ms |
| Atomic persistence | 19.288ms | 32.032ms |
| Actual ordinary edit | 151.064ms | 141.110ms |

These separate shared-host runs show substantial variance in unchanged phases;
do not derive a stable end-to-end speedup. Sharing avoids duplicating immutable
before/after source strings during state cloning. Each private immutable entry
hash is checked once in memory, then reused; chain/document/receipt constraints
remain checked on each validation. Deserialization initializes empty caches, so
untrusted disk/checkpoint input is always revalidated. Serialization still writes
the entire retained history and remains an optimization opportunity.

66 ledger tests pass with strict all-target Clippy, including recovery, backup,
import, undo/redo, receipts and uncertain I/O. Two new tests verify exact legacy
entry serialization, shared identity, cache-independent equality, cold malformed
entry rejection and real disk-tampered history refusal on reopen.

Helper integration first run:52/53 pass; the307page large-output gate times out at
its unchanged10second deadline. One isolated retry also timed out. A bounded direct
probe of the exact debug helper/fixture then received the full307page preview in
1.420seconds (controller poll416.8ms), and the unchanged isolated Rust test passed
in9.56seconds. This is a latency-sensitive gate, not a clean first full pass; root
has reported the discrepancy rather than extending its deadline. All53 individual
helper checks have now passed and strict helper Clippy passes. No universal native
or large-document responsiveness claim follows from these checks.

Reproduce: `cargo test --release growing_source_history_cost -- --ignored --nocapture`
in this crate. Compare the complete raw phase records, not only the final sample.
