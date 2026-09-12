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

The output directory receives `report.json` and the binary-safe Git diff
`companion-project-repair.patch`. The report records the exact command and exit
code for Xcode project loading and the optional unsigned simulator-SDK build. It
also detects PBX object definitions that were inserted into reference lists, the
absence of an XCTest target, the current JPEG/PNG serialization mismatch, and
whether the runtime capture fixture's base64 bytes match its declared MIME type.

Run the harness unit tests with:

```sh
python3 -m unittest tools/companion-validation/test_check_companion.py -v
```
