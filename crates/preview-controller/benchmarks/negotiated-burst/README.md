# Negotiated helper burst evidence

One run per size/policy, same release helper ab945e6 and pinned e75741e compiler.
Twenty edits target 30 ms intervals. Actual send intervals and binary hashes are
recorded in each result. No native paint or LLM call is measured.

| Actual source size | Policy | Current | Historical | Final after last send |
| --- | --- | --- | --- | --- |
| ~5 KB | off | 20 | 0 | 6.06 ms |
| ~5 KB | on | 20 | 0 | 4.03 ms |
| ~52 KB | off | 2 | 0 | 79.44 ms |
| ~52 KB | on | 6 | 12 | 68.73 ms |

All 80 acknowledged edits in those four runs preserve exact source/revisions;
four final compiler results equal independent clean builds and four killed-helper
reopens recover exact final source. Every historical frame checks original token,
source revision, session/project, inactive source-action flags and monotonic display
ordering. These checks do not constitute independent clean comparison of every
historical result. Historical delivery lag is reported separately in the JSON.

The 52 KB enabled run's actual mean send interval was 37.45 ms versus 30.05 ms
disabled: sender/consumer/runtime contention and pipe backpressure affect the test.
Do not attribute the current-preview count difference or final latency solely to
negotiation, claim a stable speedup, or equate frames with native paints.

Both requested 500 KB runs failed before typing. The unchanged workload generator
produces 521,792 bytes; the valid original compiler response is 13,117,053 bytes
and 129 pages, above the helper's 12,582,912-byte compiler-frame limit. The driver
previously ignored the failure event until timeout; it now fails immediately.
The failed cases are preserved separately and excluded from passing burst counts.
No pages were removed and no source was reduced to make the case pass. Larger-frame
or streamed-output work is required before this case can participate in the gate.
