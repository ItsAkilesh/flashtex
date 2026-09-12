# mac-packaging-tfm handoff — GH36 rooted TFM metrics in the Mac bundle

- Updated UTC: see `coordination/agents/mac-packaging-tfm.json` `updated_utc`
- Agent / parent / machine: `mac-packaging-tfm` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: GH36 "Mac bundle ships OTF fonts but omits required rooted TFM assets
  for no-TeX operation" (Commander handoff
  `crates/rendering-core/docs/handoffs/native-assets/` on main, manifest
  sha256 3f0286b3…, verifier product 8e6d2787). Owned paths:
  `apps/mac/Fonts/texmf/**`, `apps/mac/Fonts/README.md` (new section),
  `apps/mac/scripts/{make-app.sh,packaging-selftest.sh,launch-check.sh}`,
  new `apps/mac/scripts/{bundle-texmf.py,texmf-acceptance.sh}`,
  new `apps/mac/Sources/FlashTeXMac/BundledMetrics.swift`,
  `apps/mac/Sources/FlashTeXMac/{WorkerClient,PreviewControllerClient}.swift`
  (one `process.environment` line each; not parent-retained),
  new `apps/mac/Tests/FlashTeXMacTests/BundledMetricsTests.swift`,
  `apps/mac/README.md` + `apps/mac/docs/packaging.md` (packaging sections),
  `docs/evidence/mac-bundle-texmf-20260912T134120Z/`, this handoff and
  `coordination/agents/mac-packaging-tfm.json`. NOT touched: any parent-retained
  file (`ShellModel*.swift`, `ContentView`, `PreviewView`, `FlashTeXMacApp`,
  `SourceEditorView`), any crate. No parent diffs are needed: both producer
  launch sites live in non-retained files.
- Branch: `agent/mac-packaging-tfm/texmf` from
  `origin/agent/mac-claude-a/mac-shell` 5bc3fc0 with `origin/main` ffe199d8
  merged (clean; brings the pinned verifier; no apps/mac changes from main).

## Coverage audit (mandatory first step)

Searched `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`,
`apps/mac/scripts/*`, `apps/mac/Sources/**` for `tfm|texmf|FLASHTEX_TFM_DIRS|GUST|GH36`:

- No test, script or evidence covered any part of the gap. The only hits were
  `PreviewV2Tests.swift:679` (a host MacTeX `lm-math` OTF path constant),
  `Fonts.swift:19-21` / `PreviewFonts.latinModernSearchPaths` (OTF lookup, host
  TeX fallbacks), `make-app.sh:251-254` (copies `Fonts/*.otf`, `*.TXT`, README
  only — the exact block the issue cites), and
  `docs/evidence/font-resources-boundary-oracle-mac-2026-09-12/` (synthetic
  `probe.tfm` hashes for a font-resources oracle, unrelated to bundling).
- No `.tfm` existed under `apps/mac/`; no `FLASHTEX_TFM_DIRS` reference existed
  in Swift; neither `WorkerClient` nor `PreviewControllerClient` set
  `process.environment`. The mac-live report and existing XCTests contain no
  no-host-TeX or rooted-metrics check. Gap fully uncovered → implemented.

## Delivered

1. Vendored `apps/mac/Fonts/texmf/fonts/tfm/public/lm/{ec-lmr10,ec-lmr12,rm-lmr12,rm-lmr6,rm-lmr8}.tfm`
   and `apps/mac/Fonts/texmf/doc/fonts/lm/GUST-FONT-LICENSE.TXT`, copied from
   MacTeX 2026 (`/usr/local/texlive/2026/texmf-dist`) only after every SHA-256
   and length matched the manifest exactly (all six matched: cd13479f…,
   29902112…, 9d4e3d8e…, eb0bfdf8…, 80bcbfd8…, 49ea6cb9…).
2. `scripts/bundle-texmf.py` (`check <root>` / `stage <root> <Resources> <report>`)
   reuses the verifier's `pinned_manifest`/`check_resource`/`verify` (no-follow
   descriptor walk; refuses symlinks, length or digest mismatch, missing files).
   `make-app.sh`: pre-flight source check BEFORE `swift build` (exit 1), staging
   into `Contents/Resources/texmf/…` after the Fonts copy, full-Resources
   verification (9 pinned resources) before any signing, `resource-coverage.json`,
   and a `"resources"` entry with the nine hashes in `components.json`
   (written before codesign, so sealed). `FLASHTEX_BUNDLE_TEXMF_ROOT` overrides
   the source root; the host TeX tree is never consulted; nothing downloads.
