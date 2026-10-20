**NON-REFERENCE run**: power source unknown at start; power source unknown at end; CPU quota of flashtex-agents.slice throttled 455 of 1629 periods (83.9 s stopped)

| doc | pages | edit | n | ms p50 / p95 / max | gated on | target | first changed page p95 | DONE p95 ms | off-CPU p95 ms | converged | re-typeset pages p50 / max | host RSS peak | load1 | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| beamer-5 | 5 | letter@start | 19 | 4.8 / 6.1 / 6.3 | p95 | 11 | 6.1 | 39.6 | 0.1 | 19/19 | 2 / 2 | 238 MB | 0.35-0.4 | PASS (no baseline row) |
| beamer-5 | 5 | letter@middle | 19 | 4.6 / 5.4 / 5.7 | p95 | 11 | 5.4 | 38.2 | 0.1 | 19/19 | 2 / 2 | 238 MB | 0.4-0.37 | PASS (no baseline row) |
| beamer-5 | 5 | letter@end | 19 | 4.7 / 5.5 / 5.7 | p95 | 11 | 5.5 | 40.3 | 0.1 | 19/19 | 2 / 2 | 238 MB | 0.37-0.39 | PASS (no baseline row) |
| beamer-5 | 5 | sentence@middle | 19 | 5.0 / 6.5 / 6.5 | p95 | 11 | 6.5 | 31.6 | 0.1 | 19/19 | 2 / 2 | 238 MB | 0.39-0.36 | PASS (no baseline row) |
| beamer-5 | 5 | newline@middle | 19 | 4.7 / 5.4 / 5.9 | p95 | 11 | 5.4 | 37.9 | 0.0 | 19/19 | 2 / 3 | 238 MB | 0.36-0.33 | PASS (no baseline row) |
| beamer-5 | 5 | split@middle | 19 | 4.7 / 5.8 / 12.5 | p95 | 11 | 5.8 | 64.4 | 0.1 | 19/19 | 3 / 3 | 238 MB | 0.33-0.36 | PASS (no baseline row) |
| beamer-5 | 5 | typing@50ms | 39 | 4.4 / 4.7 / 5.4 | p95 | 11 | - | - | 0.1 | - | - | 238 MB | 0.36-0.36 | PASS |
| beamer-5 | 5 | typing@60ms | 39 | 4.5 / 4.8 / 5.3 | p95 | 11 | - | - | 0.1 | - | - | 238 MB | 0.36-0.33 | PASS |
| beamer-5 | 5 | typing@80ms | 39 | 5.4 / 42.6 / 43.3 | p95 | 11 | - | - | 0.1 | - | - | 238 MB | 0.33-0.33 | MISS: p95 42.6 ms > 11 ms |
| beamer-5 | 5 | typing@100ms | 39 | 5.7 / 6.2 / 6.4 | p95 | 11 | - | - | 0.1 | - | - | 238 MB | 0.33-0.54 | PASS |
| beamer-5 | 5 | typing@150ms | 39 | 6.2 / 20.1 / 20.7 | p95 | 11 | - | - | 0.1 | - | - | 238 MB | 0.54-0.82 | MISS: p95 20.1 ms > 11 ms |
| beamer-5 | 5 | preamble | 5 | 713.2 / 761.1 / 761.1 | max | 400 | 761.1 | 880.1 | 348.9 | 0/5 | 5 / 5 | 238 MB | 0.82-0.77 | MISS: max 761.1 ms > 400 ms (no baseline row) |
| beamer-5 | 5 | reopen: host's share, cold (spawn -> page 1) | 3 | 146.4 / 232.8 / 232.8 | max | (100, app) | - | - | - | - | - | 238 MB | 0.77-0.77 | report-only: max 232.8 ms > 100 ms |
| beamer-5 | 5 | reopen: host already listening (not counted, decision 8) | 3 | 73.0 / 114.5 / 114.5 | max | (100, app) | - | - | - | - | - | 238 MB | 0.77-0.77 | report-only: max 114.5 ms > 100 ms |
| beamer-30 | 58 | letter@middle | 19 | 15.1 / 50.3 / 53.3 | p95 | 11 | 50.3 | 148.1 | 34.2 | 19/19 | 2 / 2 | 304 MB | 0.71-0.68 | MISS: p95 50.3 ms > 11 ms (no baseline row) |
| beamer-30 | 58 | letter@end | 19 | 14.7 / 53.0 / 56.3 | p95 | 11 | 53.0 | 114.0 | 38.2 | 19/19 | 2 / 2 | 304 MB | 0.68-0.73 | MISS: p95 53.0 ms > 11 ms (no baseline row) |
| beamer-30 | 58 | sentence@middle | 19 | 14.6 / 16.3 / 25.7 | p95 | 11 | 16.3 | 65.6 | 0.1 | 19/19 | 2 / 2 | 304 MB | 0.73-0.77 | MISS: p95 16.3 ms > 11 ms (no baseline row) |
| beamer-30 | 58 | newline@middle | 19 | 14.5 / 15.5 / 42.2 | p95 | 11 | 15.5 | 104.4 | 0.1 | 19/19 | 3 / 5 | 304 MB | 0.77-0.71 | MISS: p95 15.5 ms > 11 ms (no baseline row) |
| beamer-30 | 58 | split@middle | 19 | 14.7 / 15.6 / 16.3 | p95 | 11 | 15.6 | 230.3 | 0.1 | 19/19 | 5 / 5 | 304 MB | 0.71-0.68 | MISS: p95 15.6 ms > 11 ms (no baseline row) |
| beamer-30 | 58 | letter@start | 19 | 16.0 / 17.1 / 21.3 | p95 | 11 | 17.1 | 69.6 | 0.1 | 19/19 | 2 / 2 | 304 MB | 0.68-0.71 | MISS: p95 17.1 ms > 11 ms (no baseline row) |
| beamer-30 | 58 | typing@50ms | 39 | 12.4 / 14.8 / 30.1 | p95 | 11 | - | - | 0.6 | - | - | 304 MB | 0.71-0.73 | MISS: p95 14.8 ms > 11 ms |
| beamer-30 | 58 | typing@60ms | 39 | 12.4 / 12.7 / 15.3 | p95 | 11 | - | - | 0.1 | - | - | 304 MB | 0.73-0.73 | MISS: p95 12.7 ms > 11 ms |
| beamer-30 | 58 | typing@80ms | 39 | 12.6 / 14.1 / 16.1 | p95 | 11 | - | - | 0.1 | - | - | 304 MB | 0.73-0.75 | MISS: p95 14.1 ms > 11 ms |
| beamer-30 | 58 | typing@100ms | 39 | 12.7 / 14.1 / 14.9 | p95 | 11 | - | - | 0.1 | - | - | 304 MB | 0.75-0.69 | MISS: p95 14.1 ms > 11 ms |
| beamer-30 | 58 | typing@150ms | 39 | 12.8 / 13.9 / 14.3 | p95 | 11 | - | - | 0.1 | - | - | 304 MB | 0.69-0.72 | MISS: p95 13.9 ms > 11 ms |
| beamer-30 | 58 | preamble | 5 | 432.0 / 438.1 / 438.1 | max | 400 | 438.1 | 1810.1 | 2.9 | 0/5 | 58 / 58 | 304 MB | 0.72-0.68 | MISS: max 438.1 ms > 400 ms (no baseline row) |
| beamer-30 | 58 | reopen: host's share, cold (spawn -> page 1) | 3 | 165.3 / 166.5 / 166.5 | max | (100, app) | - | - | - | - | - | 304 MB | 0.63-0.66 | report-only: max 166.5 ms > 100 ms |
| beamer-30 | 58 | reopen: host already listening (not counted, decision 8) | 3 | 92.8 / 94.3 / 94.3 | max | (100, app) | - | - | - | - | - | 304 MB | 0.63-0.66 | report-only |

ms: COMPILE written to the watched page's PAGE frame read on the socket (preamble, reopen: page 1); typing@: the keystroke sent to the watched page from its own compile or a later one. off-CPU: of the host's time to that page, what its engine thread spent descheduled (wall minus queue minus thread CPU): load or a CPU quota, not the engine. The first sample of each phase is a warm-up, left out; rows with n < 12 are gated on their maximum. No noise margin: a row passes only when it meets its target.
