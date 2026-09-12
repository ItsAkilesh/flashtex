# GH36 — second acceptance: producer 98e829bf (bundle discovery) + supplementary metrics (2026-09-12T13:52Z)

Lane `mac-packaging-tfm` (Claude Code subagent of `mac-claude-a`, mac-m1max-a).
Same branch/bundle layout as `../mac-bundle-texmf-20260912T134120Z/provenance.md`,
re-run after two changes requested by the parent/Commander:

1. Producer rebuilt from `origin/agent/mac-render-pipeline/unified` tip 98e829bf
   (GH36 discovery commit 421a2049): `cargo build --release` on an archive export
   of `crates/render-pipeline` at 98e829bf; source binary sha256
   9bb85a34d73b635580e0d0b079bd6263ff4b8d7e7cce20e0c272701519b17a67, as shipped
   after ad-hoc codesign 269192abf3c9d6ef43f748d9f8adbf0f2846ca4eaa6a92738670502a53e47ae2
   (`components.json` `render.sha256`; `git_sha` is `unknown` because the export
   directory is not a repository — the SHA is pinned here instead).
2. 23 supplementary Latin Modern TFMs added to `apps/mac/Fonts/texmf` under the
   in-repo pin `SUPPLEMENTARY-METRICS.json` (provenance and limitation stated
   there: MacTeX 2026 TeX Live `lm` rev 77682 / catalogue 2.005 / MANIFEST 2.004,
   byte-identical to the CTAN `lm.zip` copy on this machine, NOT verified against
   the Commander-pinned lm2.004bas.zip). `make-app.sh` verified 29 entries
   pre-build and staged them (`make-app.log`); `components.json` records them
   under `resources.supplementary` (23 sha256/byte pairs).

Faces now covered with TeX metrics: `ec-lmr` 5/6/7/8/9/10/12/17,
`ec-lmbx` 5/6/7/8/9/10/12, `ec-lmri` 7/8/9/10/12, `ec-lmbxi` 10,
`rm-lmr` 5/6/7/8/9/10/12. Not covered: sans, typewriter, caps, slanted, dunhill.

## Fixtures (6 requests)

10 pt multi-document preview-1/preview-2 (verified capture input), 12 pt text,
12 pt math, **10 pt regular+bold+italic+bold-italic with inline math**
(`styles-10pt`), **11 pt bold/italic with math** (`text-11pt`).

## Results (`README.md`; host TeX denied by env -i + sandbox-exec)

| run | FLASHTEX_TFM_DIRS | results | missing-metric diagnostics | uptime (1-min load) |
|---|---|---|---|---|
| control = **discovery route** (`--require-discovery`) | unset | 6/6 | **0** | 6.67 |
| env-direct (env route) | bundled dir | 6/6 | **0** | 6.67 |
| env-user | bundled dir + user entry | 6/6 | **0** | 6.67 |
| env-helper (bundled preview controller) | bundled dir | preview with pages | **0** | — |
| removed (`ec-lmr10.tfm` deleted from a copy) | copy's dir | 6/6 | **7** `tfm_missing` naming ec-lmr10.tfm; verifier exit 1 on the copy | 8.69 |
| verifier (real bundle) | — | — | exit 0, 9/9 pinned; components.json 9 + 23 supplementary | — |

`styles-10pt` and `text-11pt` produce only `math_resource_profile` notes (4 and
2) — bold/italic/11 pt text is laid out with the supplementary TFMs, no
`tfm_missing`. The 134120Z run (unpatched f762f82a producer, 4 fixtures) remains
the env-route-only record: there the control row had 9 missing-metric
diagnostics and every env-* row 0.

## Also

- `swift test --filter BundledMetricsTests` with `FLASHTEX_RENDER=<98e829bf build>`:
  8/8 passed (0.55 s), 1-min load 7.4 → 7.4 (`swift-test-BundledMetricsTests.log`).
- `packaging-selftest.sh` fast: 25 ok, 0 failed (`packaging-selftest-fast.md`).
- Not run: full `swift test` (parent instruction under load), GUI-driven compile.
