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
