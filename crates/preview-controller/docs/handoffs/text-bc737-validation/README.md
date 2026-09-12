# Reconciled Text compiler validation

Exact compiler basebc737126 plus Text-only patch8b676e72 passes **111 tests**
(3 existing ignored), strict all-target Clippy, and all10 recorded persistent
protocol exchanges with complete JSON equality to the previous Text reference.
Four patched/test source hashes match the earlier apply-check artifact exactly.
This validates the combination with the owner's symbol/heading/delimiter changes.

The captured replay uses the bounded probe: request/response deadlines, correlation,
complete-frame equality and final EOF/exit checks. This is a debug compiler build;
latencies are not release or native typing-to-paint performance claims.

No authoritative compiler files, producer vendor pin or native app changed.
Compiler owner may adopt after reviewing this exact patch/base. Producer still needs
its distinct Text cache/hash/shift integration and feature-enabled acceptance;
GH43fix73217526 is separate. Pixel parity and searchable PDF acceptance are not
established by compiler protocol equality.

Evidence includes test/lint logs, ten replies, probe timings and source/binary hashes.
All build/probe jobs are terminal. See ../text-bc737-reconciliation/README.md for
patch application and actual regression target names.
