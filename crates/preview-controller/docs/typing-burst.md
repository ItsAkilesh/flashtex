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

## Completed intermediate output and current-only delivery

`completion-disposition.json` audits the existing checked capture without running
another workload. It reconstructs every source revision from the original20 guarded
commands, verifies removed bytes and prior hashes, and binds each producer input and
emitted result to the helper event with the same request ID/generation.

Actual generations2,3,5,7,9,11,13,14,16,18 and20 completed successfully and became11
helper stale notifications. Eight pending generations were superseded. Only21 was
forwarded as a current preview. The lack of intermediate current previews is thus
not an absence of completed compiler results: completed results existed but no
longer matched the latest submitted source. This behavior preserves current-source
identity; the evidence does not justify calling those older results current.

An optional historical-display experiment can evaluate visible progress separately
with explicit stale labels, original source bindings and source actions disabled.
It must not change the default or claim current/native/every-keystroke acceptance.

## Explicit historical preview observation

`benchmarks/typing-burst-50kb-historical` uses the existing explicit
completed-snapshots-v1 option with one source-binding token for each burst edit.
It preserves the same source,20 command payloads (apart from those tokens), target
30ms schedule and release binaries. The default remains off and no native feature
is activated. Historical display is disabled again before old command retry tests.

All20 ACKs succeeded. Seven historical snapshots arrived at approximately86,172,
254,338,423,509 and595ms, binding editor revisions2,4,7,10,12,15 and18 respectively.
Six arrived before the last send at570ms. Each was explicitly is_current=false and
source_actions_enabled=false, with its exact original binding token and original
producer result/request source. Only revision21 was current, about98.92ms after
the last send. Clean final compilation, exact reopen and receipt/conflict gates pass.

The Python sender thread lagged its intended schedule by up to23.91ms, compared
with0.23ms in the earlier current-only capture. Maximum observed ACK arrival latency
was36.56ms. Receiving/decoding large historical frames can contend with the sender
in the same Python process; these measurements do not isolate that effect or
establish a matched-cadence latency tradeoff. They demonstrate source-bound older
visual progress, not current output for each keystroke or native paint performance.
A separately scheduled sender with deterministic lifecycle tests is needed before
any future controlled cadence comparison. No additional comparison run was made.

Post-capture review strengthened the harness to require the historical result ID
and compile revision to match its envelope and to locate a unique request/output
pair within the same captured producer stream. All seven original historical
results pass this audit; altered result IDs and compile revisions are rejected.
`correlation-review.json` records this later verification separately; original
measurement provenance and captured bytes remain unchanged.

## Separate-process sender (next experiment harness)

The harness now uses `scheduled_sender.py` rather than a receiver-process Python
thread. It inherits only the helper input descriptor, reads pre-encoded commands,
and records actual send/write-completion times on the shared monotonic clock.
The parent flushes before launch and sends no commands until this process exits.
A 200ms startup lead is not a scheduling guarantee; lateness must still be reported.

An absolute 30-second child deadline bounds waits and pipe backpressure. Writes
use at most PIPE_BUF bytes after writability, with one writer for the burst.
Parent cancellation and finish-timeout kill and reap the child, and normal cleanup
stops it before closing the helper. The parent's input descriptor remains usable
for subsequent receipt checks. No descriptor blocking flags are changed.

Five process lifecycle tests cover ordered original bytes, reuse of the parent's
writer, a closed reader, undrained-pipe deadline, cancellation and completion-timeout
reaping. These are harness checks, not a new compiler/native timing measurement.
The original archived threaded captures and their provenance remain unchanged.

Independent review added sender-failure polling before/after reads and on read
failure, so a recorded child error replaces a generic receiver timeout. Config and
result files are retained in the capture's sender directory, including failures.
Nested cleanup guarantees helper shutdown and event-file closure even if sender
cleanup raises. A forcibly killed parent is not an immediate child-death guarantee:
the child still has its absolute deadline; that case is not claimed by these tests.

A sixth lifecycle test transfers two large UTF-8/ASCII frames through many pipe
writes and fragmented reads, checking every byte and frame order. Child failure
polling does not interrupt an already-blocked helper read: its existing 15-second
read deadline can elapse before the recorded sender error is surfaced. This is
bounded eventual reporting, not an immediate child-exit wakeup.
