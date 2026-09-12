# Original pipeline versus established LaTeX

This is a real non-equivalence measurement, not a passing parity fixture.
`manifest.json` pins every input/output, peer commit, and known resource gap.
`reference.pdf` was produced by pdfTeX 1.40.29 (TeX Live 2026), with the exact
engine preamble and SHA in `reference-engine.json`. It comes unchanged from
`tests/visual-corpus/evidence/20260912T064032Z/references/01-plain-paragraph/pdflatex-lm`
on the pinned visual-oracle branch. This is the 06:40 run, not the later Mac
09:27 run or its reported position accuracy.

`original.pdf` was freshly compiled on Linux with the unchanged original
render-pipeline 9bb7b2736f16c96f4ce356fbb89caf9de395402c. The original request
strips the fixture preamble; the reference substitutes the recorded LM preamble.
Both transformations are explicit, so this is not identical full-source input.
The pipeline's `--pdf` uses its vendored runtime-v1 PDF route, not rendering-core's
exact path exporter. `original-v2.json` separately preserves original GIDs,
source bytes and full LMRoman12 resource identity.

The official existing LM 2.004 OTF archive provided LMRoman12-Regular. Matching
`ec-lmr12.tfm` was absent. Pipeline `fonts.rs` records `tfm_missing`, but no caller
reads it and diagnostics remain empty: OTF metrics were used. This resource
configuration cannot inherit the Mac's matched-TFM accuracy claim. Reported in
https://github.com/flash-tex/flashtex/issues/2#issuecomment-5645061659.

The exact experimental consumer accepts the JSON shape but semantic validation
rejects its `opentype-cff` resource with `unsupported font profile`. Internal CFF
paths exist; this does not authorize widening the negotiated wire profile.
The regression exercises the actual producer bytes and refusal, with no invented
GIDs or silent font substitution.

The existing original PDF reader compares 66 candidate operators against 5
reference operators. Raw bytes and parsed operators differ. Operator-to-source
correspondence is unknown because sequences differ. Numeric placement equality
and raster equality are unmeasured; there is no visual parity claim.

Reproduce the consumer checks from the repository root:

```
cargo test --offline --manifest-path crates/rendering-core/Cargo.toml --test original_reference
cargo run --offline --manifest-path crates/rendering-core/Cargo.toml --example pdf_compare -- crates/rendering-core/tests/fixtures/original-reference/original.pdf crates/rendering-core/tests/fixtures/original-reference/reference.pdf /tmp/original-reference-report.json
```

The second command writes the report and returns 3 for differing bytes. To
regenerate the original output, extract `git archive 9bb7b27 crates/render-pipeline`
into an isolated directory (its pinned vendor crates avoid dependency collision),
build its `flashtex-render` binary offline, then run:

```
FLASHTEX_TFM_DIRS=/path/to/only-ec-lmr10-fixture-directory flashtex-render --font-dir /path/to/extracted-lm2.004otf --secnumdepth 0 --v2 original-v2.json --pdf original.pdf < request.jsonl
```

Keep the missing 12pt TFM condition explicit. Installing the proper TFM changes
the resource configuration and requires a new evidence manifest. Never use the
reference PDF as an input to the original renderer.

## Opt-in CFF consumer and matched metrics rerun

`pipeline_cff::PipelineCff` now offers an explicit additive path: parse the
existing typed display, validate its CFF profile, bind immutable registry
resources and UTF-8 source snapshots, then expand original GIDs with exact
FontMatrix/size/origin, caller-selected hint policy and bounded command counts.
It retains producer advances separately in `display()`; CFF outline advances
never replace supplied glyph origins. Font IDs may differ from raw resource
hashes. The legacy wire validator still rejects CFF. This API is opt-in and does
not negotiate a new protocol or select implicit fonts.

The stronger check exposed a producer defect: the previously declared font SHA
`d0f39b...` is actually SHA256(font bytes + four face-index zero bytes).
The supplied, unmodified LM2.004 font has raw SHA
`e6be218ae83e61aa8a29990d3cdc401c678c1962188cb9a4a8b6359e4f5e5870`.
Thus earlier statements of full-font identity describe the producer's claim,
not successful verification. The adapter refuses `CFF resource metadata mismatch`.
Producer fix requested at
https://github.com/flash-tex/flashtex/issues/2#issuecomment-5645097175.

`matched-v2.json` and `matched-legacy.pdf` are fresh unchanged pipeline outputs
using official LM2.004 `ec-lmr12.tfm` SHA299021120f0a29ef61278a2363903bd8defbb8faaade458eb79067342aecb56f.
The reference is unchanged. The legacy PDF still differs in bytes/operators;
visual and cross-producer source correspondence remain unknown. Exact export
refuses the mislabeled hash even with correct TFM metrics.

Run the actual refusal from repository root:

```
cargo run --offline --manifest-path crates/rendering-core/Cargo.toml --example pipeline_cff_probe -- crates/rendering-core/tests/fixtures/original-reference/matched-v2.json crates/rendering-core/tests/fixtures/original-reference/request.jsonl crates/rendering-core/tests/fixtures/original-reference/lmroman12-regular.otf crates/rendering-core/tests/fixtures/original-reference/GUST-FONT-LICENSE.txt /tmp/matched-exact
```

The font fixture is unmodified official LM2.004, redistributed with its adjacent
GUST license. The automated hypothetical corrected-contract test changes only
the hash in memory to exercise successful adapter geometry and atomic budget
refusals. That edited fixture is never labeled original output or used to claim
original-versus-reference equality. No producer metadata is silently repaired.

## Current producer 4888a67 framing replay

The three `4888-*.jsonl` fixtures are fresh unchanged producer stdout:
matched metrics accepts a v2 sibling, missing metrics emits `tfm_missing` with
recovered status, and an 8000-byte reply cap explicitly declines the v2 sibling.
`pipeline_frame::pair` binds the optional line to the accepted capability,
request ID, project and revision, rejects unsolicited/missing/oversized siblings,
and optionally refuses missing TeX metrics for reference acceptance. It is
transport validation only, never permission to paint or proof of font identity.

The new producer still mislabels the engine hash as the raw font hash; running
the CFF probe on its actual sibling still returns resource metadata mismatch.
No current exact original PDF can therefore be produced through this adapter.
The fresh `4888-legacy.pdf` and `4888-comparison.json` rerun the legacy route
against the same reference: bytes/operators differ, visual equality unknown.
Earlier producer comparisons remain separately pinned. Current blocker
verification: https://github.com/flash-tex/flashtex/issues/2#issuecomment-5645134718.
