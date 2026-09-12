# Companion validation harness

This tool validates a pinned FT-004 source revision in disposable `git archive`
exports. It never writes to `apps/companion` in its own checkout.

Run it from the repository root:

```sh
python3 tools/companion-validation/check_companion.py \
  --bad-ref origin/agent/aarush-macbook/companion-capture \
  --repair-ref origin/agent/chatgpt-a/companion-project-repair \
  --output /tmp/flashtex-companion-evidence
```

The output directory receives `report.json` and the binary-safe Git diff.
`report.json` resolves and records the immutable source and repair commit SHAs, so
branch movement cannot change what the evidence describes. The output also receives
`companion-project-repair.patch`. The report records the exact command and exit
code for Xcode project loading and the optional unsigned simulator-SDK build. It
also detects PBX object definitions that were inserted into reference lists, the
absence of an XCTest target, the current JPEG/PNG serialization mismatch, and
whether the runtime capture fixture's base64 bytes match its declared MIME type.
It also builds the `FlashTeXCompanionTests` target independently (when builds are
enabled), and reports conservative source-level recovery gates for cancellation,
retry, and capture-ID consumption before serialization. A missing simulator
runtime is recorded as a runtime-test limitation rather than being misreported as
an XCTest pass.
The report also checks that the stdout transport has an atomic capture-ID registry
and explicitly rejects duplicates.
It separately detects the cross-transport failure mode where `CaptureTransport`
prints a capture and disconnected `BonjourTransport` prints the same JSON again.

Run the harness unit tests with:

```sh
python3 -m unittest tools/companion-validation/test_check_companion.py -v
```
