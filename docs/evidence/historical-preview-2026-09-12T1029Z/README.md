# Historical mode vs hold-until-preview — helper 2f2605d (main 3ac69c3), 2026-09-12T10:29Z

Same bench as `typing-bench-2026-09-12T102913Z.md` (baseline, `FLASHTEX_COMPLETED_SNAPSHOTS`
unset) and `typing-bench-2026-09-12T102931Z.md` (`FLASHTEX_COMPLETED_SNAPSHOTS=1`), app
mac-shell 04a4eaa release, compiler e75741e, helper built from main 3ac69c3 (crates/
preview-controller at 2f2605d, sha256 96805c939000d702…), load average 9–11, 30 ms typing.
Analysis with `../historical-preview-2026-09-12T1010Z/analyze.py` on these logs.

| mode | seed | paints hist/cur | keys first shown by hist/cur | first paint p50/p95/p99 ms | historical lag p50/p95/p99 | keystroke→CURRENT paint p50/p95/p99/max |
|---|---|---|---|---|---|---|
| baseline | demo | 0/200 | 0/200 | 47.5/62.5/68.4 | — | 47.5/62.5/68.4/83.6 |
| baseline | 60 KB | 0/65 | 0/200 | 160/197/208 | — | 160/197/208/222 |
| historical | demo | 1/198 | 1/199 | 48/65/69 | 49.9 (n=1) | 48/65/69/87 |
| historical | 60 KB | 120/12 | 181/19 | 195/227/238 | 198/227/239 | **2912/5877/6153/6211** |

Reading: with the encoding cache the helper keeps up with durable ACKs and historical frames
on 60 KB arrive at ~200 ms lag (comparable to the baseline's current paints), but the CURRENT
preview still starves under continuous typing (12 current paints in 200 keystrokes, 2.9 s p50).
Historical mode stays OFF by default. Source/reopen checks: every historical frame carried the
originating source_versions and the exact token; 0 frames refused or dropped by the native
checks; final current preview bound to the final editor revision in both runs.
