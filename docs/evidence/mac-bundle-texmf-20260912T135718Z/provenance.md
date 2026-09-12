# GH36 — third acceptance: producer 9aaec57a (render-pipeline tip) (2026-09-12T13:57Z)

Lane `mac-packaging-tfm`. Identical bundle layout, fixtures and procedure to
`../mac-bundle-texmf-20260912T135157Z/provenance.md`; only the producer changed,
on the parent's request to use the advanced render-pipeline tip:

- `origin/agent/mac-render-pipeline/unified` @ 9aaec57a (includes discovery
  421a2049 and the reply-side speedups), built with `cargo build --release`
  from an archive export of `crates/render-pipeline` at 9aaec57a (build exit
  0). Source binary sha256
  `ed729b02befeb7d34ff97cccf7f6d0c4463f9e2bde43670c3ea7abaf62ac5283`; as
  shipped after ad-hoc codesign
  `79c2be4c5859b7788d3f3e57c2248854bc3fe2438f4392e1df5e3f36163d7d59`.
- `components.json` now records the source revision explicitly:
  `"render": {"git_sha": "9aaec57a", "git_sha_origin": "declared", "sha256":
  "79c2be4c…"}` via the new `make-app.sh --source-sha render=9aaec57a` (a
  declared revision for a helper built outside a repository checkout;
  `resolved` is what a checkout-built helper records; a declared SHA that
  contradicts a resolvable one refuses packaging).

## Results (`README.md`; host TeX denied by env -i + sandbox-exec)

| run | FLASHTEX_TFM_DIRS | results | missing-metric diagnostics |
|---|---|---|---|
| control = discovery route (`--require-discovery`) | unset | 6/6 | **0** |
| env-direct | bundled dir | 6/6 | **0** |
| env-user | bundled dir + user entry | 6/6 | **0** |
| env-helper (bundled preview controller) | bundled dir | preview with pages | **0** |
| removed (`ec-lmr10.tfm` deleted from a copy) | copy's dir | 6/6 | **7** `tfm_missing` naming ec-lmr10.tfm; verifier exit 1 on the copy |
| verifier (real bundle) | — | — | exit 0, 9/9 pinned; components.json 9 + 23 supplementary |

Per-run `uptime` lines are in `README.md` (1-min load 8.75 at start).

- `swift test --filter BundledMetricsTests` with `FLASHTEX_RENDER=<9aaec57a build>`:
  8/8 passed (0.56 s), 1-min load 6.7 (`swift-test-BundledMetricsTests.log`).
- `packaging-selftest.sh` fast: 25 ok, 0 failed (`packaging-selftest-fast.md`).
- Not run: full `swift test` (parent instruction under load), GUI-driven compile.
