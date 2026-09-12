# Bounded compiler protocol probe

`bounded_protocol_probe.py` compares every complete JSON response with recorded
expected output while measuring request-write through complete-response latency.
Unlike the historical text-scaling capture script, partial stdout and blocked stdin
share one deadline. Oversized replies fail before unbounded accumulation; failure
terminates and reaps only the child this probe started. Stderr goes to a file rather
than an undrained pipe. This is a local single-reply-per-request compiler probe,
not a producer sibling-frame or native rendering harness.

Required options: `--binary`, `--requests`, `--expected`, `--output`.
Optional: `--timeout` seconds (default30), `--max-reply` bytes (default16MiB).
Use a new output directory to retain earlier evidence. Partial captured replies
and stderr remain on failure; result.json is written only after successful checks.
Historical benchmark files/results are unchanged and used their archived script.

Validation: four real-child tests cover a partial response stall, a worker that
never reads its request, oversized partial output, and two valid replies. Every
case verifies child termination. The actual optimized Text compiler also passes
all ten recorded text-session request/response comparisons through this probe.

Run: `python3 -m unittest discover -s crates/preview-controller/tools -p test_bounded_protocol_probe.py -v`.
