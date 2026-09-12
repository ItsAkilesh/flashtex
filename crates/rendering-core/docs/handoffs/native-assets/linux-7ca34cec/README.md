# Actual Linux sibling-layout discovery probe

Source7ca34cec09bc4b06714f7cc58720a5b24923cafa was archived unchanged and built
`--release --offline --locked -j2` with Rust/Cargo1.98.1. All375 archived source
files matched the extracted tree after build. Binary SHA256:
`d002742dba6bd309c709a748beba671d35b3010f3cbd3996cabe412c925d92d7`.
Exact archive/toolchain/build log hashes are in `build-provenance.json`.

The actual binary was copied into a temporary
`FlashTeX.app/Contents/MacOS/flashtex-render` sibling layout on Linux, with all nine
manifest-pinned resources under Resources. The existing verifier passed before
and after the controlled cases. Every compiled default host-font path was checked
absent; each child received only PATH/LANG and the deliberately supplied TFM
override. No font-dir argument or inherited TeX environment was used. Each case
used a fresh process and the existing original10pt multi-document or12pt plain
request, whose byte hash is recorded. Original stdout, v2 and stderr are retained.

Observed acceptance:

| Case | Actual result |
| --- | --- |
| bundled10pt and12pt | ok, zero diagnostics |
| missing bundled ec-lmr10 | recovered, tfm_missing |
| corrupt explicit flat ec-lmr10 with valid bundled10pt | recovered, tfm_missing: explicit override selected |
| corrupt explicit flat ec-lmr12 with valid bundled root | ok: required rooted set used before flat override |
| missing rooted ec-lmr12 plus corrupt flat12pt | recovered, required_metrics_unavailable |
| missing rooted ec-lmr12 plus valid flat12pt | ok: valid flat fallback accepted |
| missing rooted ec-lmr12 without override | recovered, required_metrics_unavailable |

These controlled outcomes establish the selection behavior; no syscall trace was
available or claimed. The required12pt ordering differs from simple non-required
10pt search. Owners must decide whether that intended root-before-flat behavior
meets explicit override UX and document it; this probe does not change policy.

`tools/probe_bundle_discovery.py` reproduces the eight cases from the recorded
build directory and existing official metric directory. The build and temporary
stage live under the owned /home target, avoiding the nearly-full /tmp tmpfs; the
stage was removed afterward. The report's ephemeral paths belong only to those
original runs. The existing recovered captures and old producer binaries remain
unchanged.

This proves actual unchanged Linux producer discovery with the sibling layout,
not signing, installation, sandbox permissions, native child launch, Mac paint,
reference parity, or full math rendering. The math OTF was byte-verified as part
of the nine resources; the two requests exercise regular10pt/12pt coverage only.
The existing Mac packaging/direct/helper app-only nohostTeX gate remains required.
