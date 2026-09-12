# Actual helper display stage attribution

Release helper `72af2d43594d4a472a1193ef2c0ab49ee9bc3397`, unchanged debug
producer `65dbe7da7a182e99322070e2c9763cc3b69a342b`, same repeat-100 fixture
as `../display-helper-scale100`. Provenance records executable, source, asset,
script and compressed artifact hashes. No caps or protocol changed.

All three states have zero diagnostics, exact direct v1/v2 JSON equality and
validated source identities. Final helper kill/reopen preserves exact source.

| Compile revision | Framed v2 bytes | Parse ms | Decode queue ms | Owner delivery ms | Source binding ms | Optional serialization ms | Candidate receipt ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| 2 | 364275 | 27.667 | 2.436 | 1.953 | 0.061 | 9.751 | 165.731 |
| 3 | 739459 | 82.459 | 0.568 | 1.702 | 0.147 | 23.904 | 596.438 |
| 4 | 1150523 | 123.705 | 3.009 | 1.941 | 0.288 | 37.925 | 928.801 |

Transport profiles identify requests preview-2/3/4 and display epoch 1. Optional
serialization entries are paired by the sequential replay: one admitted candidate
per state, followed by the next edit only after receipt. Source revisions are
1/2/3, deliberately distinct from compile revisions 2/3/4.

These are three single wall-time observations, not percentiles or native paint.
Peer integration builds were terminal before this run, but host scheduling and
external load were not controlled or recorded. Total receipts are substantially
slower than the previous sample; this does not demonstrate a code regression or
justify a before/after speedup ratio. Producer compilation, v1 work, IO and Python
receipt costs are not fully attributed by these scalar display stages.

Within this sample parsing is the largest instrumented display stage; source
binding and both waits are smaller. Extra decoder concurrency or weakening source
validation is therefore not supported by this evidence. The next bounded work is
an exact-output investigation of avoidable JSON Value serialization/representation
cost, coordinating runtime-owned parsing with FT049. Retain complete frames,
required-output priority and stale-source rejection. No raw passthrough or reduced
validation should be inferred from this measurement.

Reproduce with `examples/helper_display_replay.py` using the pinned helper and
producer/provenance/fixture paths in the adjacent provenance, `--repeat 100`,
`--compress-artifacts`, and a fresh `--output` directory. This run used the same
command shape as the earlier scale100 evidence, with the helper rebuilt using
`CARGO_INCREMENTAL=0 cargo build --release`.
