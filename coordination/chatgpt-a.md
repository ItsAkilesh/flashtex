# chatgpt-a handoff — FT-014 rev 1

- Branch: `agent/chatgpt-a/companion-validation`.
- Owned paths: `tools/companion-validation` only. No `apps/companion` files were changed.
- State: ready for integration.
- Ready behavior: `check_companion.py` exports pinned Git revisions to disposable directories, records exact Xcode commands/exit codes, detects malformed PBX object definitions in lists, detects no XCTest target, checks the PNG/JPEG serializer mismatch, and writes a binary-safe project repair diff plus JSON evidence.
- Evidence: six harness unit tests pass. The report records Xcode 26.6 build 17F113, the installed SDK inventory, and `xcrun simctl list runtimes` (no simulator runtime installed). A full run against `origin/agent/aarush-macbook/companion-capture` reports `xcodebuild -list` exit 74, eight malformed-PBX findings, no XCTest target, and the MIME mismatch. It validates repair candidate `origin/agent/chatgpt-a/companion-project-repair` with exit 0 for Xcode project loading, destination discovery, and an unsigned direct `iphonesimulator` SDK build; emitted patch size is 4442 bytes.
- Limits: this Mac has no installed simulator runtime, so no simulator launch or XCTest execution is claimed. The missing test target and MIME payload fix remain outstanding FT-004 work.
- Reviewed: main `342e1e029f80e7a5b472ad202ae87d597274127b`; FT-014 remains revision 1 and no interface change is needed.
- Worker evidence: active local Codex Desktop session; validation shell PID 14777 at the evidence checkpoint. Codex quota is not exposed. No external API calls, purchases, or Claude use.
- Next: Commander or the FT-004 owner reviews/integrates the isolated harness and repair candidate. Exact rerun command is in `tools/companion-validation/README.md`.
- Updated UTC: 2026-09-12T04:55:37Z.
