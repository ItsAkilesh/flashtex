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
