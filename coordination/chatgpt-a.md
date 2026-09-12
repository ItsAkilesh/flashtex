# chatgpt-a handoff — FT-014 rev 2

- Branch: `agent/chatgpt-a/companion-validation`.
- Owned paths: `tools/companion-validation` only. No `apps/companion` files were changed.
- State: in progress; revision 2 acknowledged.
- Current behavior: the disposable-source harness validates PBX structure, XCTest target presence and independent Debug test-target compilation, actual fixture MIME bytes, and binary-safe repair application. It now also reports recovery gaps: a missing cancellation API, a missing retry API, and capture IDs consumed before serialization (which can suppress a retry after serialization failure).
- Evidence: thirteen harness unit tests pass. The repaired candidate is `d764d54` on `agent/chatgpt-a/companion-reliability`; its unsigned Debug app and XCTest-target builds passed on Xcode 26.6. The owner branch `e7ce5b94a12e372a482a9578ac9e121b09c51c8a` remains malformed. Cross-transport validation reproduces a disconnected-path double submit. The queued interop gate now also reports the published nearby-v1 security/privacy gaps: plaintext TCP, no TLS-PSK pairing or hello proof, and missing local-network/Bonjour Info.plist declarations. This Mac has no installed iOS simulator runtime, so XCTest execution is explicitly pending and is not claimed.
- Reviewed: main `9da7e48148d91872dc0514119e1a04568ebfcb51`; FT-014 revision 2.
- Worker evidence: active local Codex Desktop session; Codex quota is not exposed. No external API calls, purchases, or Claude use.
- Next: publish this checkpoint, then give FT-004 owner/Commander the deterministic cancellation/retry findings and rerun the target build against the owner-integrated repair when available.
- Updated UTC: 2026-09-12T05:34:00Z.