3. `BundledMetrics.swift`: `tfmDirectory()` (bundle `Resources/texmf`, else the
   repository copy for `swift build` products) and
   `producerEnvironment(base:bundledDirectory:)` which prepends the bundled
   absolute directory to `FLASHTEX_TFM_DIRS`, keeps explicit user entries in
   order after it (dedupes the bundled path, drops empty entries), and passes
   every other variable through; base unchanged when no bundled dir exists.
   Applied on both routes: `WorkerClient.init` (direct producer) and
   `PreviewControllerClient.init` (helper; its compiler child inherits the env —
   the controller spawns with `Command::new(path)` and no env changes).
4. `scripts/texmf-acceptance.sh`: bundled producer, host TeX excluded by
   `env -i` AND a `sandbox-exec` deny profile for `/usr/local/texlive`,
   `/Library/TeX`, `/usr/share/texmf|texlive`; runs control / env-direct /
   env-user / env-helper (bundled preview controller, file-backed project) /
   removed (`ec-lmr10.tfm` deleted from a copy) / verifier; writes evidence.
   `packaging-selftest.sh`: four new pre-build refusal checks (corrupted,
   missing, symlinked metric; absent root), help/syntax of the new script,
   acceptance in `--full`. `launch-check.sh`: static verifier + components
   check step (1b). Docs: `apps/mac/README.md` "Rooted TeX metrics",
   `apps/mac/docs/packaging.md` section, `apps/mac/Fonts/README.md`.
5. Tests `BundledMetricsTests` (7): vendored tree vs pinned hashes, directory
   discovery (first existing root; file-not-directory refused; default finds the
   repo copy), prepend semantics, env pass-through, untouched-without-bundle,
   and a real-producer test (XCTSkip without `FLASHTEX_RENDER` or sandbox-exec)
   running the 10 pt multi-doc + 12 pt text + 12 pt math with host TeX denied:
   env route → 0 missing-metric diagnostics; user entry appended → 0; ec-lmr10
   removed → explicit diagnostic naming the file.

## Second pass (parent/Commander 13:48Z): discovery producer + supplementary metrics

- `origin/agent/mac-render-pipeline/unified` 98e829bf (discovery 421a2049)
  built from an archive export (`cargo build --release`): source sha256
  9bb85a34…, bundled/codesigned 269192ab…; repackaged with `--render`.
- 23 supplementary TFMs vendored (`ec-lmr{5,6,7,8,9,17}`, `ec-lmbx{5..12}`,
  `ec-lmri{7..12}`, `ec-lmbxi10`, `rm-lmr{5,7,9,10}`) with the in-repo pin
  `apps/mac/Fonts/texmf/SUPPLEMENTARY-METRICS.json`; `bundle-texmf.py` verifies
  them (tier `supplementary`), stages them, re-verifies the staged copies and
  records them in `components.json` `resources.supplementary`. Provenance
  stated honestly in the pin: MacTeX 2026 (TL `lm` rev 77682, catalogue 2.005,
  MANIFEST 2.004), byte-identical to the CTAN `lm.zip` copy on this machine
  (sha 71c48809…), NOT verified against the Commander-pinned lm2.004bas.zip
  (97a725ea…, not present locally; nothing downloaded). The five pinned 2.004
  files matching these same sources is the evidence the TFMs are unchanged.
  The parent may drop this tier by deleting the JSON + files; packaging then
  stays green with the pinned five only.
- `texmf-acceptance.sh --require-discovery` (new flag; fixtures now 6: +
  `styles-10pt` bold/italic/bold-italic, + `text-11pt`; per-run `uptime`):
  evidence `docs/evidence/mac-bundle-texmf-20260912T135157Z/` — control =
  discovery route 6/6 **0** missing-metric diagnostics; env-direct 6/6 **0**;
  env-user 6/6 **0**; env-helper preview **0**; removed 7 explicit
  `tfm_missing` naming ec-lmr10 + verifier exit 1 on the copy; verifier exit 0;
  components.json 9 pinned + 23 supplementary. 8 ok / 0 failed.
  `BundledMetricsTests` now 8/8 (supplementary pin test; 5 fixtures in the
  real-producer test) with the 98e829bf producer, load 7.4. packaging-selftest
  fast 25 ok.

