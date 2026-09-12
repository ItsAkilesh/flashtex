# Actual helper / multi-document producer capture

`benchmarks/display-helper-multidoc` contains original producer input/output JSONL,
original helper output JSONL, observed helper snapshot, reopened ledger documents,
phase diagnostics and binary/asset/artifact hashes. The Linux capture proxy forwards
original bytes; it does not synthesize compiler replies. Its input log proves a
proxy read and output log proves producer emission. Only correlated helper output
establishes delivery. The runtime owner performs independent consumption separately.

Sources start with editor revisions main.tex7, chapter.tex19, refs.bib23. The startup
explicitly declares refs.bib as bibliography; the actual helper snapshot confirms
that kind. Producer requests contain path/text and compile generations1–6; editor
revision identities remain in helper source_versions. Bibliography kind is index
metadata, not an added producer request field.

The capture observes default v1 generation1, explicit raw display generation2,
restart/default v1 generation3, and explicit re-enable/raw generation4. For the
stale case the proxy holds a producer-emitted generation5 frame, then a newer edit
becomes durable. After release, generation6 is current with chapter revision21;
no preview/candidate with chapter revision20 escapes after the newer ACK. This is
supersession acceptance, not a direct user cancel-command or native paint test.

For current captures, v1/v2 semantic output matches original emitted frames; raw
nested display bodies retain the producer's exact bytes. Every document's hash,
UTF8 byte length and compile revision binds correctly, independently of editor
revisions. Killing the helper and reopening each actual ledger preserves exact
final text/hash/revisions7/21/23. Both recorded producer PIDs were absent after
termination. The capture proxy uses Linux parent-death signaling with the expected
proxy PID captured before spawning, avoiding a producer orphan on proxy kill.

All six actual v1 results are recovered with a tfm_missing diagnostic; full details
are retained in provenance and original output. This proves the source/transport
checks, not full math/font/bibliography layout compatibility or pixel parity. No
latency comparison or native activation is inferred. Baseline and first gated
captures preceded the final capture; only the final capture includes every reopen,
raw-byte and post-ACK stale rejection check.
