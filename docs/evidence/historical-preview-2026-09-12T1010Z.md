# Historical previews (`completed_snapshot`) — native acceptance measurement (2026-09-12T10:10–10:13Z)

Owner: `mac-historical-preview` (Claude Code subagent, parent `mac-claude-a`), branch
`agent/mac-historical-preview/consumer` (app built at 6bbe6a1 + this evidence).
Status: measured once per cell on a loaded machine; the feature stays **default OFF**.

## What was measured

The same typing bench as `typing-bench-2026-09-12T094540Z.md` (`tools/typing-bench/run.sh`,
release build, `FLASHTEX_NO_ACTIVATE=1`, 200 typed characters, `FLASHTEX_LOG` per run),
through the durable helper route (`FLASHTEX_PREVIEW_CONTROLLER`), in two modes:

- **baseline** — the existing hold-until-preview adapter (`FLASHTEX_COMPLETED_SNAPSHOTS`
  unset): the next edit waits for the in-flight edit's preview; newest buffer coalesced.
- **historical** — `FLASHTEX_COMPLETED_SNAPSHOTS=1`: the app negotiates
  `completed-snapshots-v1`, every `edit` carries a `source_binding_token`, the adapter
  releases the next edit on the durable receipt (one durable ACK per keystroke), and the
  helper's `completed_snapshot` frames are painted labelled/gated (`HistoricalPreview.swift`).

Binaries: helper `flashtex-preview-controller` built from
`origin/agent/commander-preview-performance/preview-performance` @ **ab945e6**
(scratch `git archive` of `crates/`, `cargo build --release`; sha256
`58daf00b00fc194715b0aed87e8691970315e6fd5404ed5d0abe1098b6e7614b`); compiler
`flashtex-compiler` sha256 `004283fce5ee0f534ed546a99a6b0ce3986e203fb243967025664a5b61a38fa4`
(the main-e75741e build the parent measured with at 09:45Z). Apple M1 Max, macOS 26.3.1;
**load average 7–10 (1 min) / 13–17 (5 min)** during the runs — other agents were building
and testing on this machine, so absolute numbers are higher than the parent's 09:45Z run
(load 8–12) and only the baseline-vs-historical comparison within this table is like-for-like.

Classification (`analysis.txt`, produced by `analyze.py` in the same directory): a paint is *historical* when the
log shows `compile: applied historical revision N` for the revision the bench recorded as
`painted_by_revision`, else *current*. "historical lag" = keystroke → first paint for keystrokes
first shown by a historical frame; "k→current paint" = keystroke → the first paint of a
*current* preview covering it (from the `paint:` lines), for every keystroke.

## Results (ms; nearest-rank percentiles over 200 keystrokes)

| mode | seed | interval | paints (hist/cur) | keys first shown by hist/cur | first paint p50/p95/p99 | historical lag p50/p95/p99 | k→current paint p50/p95/p99/max | compile p50 | coalesced |
|---|---|---:|---|---|---|---|---|---:|---:|
| baseline | demo 6 KB | 30 ms | 162 (0/161) | 0/200 | 65/92/118 | — | 65/92/118/119 | 29 | 39 |
| historical | demo 6 KB | 30 ms | 178 (87/101) | 82/118 | 65/112/124 | 93/119/146 | 66/**1629**/1868/1918 | 22 | 23 |
| baseline | body60k 64 KB | 30 ms | 52 (0/52) | 0/200 | 195/272/296 | — | 195/272/296/307 | 72 | 148 |
| historical | body60k 64 KB | 30 ms | 76 (65/10) | 180/20 | 309/410/425 | 316/414/441 | **2822/5582/5809/5874** | 47 | 125 |
| baseline | demo 6 KB | 0 ms burst | 93 (0/93) | 0/200 | 69/109/134 | — | 69/109/134/141 | 25 | 107 |
| historical | demo 6 KB | 0 ms burst | 93 (35/66) | 64/136 | 71/109/116 | 92/114/129 | 72/326/387/404 | 19 | 107 |

Historical frames refused/dropped by the native floor or identity checks in these runs: 0
(the helper's single optional slot already evicts superseded history; every frame that
arrived was newer than the displayed generation). Every historical paint carried the label
`revision N shown — revision M compiling`; the `HISTORICAL` badge replaced `WORKER`;
navigation, diagnostic jump, caret sync, pin-insertion-point and export were refused while
the flag was set (`HistoricalPreviewTests`, real-helper burst test).

## Reading

- The channel works end to end with the published helper: negotiation acknowledged, tokens
  echoed verbatim, frames validated (session, project, token, `is_current:false`,
  `source_actions_enabled:false`, generation older than current), painted monotonically.
- As a *typing-progress* mechanism it does not beat the hold-until-preview adapter under this
  helper/runtime: during continuous typing at 30 ms every completion is historical (the
  compile of revision N finishes after revision N+1 was already submitted), so the **current**
  preview appears only when typing pauses — k→current p95 1.6 s on demo, 5.6 s on 60 KB —
  whereas baseline paints current previews at p50 65 / 195 ms. On demo the historical frames
  show progress at a lag comparable to baseline's current paints (p50 93 vs 65 ms); on 60 KB
  they are *slower* (316 vs 195 ms p50) because one durable edit + one compile per keystroke
  queues work the runtime does not skip.
- What the mode buys is one durable ledger revision per keystroke plus visible progress while
  the current build is behind; what it costs is source actions disabled during typing and a
  starved current preview. Keep it OFF by default (as required); a hybrid policy (hold until
  preview, but release on durable once the in-flight compile exceeds a bound) and runtime-side
  skipping of superseded queued compiles are the follow-ups that would make it worthwhile.

## Files

`historical-preview-2026-09-12T1010Z/<mode>-<seed>-<ms>.json` (bench summaries with
`per_keystroke`), `<mode>-<seed>-<ms>.log` (`FLASHTEX_LOG`: `keystroke:` / `compile: sending` /
`compile: applied [historical] revision` / `paint:` / `historical:` lines), `analysis.txt`.

## Limitations

- One run per cell on a loaded machine (load 7–17); no warm-up discard; not a regression gate.
- The bench's own `k→p` counts a historical paint as a paint of revision N (it is: N was
  shown); the split above is post-processed from the log, not a bench feature.
- `paint:` is the CoreAnimation commit, not scan-out; the window is ordered back.
- The helper's `completed_snapshot` path had no refusals to exercise here; the refusal paths
  are covered by the fake-helper tests, not by this measurement.
