# Large paste, multi-range group and kill recovery on the durable controller route — 2026-09-12

Lane `mac-paste-recovery` (Claude Code subagent of `mac-claude-a`, mac-m1max-a, M1 Max,
Xcode 26.3 / Swift 6.2.4). Tests: `apps/mac/Tests/FlashTeXMacTests/PasteRecoveryTests.swift`
at branch `agent/mac-paste-recovery/paste` (cut from `agent/mac-claude-a/mac-shell` 5bc3fc0f,
which carries main dda0b62's preview-controller with compact edit acknowledgements).

Helpers: real `flashtex-preview-controller` and `flashtex-compiler` built in this worktree at
5bc3fc0f (`cargo build --release`). No crate, helper or `apps/mac/Sources` file was changed.

Command (narrow filter, as the parent's heavy-build notice requires):

```
FLASHTEX_COMPILER=crates/compiler/target/release/flashtex-compiler \
FLASHTEX_PREVIEW_CONTROLLER=crates/preview-controller/target/release/flashtex-preview-controller \
FLASHTEX_PASTE_RECOVERY_LOAD_LIMIT=200 \
swift test --package-path apps/mac --filter PasteRecoveryTests
```

`FLASHTEX_PASTE_RECOVERY_LOAD_LIMIT` only raises the tests' load-skip threshold (default:
skip when the 1-minute load exceeds 20); every assertion below is about revisions, hashes
and bytes, not timing. The ms figures were measured **under shared load, not an isolated
result**: twelve sibling lanes were compiling (5-minute load 33–39 during both runs).

## Fixtures (deterministic; SHA-256 of the UTF-8 bytes)

| Text | Bytes | SHA-256 |
| --- | --- | --- |
| base document (r1) | 61 531 | `0ae0093653a8278154c1424e35e80292045e3c1d76a3d2ec62d9146d93a34b3d` |
| pasted block (inserted before `\end{document}`) | 512 139 | — |
| base + paste (r2 in a, c, d) | 573 670 | `905d33e251e22686e8f7cb71508d1029819401671096f38ab8740b49f8a8a70e` |
| paste + typed line (r3 in d) | 573 708 | `18540331c0db324b124a5b633a3f401e887ff52091ae53f3793c33b17859f84d` |
| base with 3 ranges replaced (r2 in b): bytes 1111–1124 → `Section 10 —`, 21731–21745 → `Section 200 — naïve`, 61506–61514 → `lazy cät` | 61 541 | `e51fbad01493c8ace9b56e9a51a207ba7ca53cadeb64f66bcc89ae0456f19cca` |

The paste block is one prose paragraph per eight commented lines. A fully-prose 560 KB
document was measured to produce 18.2 MB of compiler preview JSON (above the helper's
16 MiB output limit; `crates/preview-controller/src/main.rs` `MAX_OUTPUT_BYTES`) — that limit is
a different surface from the durable source path this lane covers, and is reported, not patched.

## Run 2 (final, 5/5 passed) — 13:39Z

`uptime` at start: `9:39 up 1 day, 8:31, 2 users, load averages: 12.87 35.35 26.60`;
at end: `load averages: 10.29 33.30 26.07`. `Executed 5 tests, with 0 failures in 19.655 s`.

| Test | Result | Measured (ms from the edit call; under shared load) |
| --- | --- | --- |
| (a) `testLargePasteIsExactlyOneDurableEditAndThePreviewBindsToIt` | passed 3.63 s | paste 512 139 B into 61 531 B → durable r2 `905d33e251e2…`; ACK 127.0 ms; preview bound to the paste's editor revision 2 220.7 ms; `history_status.undo_labels == ["Source edit"]` |
| (b) `testMultiRangeReplacementIsOneGroupWithExactText` | passed 2.65 s | 3-range `apply_group` on 61 531 B → r2 `e51fbad01493…`, `replayed_command:false`, `command_revision:2`; ACK 22.6 ms; buffer adopted the exact text via `controllerAdoptHistoryResult`, nothing resubmitted (still r2), preview bound 1 247.6 ms; `undo_labels == ["Replace 3 ranges"]` |
| (c-i) `testHelperKilledAfterThePasteACKRecoversTheACKedRevision` | passed 3.85 s | SIGKILL pid 88262 at 111.9 ms with the shell holding r2 and the preview not yet arrived; `.exited` observed, buffer untouched; reopen pid 88271; recovered r2 `905d33e251e2…` at 2 444.4 ms; resubmitted = no; one history step |
| (c-ii) `testHelperKilledBeforeThePasteACKResubmitsExactlyOnce` | passed 3.72 s | SIGKILL pid 88304 at 4.6 ms with the shell holding r1 (no ACK); reopen pid 88598: ledger held r1, the shell logged "differs from the buffer … submitting the buffer" once; recovered r2 `905d33e251e2…` at 2 287.2 ms; resubmitted = yes; one history step |
| (d) `testEditTypedBeforeThePastePreviewIsHeldAndBothEndDurableInOrder` | passed 5.81 s | paste r2 ACK 112.7 ms; a line typed before the paste's preview: `queued == true`, in-flight still the paste, durable still r2; released after the paste's preview, r3 `18540331c0db…` ACK 2 282.8 ms, final preview 4 359.5 ms; `textByDurable[2]` == paste, `[3]` == paste+typed; `undo_labels == ["Source edit","Source edit"]` |

## Run 1 — 13:38Z (4/5; the one failure was a test bug, fixed)

`uptime`: `load averages: 21.91 39.91 27.71` at start, `15.79 37.03 27.03` at end. (a) ACK
120.2 ms / preview 2 227.1 ms; (b) ACK 32.0 ms / preview 1 488.3 ms; (c-i) kill at 117.8 ms
(shell had r2, preview not yet), recovered r2 at 2 445.0 ms, no resubmission; (c-ii) kill at
4.7 ms (shell had r1), recovered r2 at 2 273.1 ms, resubmitted once; (d) r3 ACK 2 472.5 ms,
final preview 4 574.6 ms. (b) failed only on `XCTAssertNil(result["preview_error"])`: the
helper sends JSON `null` (NSNull) when the compiler ran; the assertion now checks the string.
All revision/hash/byte assertions passed in both runs.

## What the kill outcomes prove

The exact-r2 assertion is the recovery invariant: a duplicate application after reopen would
be r3, a lost paste would leave r1. Both kill points landed on the intended side (after / before
the ACK) in both runs. The controller route has no automatic relaunch (the shell clears its
state on `.exited`; `ShellModel+Controller.handleController`), so the tests re-attach the
helper explicitly, which is what the user's reopen does; the re-attach reads the ledger's
`document`, and the shell's attach policy resubmits a differing buffer once (c-ii) or keeps
the ledger's revision when it already equals the buffer (c-i).

## Not measured / limitations

- Not an isolated latency result (see load above); no full `swift test` sweep was run in
  this lane (the parent runs the suite at integration under load < 15).
- The kill-before-ACK point is a race by construction (the 573 KB frame is fully written
  when `updateActiveText` returns; the helper may apply it before the signal lands). The test
  accepts either ledger state and reports which occurred; both runs hit "ledger held r1".
- `apply_group` replay with a fixed command id is covered by the search lane's
  `ProjectSearchTests.testRetryingAnUncertainCommandReplaysExactly` (branch
  `agent/mac-search/panel`), not repeated here.
