# Hybrid release policy for the helper route — measurement (2026-09-12T10:40–10:49Z)

Owner: `mac-historical-preview` (Claude Code subagent, parent `mac-claude-a`), branch
`agent/mac-historical-preview/hybrid` (base mac-shell 6f4ee94; policy 77690b2, parent diff 414e4d1).
Status: opt-in only — `FLASHTEX_CONTROLLER_RELEASE=hybrid`; default stays hold-until-preview,
historical frames stay OFF (`FLASHTEX_COMPLETED_SNAPSHOTS` unset). **All cells are
load-affected (1-minute load 32–67 during the runs; other lanes were building and testing).**
Only comparisons within this table are like-for-like; absolute numbers are not comparable to
the parent's quiet-machine runs.

## Policy under test (`apps/mac/Sources/FlashTeXMac/ControllerRelease.swift`)

`hybrid`: hold the next edit until the in-flight edit's preview arrives, **but** once that
edit is durable, newer text is waiting and it has been in flight for
`bound = clamp(factor × last observed edit→preview latency, 40 ms, 250 ms)` (factor 2 by
default; `FLASHTEX_CONTROLLER_RELEASE_FACTOR` overrides it for measurement only), send the
newest buffer anyway. A check runs on every keystroke while an edit is in flight and on a timer
armed at the durable receipt. A released edit's compile may be superseded by the helper.
`hold` = the existing adapter. `hybrid-historical` adds `FLASHTEX_COMPLETED_SNAPSHOTS=1`, so a
superseded compile that completes is painted as a labelled historical frame (HistoricalPreview.swift)
instead of being discarded; in hybrid mode the historical channel no longer releases on the durable
receipt.

## Setup

`tools/typing-bench/run.sh --producers controller --no-render --seeds "demo body60k"
--intervals "30 0" --quiet-load 30 --quiet-wait 0` (release build of the branch app; 200 typed
characters; fresh seed copy + ledger root per cell; `FLASHTEX_NO_ACTIVATE=1`). Helper
`flashtex-preview-controller` built from **origin/main 60c40c1** (crates/preview-controller at
2f2605d; sha256 `2d9bd1d399a3658b0d7aaf072bc826ad82d948250eee0255d29246974444281a`), compiler
sha256 `004283fce5ee0f534ed546a99a6b0ce3986e203fb243967025664a5b61a38fa4` (main e75741e). Apple M1
Max, macOS 26.3.1. Analysis: `analyze.py` (paints split historical/current from the app log;
`durable: r<n> for revision N` lines count durable ACKs; `release: hybrid …` lines count releases);
final source / reopen: `reopen_check.py` relaunches the helper on each cell's session project root +
ledger root, asks `document`, and compares with the seed + typed script inserted before
`\end{document}`.

## Results (ms; nearest-rank percentiles over 200 keystrokes; `hist/cur` = historical/current)

| mode | seed | interval | edits sent | durable ACKs (per key) | hybrid releases (after p50/max ms, bound range) | paints (hist/cur) | first paint p50/p95/p99 | k→current paint p50/p95/p99/max | load 1-min |
|---|---|---:|---:|---|---|---|---|---|---|
| hold | demo | 30 | 198 | 198 (0.99) | — | 193 (0/198) | 53/79/96 | 53/79/96/100 | 67→58 |
| hybrid (2×) | demo | 30 | 180 | 180 (0.90) | 1 (58/58, 56) | 181 (0/180) | 72/103/114 | 72/103/114/125 | 50→50 |
| hybrid+historical | demo | 30 | 195 | 195 (0.97) | 0 | 194 (0/195) | 56/76/83 | 56/76/83/88 | 36→34 |
| hold | demo | 0 | 154 | 154 (0.77) | — | 155 (0/154) | 56/79/86 | 56/79/86/96 | 58→54 |
| hybrid (2×) | demo | 0 | 175 | 175 (0.88) | 0 | 176 (0/175) | 59/81/97 | 59/81/97/117 | 50→47 |
| hybrid+historical | demo | 0 | 147 | 147 (0.73) | 0 | 146 (0/147) | 61/96/116 | 61/96/116/126 | 34→32 |
| hold | body60k | 30 | 61 | 61 (0.30) | — | 61 (0/61) | 173/241/274 | 173/241/274/291 | 54→48 |
| hybrid (2×) | body60k | 30 | 49 | 49 (0.24) | 1 (57/57, 40) | 50 (0/49) | 233/322/344 | 233/322/344/362 | 47→41 |
| hybrid+historical | body60k | 30 | 37 | 37 (0.18) | 4 (256/259, 40–250) | 38 (0/37) | 299/465/504 | 299/465/504/544 | 35→36 |
| hold | body60k | 0 | 20 | 20 (0.10) | — | 20 (0/20) | 235/350/391 | 235/350/391/408 | 48→45 |
| hybrid (2×) | body60k | 0 | 27 | 27 (0.14) | 1 (51/51, 40) | 28 (0/27) | 192/238/257 | 192/238/257/283 | 41→39 |
| hybrid+historical | body60k | 0 | 24 | 24 (0.12) | 3 (51/100, 40) | 24 (2/22) | 236/337/392 | 236/340/392/409 | 36→44 |

