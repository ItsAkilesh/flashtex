# Independent compiler scaling checkpoint

Commander requested root verification of Kabir's parser update. These artifacts
live on root's owned product branch for integration into the central performance
record; they measure the compiler runtime, not the assistant feature.

Candidate compiler source4425d9d, combined base23c3024 and exact binary/dependency
provenance are recorded in provenance.json. The pinned replay/generator source is
41602c0. Twenty alternating edits in a three-file project are compared with fresh
compiler processes using exact positioned-result JSON equality.

| Source bytes | Default8MiB frame | Explicit16MiB frame |
| --- | --- | --- |
| 5,000 | 20/20 equal; p95 3.483ms | 20/20 equal; p95 4.106ms |
| 50,000 | 20/20 equal; p95 28.480ms | 20/20 equal; p95 25.638ms |
| 500,000 | output validation failure before first sample | 20/20 equal; p50 261.596ms, p95 305.180ms, max406.182ms |

Timing includes request cloning/encoding, compiler, pipe transport, JSON validation
and polling. Native paint and reference PDF parity are not measured. The explicit
larger frame is an experiment, not a change to production defaults. GH21 remains
open: the default runtime rejects the large result, and total observed latency
still exceeds200ms with the larger limit. Historical9026 results have different
replay hashes and timing scope; no direct speedup ratio is inferred. This shared
machine was concurrently running other build/tests, so these are observed timings,
not isolated performance guarantees. Inputs are reproducible from pinned generator
hash and recorded input hashes; output samples retain all20large-case comparisons.
