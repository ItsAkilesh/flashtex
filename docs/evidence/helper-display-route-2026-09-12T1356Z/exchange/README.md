# One exact display-candidate exchange (helper wire, unredacted)

Helper `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ae7ddcfb739815fac/crates/preview-controller/target/release/flashtex-preview-controller` (sha256 c95a26a3a80fa32a…); producer `/private/tmp/claude-501/-Users-jay3332-Projects-flashtex/e30fd4a4-f46a-4c3f-a28c-cbb8617b4425/scratchpad/render-9aaec57a/crates/render-pipeline/target/release/flashtex-render` (sha256 ed729b02befeb7d3…); app `agent/mac-helper-display/route-applied` @ `c5fa71cd`.

`helper-stdin.jsonl` = every frame the shell wrote to the helper; `helper-stdout.jsonl` = every frame the helper wrote back, byte for byte (the `display_list` sibling is inside the `update{kind:"display_candidate"}` line); `app.log` = the shell's log including the native gate's admission / validation / paint decisions; `bench.json` = the 3-keystroke bench summary. Absolute paths of this machine present in the frames: the project root and ledger under `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ae7ddcfb739815fac/apps/mac/build/helper-display-route/` and `/var/folders/...` temp dirs; nothing was removed.


## helper-stdin.jsonl

| line | bytes | type | id | kind | detail |
|---:|---:|---|---|---|---|
| 1 | 214 | configure_layout | pc-1 |  | {"layout_capabilities":["rules-v1","font-hints-v1"],"renderer_support_confirmed":true} |
| 2 | 227 | configure_display_candidates | pc-2 |  | {"capability":"display-candidates-v1","enabled":true,"renderer_support_confirmed":true} |
| 3 | 139 | document | pc-3 |  |  |
| 4 | 121 | compile | pc-4 |  |  |
| 5 | 197 | complete | pc-5 |  |  |
| 6 | 200 | complete | pc-6 |  |  |
| 7 | 199 | complete | pc-7 |  |  |
| 8 | 197 | complete | pc-8 |  |  |
| 9 | 200 | complete | pc-9 |  |  |
| 10 | 200 | complete | pc-10 |  |  |
| 11 | 364 | edit | pc-11 |  | {"expected_revision":1,"expected_sha256":"0f11ed63aff17ebeb92ed30aafee9c553311d4eb095047088d1bc4b485694e5f","path":"main.tex"} |
| 12 | 198 | complete | pc-12 |  |  |
| 13 | 201 | complete | pc-13 |  |  |
| 14 | 200 | complete | pc-14 |  |  |
| 15 | 365 | edit | pc-15 |  | {"expected_revision":2,"expected_sha256":"cfd0f11bb2c9a315250e6a095f7cd77f4a7c77f9279cd35ec7f0582ae4135ada","path":"main.tex"} |
| 16 | 198 | complete | pc-16 |  |  |
| 17 | 201 | complete | pc-17 |  |  |
| 18 | 200 | complete | pc-18 |  |  |
| 19 | 366 | edit | pc-19 |  | {"expected_revision":3,"expected_sha256":"90084984bf5cbcb215c4a7e005e02d89010e95008b599395e2110b4821b6079d","path":"main.tex"} |
| 20 | 198 | complete | pc-20 |  |  |
| 21 | 201 | complete | pc-21 |  |  |
| 22 | 200 | complete | pc-22 |  |  |

## helper-stdout.jsonl

| line | bytes | type | id | kind | detail |
|---:|---:|---|---|---|---|
| 1 | 208 | ready | None |  | {"compiler_error":null,"compiler_max_frame_bytes":8388608,"helper_max_output_bytes":16777216} |
| 2 | 1453 | update | None | preview | request_id=preview-1 compile_revision=1 source_versions={'main.tex': 1} accepted=None |
| 3 | 136 | result | pc-1 |  | submitted |
| 4 | 192 | result | pc-2 |  | capability,enabled,preview_error |
| 5 | 386 | result | pc-3 |  | document |
| 6 | 178 | update | None | stale |  |
| 7 | 1954 | update | None | preview | request_id=preview-3 compile_revision=3 source_versions={'main.tex': 1} accepted=['rules-v1', 'font-hints-v1', 'display-list-v2'] |
| 8 | 17307 | update | None | display_candidate | request_id=preview-3 compile_revision=3 source_versions={'main.tex': 1} membership_generation=2 display_list=16974 bytes untrusted=True source_actions_enabled=False |
| 9 | 136 | result | pc-4 |  | submitted |
| 10 | 1957 | update | None | preview | request_id=preview-4 compile_revision=4 source_versions={'main.tex': 1} accepted=['rules-v1', 'font-hints-v1', 'display-list-v2'] |
| 11 | 17307 | update | None | display_candidate | request_id=preview-4 compile_revision=4 source_versions={'main.tex': 1} membership_generation=2 display_list=16974 bytes untrusted=True source_actions_enabled=False |
| 12 | 169 | result | pc-5 |  | completions,source_versions |
| 13 | 169 | result | pc-6 |  | completions,source_versions |
| 14 | 596 | result | pc-7 |  | completions,source_versions |
| 15 | 169 | result | pc-8 |  | completions,source_versions |
| 16 | 169 | result | pc-9 |  | completions,source_versions |
| 17 | 597 | result | pc-10 |  | completions,source_versions |
| 18 | 440 | result | pc-11 |  | document,preview_error,save_and_submit_ms |
| 19 | 2187 | update | None | preview | request_id=preview-5 compile_revision=5 source_versions={'main.tex': 2} accepted=['rules-v1', 'font-hints-v1', 'display-list-v2'] |
| 20 | 17914 | update | None | display_candidate | request_id=preview-5 compile_revision=5 source_versions={'main.tex': 2} membership_generation=3 display_list=17581 bytes untrusted=True source_actions_enabled=False |
| 21 | 170 | result | pc-12 |  | completions,source_versions |
| 22 | 170 | result | pc-13 |  | completions,source_versions |
| 23 | 597 | result | pc-14 |  | completions,source_versions |
| 24 | 441 | result | pc-15 |  | document,preview_error,save_and_submit_ms |
| 25 | 2189 | update | None | preview | request_id=preview-6 compile_revision=6 source_versions={'main.tex': 3} accepted=['rules-v1', 'font-hints-v1', 'display-list-v2'] |
| 26 | 18271 | update | None | display_candidate | request_id=preview-6 compile_revision=6 source_versions={'main.tex': 3} membership_generation=4 display_list=17938 bytes untrusted=True source_actions_enabled=False |
| 27 | 170 | result | pc-16 |  | completions,source_versions |
| 28 | 170 | result | pc-17 |  | completions,source_versions |
| 29 | 597 | result | pc-18 |  | completions,source_versions |
| 30 | 451 | result | pc-19 |  | document,preview_error,save_and_submit_ms |
| 31 | 2180 | update | None | preview | request_id=preview-7 compile_revision=7 source_versions={'main.tex': 4} accepted=['rules-v1', 'font-hints-v1', 'display-list-v2'] |
| 32 | 18628 | update | None | display_candidate | request_id=preview-7 compile_revision=7 source_versions={'main.tex': 4} membership_generation=5 display_list=18295 bytes untrusted=True source_actions_enabled=False |
| 33 | 170 | result | pc-20 |  | completions,source_versions |
| 34 | 170 | result | pc-21 |  | completions,source_versions |
| 35 | 597 | result | pc-22 |  | completions,source_versions |