Extra data point, factor 1 (bound = the last latency itself, not part of the requested policy):

| mode | seed | interval | edits sent | durable ACKs (per key) | releases (after p50/max, bound range) | paints (hist/cur) | first paint p50/p95/p99 | k→current p50/p95/p99/max | load |
|---|---|---:|---:|---|---|---|---|---|---|
| hybrid factor 1 | demo | 30 | 193 | 193 (0.96) | 4 (46/55, 40–43) | 189 (0/190) | 61/92/124 | 61/92/124/169 | 37→34 |
| hybrid factor 1 | demo | 0 | 119 | 119 (0.59) | 13 (52/106, 40–103) | 109 (0/110) | 84/223/318 | 84/223/318/365 | 34→39 |
| hybrid factor 1 | body60k | 30 | 36 | 36 (0.18) | 16 (186/251, 40–250) | 37 (0/36) | 331/447/497 | 331/447/497/519 | 39→47 |
| hybrid factor 1 | body60k | 0 | 26 | 26 (0.13) | 18 (55/349, 40–250) | **5 (0/4)** | **780/1306/1400** | 780/1306/1400/1501 | 47→52 |
| factor 1 + historical | demo | 30 | 189 | 189 (0.94) | 10 (53/67, 40–57) | 187 (4/184) | 76/124/190 | 76/125/307/353 | 52→67 |
| factor 1 + historical | demo | 0 | 158 | 158 (0.79) | 12 (48/55, 40–50) | 154 (2/154) | 75/111/138 | 75/111/146/184 | 67→66 |
| factor 1 + historical | body60k | 30 | 63 | 63 (0.32) | 17 (98/149, 40–144) | 64 (2/61) | 182/257/289 | 182/261/314/352 | 66→57 |
| factor 1 + historical | body60k | 0 | 23 | 23 (0.12) | 11 (127/255, 40–250) | 24 (0/23) | 224/377/408 | 224/377/408/422 | 57→57 |

Full rows (compile p50, coalesced, historical lag) in `analysis.txt`.

**Final source / reopen check (`reopen-check.txt`): 20/20 cells OK** — every mode, seed and
interval: the relaunched helper returns the durable text equal to the seed with all 200 typed
characters (60 KB: 64 103 bytes sha `0d7ce0f4aae1…`; demo: 6 113 bytes sha `f18905b73ac9…`), and
the last `durable:` receipt in each log is bound to the last keystroke revision (202). Durable ACK
count per keystroke is the "edits sent" column (every edit was acknowledged; none refused).

## Reading

- **hybrid at 2× behaves like hold.** With the bound at twice the last edit→preview latency
  (saturating at 250 ms on 60 KB, where a preview takes ~170–230 ms) the release fires only
  on tail compiles: 0–4 releases per 200 keystrokes. Durable ACKs per keystroke are unchanged
  (0.18–0.30 on 60 KB, 0.9–1.0 on demo — the demo compiles faster than the typing interval so
  hold already acknowledges nearly every keystroke). Paint latencies are within the load noise
  of hold (this session's load was 32–67; the parent's quiet runs are the reference for absolutes).
  It is a safe tail guard, not a lever on the ACK rate.
- **A tighter bound starves previews.** Factor 1 releases 13–18 times per 200 keystrokes on the
  burst cells, and each release supersedes a compile that was about to finish: 60 KB at 0 ms
  dropped to 5 paints with first-paint p50 780 ms. With historical frames on, the superseded
  compiles come back as history only occasionally (2–4 historical paints; the helper keeps one
  optional slot and cancels superseded work), so it does not recover the loss.
- The durable-ACK rate under any hold-like policy is bounded by compile time; only
  release-on-durable (plain historical mode, measured in `historical-preview-2026-09-12T1010Z.md`
  and the parent's `…T1029Z`) reaches ~1 ACK per keystroke, at the cost of current-preview
  starvation. There is no setting of this policy that gives both on the 60 KB document with the
  current helper; the remaining lever is on the helper/runtime side (skip superseded queued
  compiles, or compile the newest durable text when the in-flight one is cancelled).
- Recommendation: keep the default (hold, historical OFF). `hybrid` at 2× may ship as an opt-in
  tail guard; do not lower the factor.

## Files

`hybrid-release-2026-09-12T1040Z/<mode>-<seed>-<ms>.json` (bench summaries incl. `per_keystroke`,
`load_avg_before/after`, `load_affected`), `<mode>-<seed>-<ms>.log` (`FLASHTEX_LOG` with
`keystroke:` / `compile: sending` / `durable:` / `release: hybrid` / `compile: applied [historical]` /
`paint:` / `historical:` lines), `analysis.txt`, `reopen-check.txt`, `analyze.py`, `reopen_check.py`.

## Limitations

- One run per cell under heavy load (32–67); the bench marks every cell `load_affected`; not a
  regression gate. A quiet-machine repeat is needed before any absolute claim.
- `paint:` is the CoreAnimation commit, not scan-out; the window is ordered back.
- The reopen check reads the helper's private ledger for the session project root; it does not
  test the on-disk `.tex` (the bench never saves).
