# Durable ledger cost experiment

Measured on Linux 6.19.10, Rust 1.98.1, release mode, with cached dependencies.
The persistent-disk run uses encrypted Btrfs under `/home`; `/tmp` is tmpfs.
This measures the actual file-sync/rename/directory-sync implementation, not native
paint latency or power-loss behavior. Other project workers shared the host, and
the 500 KB run has large scheduling outliers. These are observations, not latency
guarantees or an isolated hardware benchmark.

The opt-in `cost_benchmark::durable_cost_matrix` uses decimal 5,000/50,000/500,000
byte documents, 20 capture edits with confirmation, undo and redo, explicit history
retention capped at eight entries, and two rotated checkpoints. Four initial
samples are discarded; each metric reports 16 samples. History is still filling
in the earliest measured rounds. Each final whole serialized state must equal
the reopened state, and every original capture must replay its original receipt.
The output records final source hashes and actual bundle sizes (101,329 / 866,329 /
8,516,329 bytes). Pending capture snapshots briefly add further copies during edits.

## Persistent-disk results

Median milliseconds from `btrfs-paired.txt`:

| Source bytes | Durable edit | Undo | Checkpoint rotation | Atomic write + sync | State serialize | State validate | State clone |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 5,000 | 1.678 | 1.581 | 6.471 | 1.542 | 0.054 | 0.057 | 0.014 |
| 50,000 | 4.127 | 3.254 | 21.384 | 2.111 | 0.459 | 0.498 | 0.065 |
| 500,000 | 36.418 | 18.805 | 170.196 | 8.020 | 6.951 | 5.241 | 0.596 |

Phase probes operate on the current confirmed state, so they do not sum exactly
to an edit, which also validates the pending pre-edit snapshot and computes new
hashes. Disk sync dominates small bundles. At 500 KB, serialization plus validation
exceeds the isolated write cost; cloning is much smaller. Checkpoint rotation also
reads and validates retained backups, so its cost exceeds encoding a single copy.

## Retained optimization

Checkpoint `encode` previously serialized the complete envelope during validation
to check its size, discarded that allocation, then serialized it again for return.
It now returns those same validated bytes. Version, state validation, identity,
integrity, byte limit and all durable write ordering remain identical. No wire
schema, retention, receipt, history or filesystem durability semantics changed.

Same-process paired old/new encoding verifies byte-for-byte equality on every
round. Median legacy → reused encode times were 0.250 → 0.202 ms at 5 KB,
2.256 → 1.826 ms at 50 KB, and 24.092 → 19.429 ms at 500 KB: approximately
19% less encode time in each case. This removes one whole-bundle serialization
and allocation. These measurements do not establish a 19% improvement to total
rotation or edit latency. The separate tmpfs baseline/optimized runs are retained
as raw evidence; varying host load makes their cross-run timing deltas unsuitable
for attributing speedups.

Reproduce on a persistent filesystem (choose a private existing parent):

```sh
TMPDIR=/path/on/persistent/filesystem cargo test --manifest-path crates/edit-ledger/Cargo.toml --release --offline --lib durable_cost_matrix -- --ignored --nocapture
```

The comparison's legacy path exactly matches the former `validate(); to_vec()`
implementation. `SHA256SUMS` binds the harness, relevant implementation files and
raw outputs. Baseline production revision: `9bd7663b06c27c28deb4afdefac518d31bc03455`.

The added 120-edit service regression repeatedly abandons accepted replies,
checks capacity-one rejection, retries and confirms every receipt, compacts
acknowledged payloads and explicitly trims undo history. Restart must preserve
the exact final source and all 120 permanent capture identities. Dropping a reply
only cancels observation; it does not cancel an admitted transaction.
