# Preview helper owned-wire integration

Exact7956d37 removes two redundant deep copies when embedding a large validated
result and its payload into JSON envelopes. It moves owned Values; protocol,
queue, durability, final serialization and response identities are unchanged.

Independent combined-main validation:35 tests including configured original
compiler, large helper output, stale response/backpressure and durable recovery;
strict all-target Clippy passes. Source review confirms only metadata uses json!
while large result/payload values move into their final objects.

Published paired benchmark compares identical10.1MBresult:20rounds,40exact wire
comparisons, wrapper median221.452ms before versus0.006284ms after; final encoding
27.426ms versus27.751ms. This isolates wrapper work and is not a native latency
claim. Existing Mac validation owner must remeasure actual helper typing/presentation.

## Followup: exact frame bound and lost-response recovery

Merged4db4a7a:16MiB now includes the JSONL delimiter; requested Vec growth is capped
and serialization overflow starts a fresh small error buffer. This fixes the prior
16MiB+1 wire edge and retained large capacity on tiny error responses.38 tests,
including configured compiler/large-output/recovery cases, and strict Clippy pass.

Published real-helper sequential comparison (same dependencies/compiler/input):
30/30 clean exact samples,50KB median141.457→94.910ms,5KB17.851→12.566ms. Lost-response
fixtures send an edit, observe unread output, kill without reading acknowledgement,
and reopen to exact new source/hash/revision; stale original request is refused.
These are sequential subprocess measurements, not native paint or burst guarantees.
