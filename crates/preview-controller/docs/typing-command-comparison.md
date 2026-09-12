# Actual 50KB typing command comparison

`benchmarks/typing-command-50kb` records one sequential pair using the same pinned
actual producer, verified10pt assets, helper binary and initial50KB source. A
visible first-letter edit follows a multibyte UTF8 comment, so its byte offset is
not its character offset. Both modes produce byte-identical final source and the
same parsed actual current preview. Restart/reopen confirms durable revision2 and
exact text/hash. The grouped permanent-ID retry is idempotent.

| Mode | Request bytes | ACK bytes | ACK arrival | Preview arrival |
| --- | ---: | ---: | ---: | ---: |
| Full-source edit, full reply | 50,231 | 50,303 | 11.25ms | 364.37ms |
| Grouped edit, metadata reply | 381 | 424 | 10.27ms | 347.84ms |

Observed save/submit was9.57ms versus10.05ms; response serialization was1.19ms
versus0.029ms. The large wire-size reduction is demonstrated. This single pair on
debug binaries was not isolated from other engineering activity and establishes
no calibrated speed advantage or native paint result. Arrival timestamps include
producer/helper/pipe work and Python line buffering, before decoding that frame.

The preview observations exceed200ms in this setup. That is a concrete profiling
case, not a production release regression or a universal latency bound. Neither
reply-size savings nor quick ACKs establish imperceptible preview updates. Further
phase attribution should locate compiler, serialization and transfer costs before
changing scheduling or dropping output.

Raw request/ACK/event artifacts, final preview values, initial/edited source and
phase diagnostics are preserved as deterministic gzip files. Provenance records
both original and compressed hashes. The executable example is
`examples/typing_command_compare.py`; no mock compiler/provider or native app is
involved, and no source approval or wire protocol was changed.
