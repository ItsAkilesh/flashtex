# Commander recovery checkpoint

Updated 2026-09-12T11:06:33.760495+00:00, baseline main 2d68926cf1cb7c311b633ff05724eeee2d278ac1. Historical detail remains in Git and committed evidence; this file records current handoff state.

## Witness disk-quota recovery

At11:19:46Z the old witness PID752107 terminated with OSError122 while writing its local observation; transient unit was collected. Dispatcher1099831 and exact Commander268514 remained live, no witness publication journal existed, and authority was unchanged. After scoped worker-owned cache cleanup, a one-shot observation succeeded and correctly blocked takeover. Restored witness is active PID1405586, fresh11:21:45Z observation, no journal blockers and claim_authorized=false. No dispatcher or Commander restart occurred. The write-error resilience fix and15 tests are committed here, but the already-running restored interpreter predates that fix; it takes effect on a future legitimate launch, no needless live-service restart. Remote quota-terminal hook remains unverified.

## Latest verified checkpoint (supersedes older pending lists below)

Main e4c9252d integrates runtime b9240b0 and helper ccd474f4, including c47139ff observable-condition sibling test repair. Original parallel short-wall-deadline failures are retained; repaired default-parallel runtime, serial runtime, full real-compiler helper suite and strict lints pass. Full-size optional-output refusal/stall recovery passed. Main0aac2b7f integrates font5c89501 and rendering4ca51d94 with combined font/render suites+lints, actual pinned PFB and Python gates. Helper candidate adapter accepts real producer/runtime/helper fixtures without metadata repair; native mode stays OFF.

Current formal assignments: FT023r10 subsetter consumer (unchanged mac-pdf20e5277, existing five fixtures), FT049r4 independent incremental producer6e69661 acceptance, FT048 helper measurements/diagnostics. Same four handles running; drained three and Sol remain completed. Renderer/runtime correctness builds active in their own worktrees; no Commander build sessions remain. Pending evidence-only runtime9e4117fb lifecycle and root34742219 scaling/decline harness; review/merge next. New additive runtime display-stage profiling approved after correctness checkpoint, no helper-owned runtime edits.

Latest remote structured Jaysen10:55:23 reports9children+parent, target15; parent tip7ccbd7e9 at11:06. Danielb40e994 unchanged, no fresh live census. Issue2comment5645542186 requests existing queue pickup/census and announces tested main; previous5645530349 routes native helper/source-plan/history acceptance. Services remain active PIDs1099831/752107; do not restart absent terminal proof. No active global publication uncertainty.

## Authority and staffing

Sole Commander is `/root/runtime_validator`, orchestrator-astra. `/root` is a product engineer/user-facing relay, not a second global writer. Exactly four local slots: Commander, root helper/ledger engineer, existing `/root/compiler_corpus` handle NOW runtime-display engineer, and `/root/supervisor_api_review` renderer. No new agent was spawned for the role transfer.

The former font role completed5c89501/report55bfd48, preserves its clean font branch, and holds flex. FT024 was cancelled to suspend that queue, not to erase work. `commander-font-resources` is paused as an OLD ID; `/root/compiler_corpus` itself remains active as `commander-runtime-display` on FT049. The three user-drained handles bridge_context, corpus_continuation and mac_integration_review remain completed with no followups. Sol remains quiesced.

Current authoritative task records:
- FT-023 r8 assigned: commander-render-core; crates/rendering-core.
- FT-024 r17 cancelled: commander-font-resources; crates/font-resources.
- FT-048 r6 assigned: commander-preview-performance; crates/preview-controller, crates/edit-ledger.
- FT-049 r3 assigned: commander-runtime-display; crates/document-runtime.

FT049 ownership fence maineed1847 followed root's explicit runtime quiescence afterecbaf7c. New worktree `/home/natkarri/flashtex-runtime-display`, branch `agent/commander-runtime-display/display-runtime`, actual registration/ACK f3c7ee3; first implementation f58b656. Root retains preview-controller/edit-ledger only. No runtime edits by root after release.

## Publication and revival

All main/control publication MUST hold `dispatch_loop.publication_lock(root)` over common Git `flashtex/main-publication.lock`, fetch/reread active authority, merge current main and nonforce push. Dispatcher active PID1099831 and witness active PID752107 were independently checked this session; last inspected dispatcher journal state published. Recheck actual processes/journal before recovery; do not replay an uncertain push or paid call. `coord.py dispatch` still hardcodes legacy `commander` branch and rejects this authorized branch; same-owner/path records were updated directly under authority and common lock, not by changing identity or bypassing remote approval.

Revival prompt remains AGENTS.md → docs/commander-failover.md. Require exact host/session termination or explicit quiesced handoff plus stopped dispatch/publication/integration jobs and reconciled journals. Silence, heartbeat age, timeout, quota error or network failure is not proof offline. This host remains preferred while usable; successor needs fresh comparable verified authorized resources. Actual hosted quota-to-terminal hook and remote end-to-end claim remain unverified. No shadow Commander.

## Main integration checkpoints

