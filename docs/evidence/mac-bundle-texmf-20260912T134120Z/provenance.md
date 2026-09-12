# GH36 — Mac bundle rooted TFM metrics, app-only acceptance (2026-09-12T13:41Z)

Lane `mac-packaging-tfm` (Claude Code subagent of `mac-claude-a`, mac-m1max-a,
Xcode 26.3 / Swift 6.2.4). Branch `agent/mac-packaging-tfm/texmf` (from
`origin/agent/mac-claude-a/mac-shell` 5bc3fc0 + `origin/main` ffe199d8 merged
for the pinned verifier). Tested source SHA: the commit that carries this
directory (see `coordination/mac-packaging-tfm.md`).

## What was run

1. `apps/mac/scripts/make-app.sh --debug --helper-root /Users/jay3332/Projects/flashtex --render <flashtex-render built from f762f82a>`
   (`make-app.log`). The producer was built with `cargo build --release` from an
   archive export of `crates/render-pipeline` at exactly
   `origin/agent/mac-render-pipeline/unified` tip f762f82a, unpatched: source
   binary sha256 `d8d2040e7e8d5716291398a5818a15c7c7586e3b79d57748b7f91c250bf00d63`,
   as shipped after ad-hoc codesign inside the bundle
   `59ca1ad82067eaf3f212976ce77ac910ad5be9a3d8a34b59091c27f6a1bb73f6`
   (`components.json` `render.sha256`). Other helpers came from the main
   checkout's `crates/*/target/release` (their source SHA/sha256 are in
   `components.json`).
   - Pre-flight: "Pinned rooted TFM metrics verified under apps/mac/Fonts/texmf".
   - Staging: "verified 6 rooted metric/license files + 3 pinned faces; report at
     Contents/Resources/resource-coverage.json" (copied here).
   - `components.json` (copied here) carries `"resources"` with the nine verified
     hashes and the manifest hash `3f0286b3…`.
2. `apps/mac/scripts/texmf-acceptance.sh --evidence <this dir>` (`README.md`,
   `*.output.jsonl`, `*.env.txt`, `*.stderr.txt`, `env-helper.frames.jsonl`,
   `removed.verifier.json`, `verifier.json`): the producer INSIDE the bundle,
   `env -i PATH=/usr/bin:/bin HOME=<empty tmp>` (no FLASHTEX_*/TEXMF*), under a
   `sandbox-exec` profile denying reads of `/usr/local/texlive`, `/Library/TeX`,
   `/usr/share/texmf`, `/usr/share/texlive` (MacTeX 2026 is installed on this
   machine; the sandbox makes it unreadable to the producer). Fixtures: the
   verified capture's exact 10 pt multi-document requests
   (`crates/preview-controller/benchmarks/display-helper-multidoc-10pt/verified/producer-1636267.input.jsonl`,
   preview-1/preview-2), a 12 pt text document and a 12 pt roman-math document.
3. `swift test --filter BundledMetricsTests` with `FLASHTEX_RENDER=<f762f82a build>`
   (`swift-test-BundledMetricsTests.log`): 7/7 passed, 0.46 s; 1-min load 21.3
   before / 26.8 after (`uptime` lines in the log). Full `swift test` NOT run
   (parent instruction: load 80–120 while 12 lanes compile).
4. `apps/mac/scripts/packaging-selftest.sh --evidence packaging-selftest-fast.md`:
   25 ok, 0 failed (fast mode; includes the four new pre-build refusal checks:
   corrupted metric, missing metric, symlinked metric, absent root).

## Results

| run | FLASHTEX_TFM_DIRS | compile results | missing-metric diagnostics |
|---|---|---|---|
| control (env route off) | unset | 4/4 | **9** — 6 `required_metrics_unavailable` + 3 `tfm_missing`: the unpatched f762f82a producer does not find `../Resources/texmf` on its own |
| env-direct (what `WorkerClient` sets) | `<app>/Contents/Resources/texmf/fonts/tfm/public/lm` | 4/4 | **0** (only `math_resource_profile` notes, 8) |
| env-user (user entry appended after the bundled dir) | `<bundled>:<home>/user-tfm` | 4/4 | **0** |
| env-helper (bundled `flashtex-preview-controller`, file-backed project, spawning the bundled producer) | as env-direct | preview update with pages | **0** |
| removed (`ec-lmr10.tfm` deleted from a `ditto` copy of the bundle) | copy's bundled dir | 4/4 | **3** `tfm_missing` naming `ec-lmr10.tfm` (explicit, not silent); `verify_bundle_resources.py` on the copy exits 1 with `ec-lmr10.tfm … "missing"` |
| verifier on the real bundle | — | — | `verify_bundle_resources.py <app>/Contents/Resources` exit 0, 9/9 verified |

Missing-metric codes counted: `tfm_missing`, `required_metrics_unavailable`,
`font_unavailable` (render-pipeline `typeset.rs`).

## Resource hashes (as shipped, byte-verified)

- `Fonts/lmroman10-regular.otf` 1aa18cfe… (111536), `Fonts/lmroman12-regular.otf` e6be218a… (110400), `Fonts/latinmodern-math.otf` 6075562b… (733736)
- `texmf/fonts/tfm/public/lm/ec-lmr10.tfm` cd13479f… (12056), `ec-lmr12.tfm` 29902112… (12092), `rm-lmr12.tfm` 9d4e3d8e… (11888), `rm-lmr6.tfm` eb0bfdf8… (11836), `rm-lmr8.tfm` 80bcbfd8… (11864)
- `texmf/doc/fonts/lm/GUST-FONT-LICENSE.TXT` 49ea6cb9… (1377)

Full digests in `resource-coverage.json` / `components.json`. The vendored
sources (`apps/mac/Fonts/texmf`) were copied from MacTeX 2026 only after their
SHA-256 matched the manifest exactly (all six did).

## Limitations

- The app GUI itself was not launched for this acceptance (no window, no
  focus): the env is what `BundledMetrics.producerEnvironment()` computes, unit
  tested, and applied in `WorkerClient`/`PreviewControllerClient`; the
  acceptance runs the bundled binaries with that exact env. A GUI-driven
  compile through the packaged app with the sandbox is not included.
- Producer discovery route (Commander's `producer-discovery.patch`): NOT
  applied here (render-pipeline lane owns it); the control run documents the
  current behaviour. Re-run with a patched producer SHA to prove that route.
- Coverage is the five pinned files: bold/italic/7 pt/17 pt TFMs are not
  bundled (separate inventory).
- Debug app build (`--debug`) to bound build time under load; the staging
  and signing paths are identical for release.
