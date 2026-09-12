# Actual inline-math producer/reference evidence

This fixture comes from untouched published render-pipeline65dbe7d and the existing
visual-corpus06-math-inline source/reference. It uses the exact Mac b898cfc
LatinModernMath-Regular asset (733736bytes,SHA6075562b771f8b82f0c179e363389684f2dd09de30038269e2628e504bd7be0f)
with its adjacent GUST license, existing LM2.004 text font, and the same rooted
four required TFM/license assets as the clean plain fixture. No fonts were
substituted, installed, or downloaded. The producer reports zero diagnostics.

`display.json` is unmodified actual producer output. Math logical text contains
`ab`, `α+β` and `√x`; provenance may cover the whole original TeX expression,
including command spelling. The exporter consumes cluster text; it neither uses
source spelling as visible text nor guesses Unicode by reversing glyph IDs.
The ordinary regression recreates exact PDF bytes through immutable registry
resources and the existing PDF owner subsetter/writer.

Poppler26.01.0 extracts identical UTF-8 bytes from original and established
pdfTeX2026 reference PDFs. At144DPI1224×1584 RGB,602 pixels differ. Therefore this
case establishes equal linear extracted text, not visual parity or semantic
fraction/accessibility structure. Raw PDF bytes and operators also differ;
cross-producer operator/source correspondence remains unknown. No new text-policy
refusal is justified by this successful mapping case; existing ambiguous-GID,
multiple-glyph-cluster and error-diagnostic refusals remain.

Reproduce export with the new `pipeline_fonts_probe` example, using a directory
containing exactly the declared text and math font bytes plus the supplied GUST
license. It resolves only exact raw digests from a bounded explicit directory,
then uses existing Registry/PipelineCff APIs. It does not add a parser, subsetter
or writer. The single-font plain replay command is unchanged in behavior; its
source guard now names only the two examples it executes, so this independent
math probe does not invalidate that acceptance baseline.

```
cargo test --offline --manifest-path crates/rendering-core/Cargo.toml --test math_reference
cargo run --offline --manifest-path crates/rendering-core/Cargo.toml --example pipeline_fonts_probe -- display.json request.jsonl /path/to/explicit-font-directory GUST-FONT-LICENSE.TXT /tmp/math-original --searchable
```

`measurement.json` pins assets, source, output, reference, extractor and decoded
raster hashes. Rasterization uses existing `pdftoppm -r144 -singlefile -png`;
Pillow only compares the decoded RGB arrays. Nothing re-emits the reference PDF
as proof of original compilation.
