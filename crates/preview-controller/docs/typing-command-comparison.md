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

## Release helper, unchanged debug producer

`benchmarks/typing-command-50kb-release-helper` records an optimized helper build
from58d27314, preserving the old debug helper and earlier release binary under
`/tmp/ft048-release-profile-binaries`. The unchanged debug producer and verified
asset hashes are pinned. The visible source edit, actual final preview and helper
request bytes match the original debug artifact exactly.

A transparent capture proxy now records actual producer request/output frames.
Full/group edited producer request frames match byte-for-byte. The proxy changes
the observation setup and contributes unseparated overhead. Its original frames
are supplied to the runtime engineer for independent release-producer comparison.
All observed producer PIDs were absent after completion.

| Mode | ACK arrival | Save/submit | Runtime interval | Controller interval | Preview frame arrival |
| --- | ---: | ---: | ---: | ---: | ---: |
| Full | 0.80ms | 0.49ms | 239.51ms | 240.01ms | 244.39ms |
| Group | 0.68ms | 0.47ms | 267.24ms | 267.72ms | 273.45ms |

About4.39/5.73ms remained between the controller interval and observed frame
arrival. This residual includes helper emission and transport/test-reader work;
it is not an independently measured serialization phase. The runtime interval
includes producer execution, capture forwarding and runtime validation; it does
not isolate compiler execution. This is one sequential pair, not a calibrated
before/after benchmark or native paint result. Request sizes remain50,231/381bytes;
ACK sizes50,302/434bytes vary slightly with numeric timing formatting.

The post-measurement grouped retry proves durable-command idempotency only. It
still requests compilation and its later diagnostics are outside the measured
ACK/current-preview pair. No claim of zero extra compilation or later-revision
retry behavior follows from this particular experiment.
