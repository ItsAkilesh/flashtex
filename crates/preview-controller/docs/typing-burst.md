# Sustained 50KB grouped typing acceptance

`examples/typing_burst.py` schedules20 visible grouped edits30ms apart on the same
50KB source/release helper/release producer used in the single-edit experiment.
It records actual send and flush timestamps independently from the receiver;
predicted revision/hash guards are never silently repaired or retried on refusal.
All edits are scalar-aligned byte ranges after a UTF8 prefix, with permanent IDs.

The initial run acknowledged all20 edits but emitted only final current revision21,
130.94ms after the last send. Independent review requested explicit monotonic/current
preview checks and an overall deadline, plus binding the final metric to revision21
rather than whichever frame was last. The strengthened rerun also acknowledged20,
with only final revision21 appearing98.03ms after the last send. The two original
captures and exact captured harness versions are preserved separately in
`benchmarks/typing-burst-50kb`; they are not a calibrated statistical comparison.

In the checked run, sends stayed within0.23ms of the intended schedule and ACK
arrival latency was at most3.10ms. No current preview arrived during the roughly
570ms send window. Fast durable ACKs and a final preview below200ms therefore do
not establish continuously updating current previews while typing. This gap stays
in GH21; no native paint, arbitrary-document or worst-case guarantee follows.

Both runs verify final exact source/revision21 after killing and reopening the
helper. A clean invocation uses the original captured final compiler request and
matches the actual current preview result. After measured typing, history contains
20 permanent IDs; retries of the first and last commands report command revisions2
and21 against current document21 without changing source. Changed fingerprint under
the first ID rejects. These post-measurement actions can request extra compilation;
they do not enter burst latency metrics. All recorded producer PIDs terminated.

The checked receiver refuses previews older than the latest delivered ACK and
nonincreasing preview revisions. An overall experiment deadline is checked around
each bounded reader call. Failure remains a failure rather than an extended timing
budget or permission wait. Original request/event/producer/clean-output/source and
phase artifacts are deterministically compressed with original and compressed hashes.
