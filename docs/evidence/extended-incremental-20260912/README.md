# Extended incremental consistency acceptance

Codex `mac-reference-corpus` ran all fourteen recorded UTF-8 edits in both legacy
and typed runtime-v1 modes against the existing local original-compiler artifact.
**28 scenario/profile runs, 84 byte-exact comparisons, zero failures.**
Each warm worker receives original, edited, undo and reapplied snapshots of the
whole project. Each of its final three replies is compared to the complete raw
JSONL reply from a separate new worker receiving the identical request. No JSON
normalization, ID/revision rewriting, geometry tolerance or output filtering is
used. This covers macro/register dependency edits, math alignment/sizing,
included-file and graph-data edits, include selection, and invalid-syntax repair.

The compiler SHA-256 was unchanged before/after:
`1d615ef71e59435a45846aaf5b763d4e17589e6ce5ee1b55d4a47859c1c98238`.
Its build-source revision is **unknown**; these results are not current-main
acceptance. Total measured worker wall time was 0.619 seconds across 112 processes.
The default bound was 20 seconds per worker. This is a small-fixture correctness
check, not an interactive latency or cache-efficiency claim.

```sh
python3 tools/extended-tex-corpus/incremental.py \
  --compiler /Users/jay3332/Projects/flashtex/crates/compiler/target/release/flashtex-compiler \
  --output /private/tmp/flashtex-hw1-incremental-acceptance-20260912
python3 -m unittest discover -s tools/extended-tex-corpus -v
```

The complete development suite passed **13 tests**. Six acceptance-gate tests
exercise actual bounded subprocesses with deliberate stale warm output, bad
correlation, missing/malformed replies, nonzero exits and timeouts, plus included
file preservation, exact undo/reapply, UTF-8 byte offsets and raw whitespace
differences. Seven existing tests check source/PDF hashes and reference tooling.

`report.json` records compiler, manifest, edit and source hashes, process state,
durations and comparison results. `raw-traces.zip` contains all exact requests,
raw stdout, stderr and per-scenario results, including the original absolute
artifact path. `artifact-hashes.json` pins the executed runner/tests and these
artifacts. The deterministic ZIP is only storage compression; comparison used
uncompressed raw replies.

Consistent recovered output can still diagnose unsupported features. This gate
does **not** establish TeX feature support, complete recovery quality, rendered
PDF correctness, reference-PDF parity or warm auxiliary-file semantics. Edited
TeX oracles and complete rendered-page comparisons remain separate gates. No
provider calls, paid CLI calls, children, owner worktree edits or main writes
were made for this acceptance increment.
