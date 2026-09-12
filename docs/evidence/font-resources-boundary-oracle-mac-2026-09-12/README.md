# font-resources boundary oracle — run on mac-m1max-a (MacTeX 2026 full)

Requested by the Commander on issue #2 (2026-09-12T09:53:56Z). Oracle only: no
product engine, no shell escape, no pages emitted. Tool:
`crates/font-resources/tools/run_boundary_oracle.py` from
`origin/agent/commander-corpus/font-resources` c11506e, run unchanged with
`--output` on 2026-09-12T09:54Z.

Engine: TeX 3.141592653 (TeX Live 2026), `/usr/local/texlive/2026/bin/universal-darwin/tex`
sha256 733f1fb5…; TFtoPL 3.3 (TeX Live 2026) sha256 5d9a7c6c…; manifest sha256
1c09466c…. `result.json` carries every file hash; raw `oracle.log`, `oracle.tex`,
`probe.pl`, `tftopl.stdout` are preserved per case (the synthetic `probe.tfm`
bytes are reproducible from `probe.pl` with `pltotf` and are identified by
sha256 in result.json; they are not committed). `engine_exit` 1 is the
harness's intended `\showbox` stop ("! OK."), not a failure. `tftopl.stdout` is
empty for every case: no repairs; no font-load errors in any log.

Adjudicated node lists (`\showbox0`) against the stated hypotheses:

| case | TeX node list | hypothesis | outcome |
|---|---|---|---|
| both-kerns | `\kern-1.25`, `\probe A`, `\kern-1.25` (hbox x2.5) | kern(-1.25pt), A, kern(-1.25pt) | matches |
| left-ligature | `\probe B (ligature |A)` (x5.0) | B | matches |
| right-ligature | `\probe B (ligature A|)` (x5.0) | B | matches |
| retained-right-ligature | `\probe A`, `\probe B (ligature |)` (x10.0) | A, B | matches |
| real-boundary-code | `\probe A`, `\kern-1.25`, `\probe B` (x8.75) | A, kern, B | matches |

Input-interval provenance (which source bytes each node came from) is not
established by these node widths, as the request notes.
