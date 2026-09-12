# Searchable whitespace: exact ownership boundary

Read against rendering4614248e/fce554f1 unchanged consumer/PDF source and
published producer65ce2dd9. Actual mode evidence is647c50c5.

Owner-after `space/display.json` contains two GlyphRuns, logical texts `a` and
`b`, each with a whole-formula source span. There is no semantic whitespace
item/run/cluster. The positioned gap is geometry, not an unambiguous literal
space: math kerning and other TeX constructs can create the same gap. Reading
raw `\text{a b}` from that whole-expression span would require parsing TeX and
its macro context, so rendering-core must not infer the missing logical text.

Before/f261 explicitly supplies a space glyph/cluster and text `a b`.
PipelineCff::export_searchable uses original bytes and validates unambiguous
per-GID cluster text; PDF v2::from_v2 maps every supplied cluster substring into
ToUnicode, including that known space. It does not discard it. Nevertheless
Poppler default/layout returns `ab` for BOTH PDFs, while -raw returns `a b`.
This is not evidence that the new producer alone caused the observed mode gap.

Existing exact::Op exposes text matrices, ShowText and ShowTextArray, but no
marked-content/ActualText operators. Content::Verbatim still passes the same
bounded operator parser and is not a supported bypass. PipelineCff explicitly
refuses ambiguous per-GID or multi-glyph mappings requiring ActualText. Thus
there is no already-supported semantic-whitespace metadata/facility that this
rendering consumer can safely switch on as a local fix.

Required owner work if a stronger extraction guarantee is chosen:
1. Existing producer owner must supply explicit logical run/group text and
   occurrence-to-source association for glue/ligatures; no inferred spaces or
   GID reinterpretation. Whole-expression navigation provenance remains separate.
2. Existing PDF owner must choose and implement bounded occurrence-level text
   facilities (such as marked ActualText) with exact geometry unchanged and an
   explicit capability/unsupported path. Producer-supplied semantic text must be
   validated before use; no guessed reconstruction from raw TeX.
3. Existing reference/validation lane should test default/layout/raw extraction
   separately and pixel equality on matched references; do not change positions,
   font size or glyph identities merely to satisfy extractor heuristics.

No local rendering fix is justified by current metadata. No new parser/writer,
wire fields, marked operators or native activation were added in this review.