- 4248b49: corrected c158f4e plain original/reference evidence; zero diagnostics and exact text/144DPI pixels for ONE fixture. Prior211pixel result explicitly fallback/error evidence.
- 70a5f30: root397bff7 full-frame GH29 fix (opt-in15MiB/default8MiB, independent16MiB output guard) plus exact-schema shared immutable history. Prior timeout evidence preserved.
- 3ac69c3: bounded encoded history cache2f2605d, combined ledger/helper and explicit original-compiler gates/lint pass; no native or RSS guarantee.
- d25e6f9: font79bdada passive Type1/required metrics chain; combined font/render tests/lints and pinned official Type1/TFM replays pass. Strict default/context/refusal gates remain; TTC repair is only an owner handoff.
- 60c40c1: pinned replay runnera45e71b. Current source drift returns4/unknown before build, intentionally not current-main acceptance.
- 7802c9a includes91296cb/351ce8f: math8b06436 +44b7779 classification. Inline math602pixels, matching linear extraction;582math-window/20colon differences, reference optical fonts differ, checked origins/rules<0.001bp apart. No proven consumer geometry bug.
- 8a44552: decoder ecbaf7c, raw4 +single decoding/validation permit, joined shutdown. Combined runtime/helper real-compiler gates and both lints pass.
- 796b982c includes4e15783: helper source plans through6e515be, explicit bibliography lifecycle, source-bound history metadata, separate grouped application; full helper/compiler checks and lint pass. Native app must persist bibliography declarations; no kind inference or project-wide atomicity claim.

## Outstanding tested products and current work

Commander integration worktree `/home/natkarri/flashtex-drained-integration`, branch `agent/orchestrator-astra/drained-products`, was clean after796b982c; no active Commander build/publication handles at this checkpoint.

Pending font chain through5c89501/55bfd48: exact-rational Type1 raw and verified-context matrix adapters. Actual LM conditional prologue remains refused; no PS execution, renderer activation or guessed defaults. Preserve old font worktree `/home/natkarri/flashtex-font-resources`.

Pending renderer:83d4a0e display math (1244pixels; reference code88 missing ToUnicode means extracted X is not a corrective target for original∑),7ac845f three-fixture runner,ef350d2 wrapping (13line memberships/210words/text match,979pixels, no consumer-added shift). Current same renderer investigating existing18-ligatures, then aggregate actual fixture status; no broad sweep or invented parity.

Pending runtime FT049:f58b656/f3c7ee3 bounded default-OFF contiguous source-bound UntrustedDisplayCandidate, stale/cancel/toggle and historical mutual exclusion, default legacy unchanged. Current actual published producer replay/helper feedback stage. Reuse existing native/core rendering validation; do not duplicate parser or call candidate render-ready.

Pending root helper:e764d2a forwarding +unchangedFT049 dependency854b041, e5a894b deterministic stale/corrupt-sibling recovery; full59helper checks reported plus focused extra tests/lint. Current optional oversize/stall gates. Native helper-v2 acceptance remains unverified/defaultOFF. Snapshot kinds/source-plan/native history metadata already integrated through6e515be.

Compiler ce5bf48 isolated port remains UNCOMMITTED/BLOCKED in `/home/natkarri/flashtex-compiler-delimiters`, branch `agent/orchestrator-astra/compiler-delimiters`. Six supplied tests pass; independent unknown command after left silently disappears with statusok/diagnostics empty. Exact failure `/tmp/flashtex-delimiter-unknown.log`, owner request issue1comment5645309209. Do not merge or discard pending repro. Never import peer trailing-page truncation d5c6118. Safe serializer e75741e and font literal portdc3f7d2 already main.

## Remote census and requests

Latest Jaysen explicit census is issue2comment5645364728 at10:37:03Z: header11 corrected to12 listed running child IDs +parent, target15. Earlier09:51 count16 is historical. Three concrete remaining-slot proposals search/history/preferences sent5645383723, require overlap audit and actual launch ACK; no claim started. Native2f2605d60KB hold baseline160/197/208ms, all current paints; historical mode current2912/5877/6153ms and remainsOFF. Direct v2 demo200keystrokes89/116/134ms and six0pixel-vs-CG frames are a separate direct-route measurement, not helper or LaTeX-reference parity.

Daniel latestb40e99409:55:54:15round2 complete, FT030 local tool-permission launch block, others waiting for integration. FT046r4 standing supervision/live16 census not confirmed; do not bypass permission layer. Kabirce5bf4809:54:34r12 complete/watcherarmedr13, no new active session evidence. Aarush/companions stale. Fresh pickup/census request5645300305; no response from these leads yet. Assigned/completed is not running; plan multiplier is not remaining quota. No purchases/overages/unknown paid fallback.

## Next Commander actions

Keep retained slots on their existing queues, check exact ACKs after dispatcher revision changes, integrate tested pending chains with relevant combined checks, and route actual helper/native handoffs through existing authorized issue2 parent. Preserve remote targets without duplicate work or credential bypass. Fix/record actual operational blockers, never assert all-machine concurrency without fresh execution evidence. Keep source/reference/raw-PDF/text/raster/native-paint gates separate. Context policy uses actual telemetry if exposed; none is currently available, so no invented percentages or forced restart.