## Third pass (parent 13:56Z): render-pipeline tip 9aaec57a

- Producer rebuilt from `origin/agent/mac-render-pipeline/unified` @ 9aaec57a
  (archive export, `cargo build --release`): source sha256 ed729b02…, bundled/
  codesigned 79c2be4c…. `make-app.sh --source-sha render=9aaec57a` (new
  option) records `"git_sha": "9aaec57a", "git_sha_origin": "declared"` in
  `components.json` for a helper built outside a checkout (a checkout-built
  helper records `resolved`; a contradiction refuses packaging).
- `texmf-acceptance.sh --require-discovery` → evidence
  `docs/evidence/mac-bundle-texmf-20260912T135718Z/`: discovery 6/6 **0**,
  env-direct 6/6 **0**, env-user 6/6 **0**, helper **0**, removed 7 explicit +
  verifier exit 1, verifier exit 0, components 9 + 23. 8 ok / 0 failed, load
  8.75. `BundledMetricsTests` 8/8 with the 9aaec57a producer (load 6.7).
  packaging-selftest fast 25 ok.

## Evidence (`docs/evidence/mac-bundle-texmf-20260912T134120Z/` — env route, unpatched f762f82a producer)

- `make-app.log`: `--debug --helper-root <main checkout> --render <f762f82a build>`;
  pre-flight and staging verified; components.json valid; ad-hoc signed.
  Producer built from an archive export of `crates/render-pipeline` at exactly
  f762f82a (unpatched): source sha256 d8d2040e…, as-shipped (codesigned)
  59ca1ad8….
- `README.md` (acceptance): control 4/4 results with **9** missing-metric
  diagnostics (unpatched producer does not discover the bundle by itself);
  env-direct 4/4, **0**; env-user 4/4, **0**; env-helper preview with pages,
  **0**; removed: 3 explicit `tfm_missing` naming `ec-lmr10.tfm` and verifier
  exit 1 on the copy; verifier exit 0 on the real bundle; components.json
  carries 9 verified hashes. `uptime` at run: load 5.6 / 23.5 / 23.2.
- `swift-test-BundledMetricsTests.log`: 7/7 passed (0.46 s), load 21.3 → 26.8.
- `packaging-selftest-fast.md`: 25 ok, 0 failed.
- `components.json`, `resource-coverage.json`, `verifier.json`,
  `removed.verifier.json`, per-run `*.output.jsonl` / `*.env.txt`, `provenance.md`.

Not measured: full `swift test` (parent instruction under load 80–120), a
GUI-driven compile through the packaged app under the sandbox. The discovery
route IS measured in the second pass above (98e829bf).

## Durable checkpoint

- Task: GH36 / Commander handoff f576ac11 (patches) + 06ad9179 (manifest);
  assignment via parent brief `prompt-mac-packaging-tfm.md`.
- Branch/worktree: `agent/mac-packaging-tfm/texmf` in
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ac12dd0004019340d`;
  merged `origin/main` ffe199d8 at 983d47e9; consumed mac-shell 5bc3fc0.
- Dirty files: none after the two commits (4e666555 first pass; second pass
  commit carries the supplementary tier, discovery acceptance, registration).
- Scratch: `/tmp/ft-tfm` → session scratchpad; `render-f762f82a/` build,
  `make-app.log`, `swift-test.log`; app at `apps/mac/build/FlashTeX.app`
  (gitignored).
- Next commands (if resumed): `git fetch origin && git status`;
  `apps/mac/scripts/packaging-selftest.sh`; with a new producer SHA:
  `cargo build --release` from that tip, `make-app.sh --debug --render <bin>`,
  `texmf-acceptance.sh --evidence docs/evidence/mac-bundle-texmf-<UTC>`.
- Decisions: env route (not producer patch) per Commander scope; bundled dir
  FIRST then user entries (brief + handoff README wording "prepend … preserving
  explicit user entries"); vendored tree in-repo rather than a packaging-time
  external root so packaging is reproducible and refuses on drift.
- Ownership: no parent-retained file changed; no crate changed.
- Staffing/billing: shared Claude Max quota with parent; no purchases; no
  paid calls; no background jobs left running (all spawned processes exited).
