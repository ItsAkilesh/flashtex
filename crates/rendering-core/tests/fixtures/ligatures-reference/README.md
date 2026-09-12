# Existing ligature fixture

Actual unchanged65dbe7d output for existing18-ligatures, using the pinned rooted
LM2.004 metrics, LM12 OTF and license. Diagnostics are empty. Source/request,
reference engine metadata, PDFs and word boxes are preserved without reference
geometry substitution.

Poppler26.01/144DPI1224×1584: **337 differing RGB pixels**, equal extracted text
and identical word membership on three lines. No visual/kerning parity follows.
Original GIDs122/123/124/125/126 map ff/ffi/ffl/fi/fl respectively, with original
input intervals and exact positions/advances retained in measurement.json.
For example office's ffi remains one GID123 with source bytes34..37; waffle's
ffl remains one GID124 with bytes81..84. Repeated uses keep the same visible text
mapping. No character-code-to-GID cast or replacement glyph is introduced.

The existing PDF reader verifies original ToUnicode values and reference
Type1 codes27..31 mapping ff/fi/fl/ffi/ffl. Unlike the display-sum reference, these
specific mappings are present; reference code30 is not original GID123. The
linear text comparison is meaningful within this limited scope, not proof of
accessibility structure or roundtrip TeX. Existing ambiguous/empty mapping
refusal gates remain unchanged. Exact source geometry is retained, but reference
kerning equality and per-pixel causes are not established.

Run `replay_original.py --fixture ligatures --metrics-root ROOT --report FILE`
with the pinned tools as described in tools/README.md. Expected actual outcome
is mismatch/exit3, not acceptance. The Rust ligatures_reference test verifies
GIDs, multi-byte source intervals and both declared text maps using existing
PDF APIs. No new writer, font parser or renderer is used.
