# chatgpt-a handoff — FT-014 rev 1

- Branch: `agent/chatgpt-a/companion-validation`.
- Owned paths: `tools/companion-validation` only. No `apps/companion` files were changed.
- State: ready for integration.
- Ready behavior: `check_companion.py` exports pinned Git revisions to disposable directories, records exact Xcode commands/exit codes, detects malformed PBX object definitions in lists, detects no XCTest target, checks the PNG/JPEG serializer mismatch, and writes a binary-safe project repair diff plus JSON evidence.
- Evidence: six harness unit tests pass. A full run against `origin/agent/aarush-macbook/companion-capture` reports `xcodebuild -list` exit 74, eight malformed-PBX findings, no XCTest target, and the MIME mismatch. It validates repair candidate `origin/agent/chatgpt-a/companion-project-repair` with `xcodebuild -list` exit 0 and unsigned simulator-SDK build exit 0; emitted patch size is 4442 bytes.
- Limits: this Mac has no installed simulator runtime, so no simulator launch or XCTest execution is claimed. The missing test target and MIME payload fix remain outstanding FT-004 work.
- Reviewed: main `342e1e029f80e7a5b472ad202ae87d597274127b`; FT-014 remains revision 1 and no interface change is needed.
- Resource state: no external API calls, purchases, or Claude use; Codex quota is not exposed.
- Next: Commander or the FT-004 owner reviews/integrates the isolated harness and repair candidate. Exact rerun command is in `tools/companion-validation/README.md`.
- Updated UTC: 2026-09-12T04:55:37Z.