## app.log (native decisions)

```
2026-09-12T14:26:40Z	controller ready: compiler frames ≤ 8 MiB, helper output ≤ 16 MiB
2026-09-12T14:26:40Z	display-candidate: requested display-candidates-v1 (pc-2)
2026-09-12T14:26:40Z	display-candidate: helper acknowledged display-candidates-v1
2026-09-12T14:26:40Z	status: revision 2: ok, 0 diagnostics in 6 ms (durable r1)
2026-09-12T14:26:40Z	display-candidate: admitted preview-3 (generation 3, 17306 B)
2026-09-12T14:26:40Z	status: revision 2: ok, 0 diagnostics in 3 ms (durable r1)
2026-09-12T14:26:40Z	display-candidate: admitted preview-4 (generation 4, 17306 B)
2026-09-12T14:26:40Z	display-candidate: dropped preview-3 at paint: request preview-3 is not the applied v1 preview (preview-4)
2026-09-12T14:26:40Z	display-candidate: painted preview-4 as revision 2 (generation 4, durable ["main.tex": 1])
2026-09-12T14:26:40Z	status: revision 3: ok, 0 diagnostics in 16 ms (durable r2)
2026-09-12T14:26:40Z	display-candidate: admitted preview-5 (generation 5, 17913 B)
2026-09-12T14:26:40Z	display-candidate: received preview-5 generation 5 (17913 B) at 99437613845041
2026-09-12T14:26:40Z	display-candidate: validating preview-5 as revision 3 ticket 3 at 99437621083250
2026-09-12T14:26:40Z	display-candidate: validated preview-5 in 0.259541 ms, prerastered 1 page(s) in 0.305709 ms, delivered 22.498541 ms later
2026-09-12T14:26:40Z	display-candidate: published preview-5 revision 3 at 99437644249208
2026-09-12T14:26:40Z	display-candidate: painted preview-5 as revision 3 (generation 5, durable ["main.tex": 2])
2026-09-12T14:26:40Z	status: revision 4: ok, 0 diagnostics in 23 ms (durable r3)
2026-09-12T14:26:40Z	display-candidate: admitted preview-6 (generation 6, 18270 B)
2026-09-12T14:26:40Z	display-candidate: received preview-6 generation 6 (18270 B) at 99437924383708
2026-09-12T14:26:40Z	display-candidate: validating preview-6 as revision 4 ticket 4 at 99437934798541
2026-09-12T14:26:40Z	display-candidate: validated preview-6 in 0.281417 ms, prerastered 1 page(s) in 0.276375 ms, delivered 30.954417 ms later
2026-09-12T14:26:40Z	display-candidate: published preview-6 revision 4 at 99437966453625
2026-09-12T14:26:40Z	display-candidate: painted preview-6 as revision 4 (generation 6, durable ["main.tex": 3])
2026-09-12T14:26:41Z	status: revision 5: ok, 0 diagnostics in 21 ms (durable r4)
2026-09-12T14:26:41Z	display-candidate: admitted preview-7 (generation 7, 18627 B)
2026-09-12T14:26:41Z	display-candidate: received preview-7 generation 7 (18627 B) at 99438222693041
2026-09-12T14:26:41Z	display-candidate: validating preview-7 as revision 5 ticket 5 at 99438234612875
2026-09-12T14:26:41Z	display-candidate: validated preview-7 in 0.280833 ms, prerastered 1 page(s) in 0.278792 ms, delivered 28.915166 ms later
2026-09-12T14:26:41Z	display-candidate: published preview-7 revision 5 at 99438264244458
2026-09-12T14:26:41Z	display-candidate: painted preview-7 as revision 5 (generation 7, durable ["main.tex": 4])
```
