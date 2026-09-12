# Unchanged PDF subsetter uptake

Exact upstream20e5277857b2cd37f102fb07acdd164f82bb49db PDF tree is imported
unchanged; tree hash and empty-diff proof are in summary.json. No PDF-owner
implementation was edited. Additive V2Report/TJ/subroutine pruning APIs compile
with the existing consumer. Old fixture PDFs remain in their original paths;
new deterministic byte snapshots live here.

| Existing candidate | PDF bytes before | PDF bytes after |
|---|---:|---:|
| Plain | 29547 | 9595 |
| Inline math | 140238 | 11694 |
| Display math | 139720 | 11536 |
| Wrapping | 96447 | 76903 |
| Ligatures | 39690 | 19958 |
| Existing escaped-text regression | 27660 | 7238 |

The inline math embedded math program specifically falls109797→1753 bytes.
All shown original glyph identities and exact rational positions, declared
Unicode/width maps, and expanded glyph charstrings compare equal using the
PDF owner's existing reader, glyph_positions and CFF scanner. The compact CFF
charstring index is resolved through its charset to original CID/GID; it is
never treated as the original GID directly. No parser or subsetter is duplicated.

Poppler26.01 at144DPI gives byte-identical RGB before/after for each case, with
identical extracted text. Therefore the previous reference raster differences
remain0/602/1244/979/337, and the display-sum reference Unicode limitation remains.
This is measured subset preservation, not full TeX/native parity, a new corpus,
or a claim that byte-identical expanded programs prove every font property.
The escaped case is an existing regression, not another benchmark sweep.

Reproduce by building existing pipeline_fonts_probe and pdf_subset_compare
examples, exporting each unchanged display/request through its exact existing
font directory/license, then running `pdf_subset_compare OLD.pdf NEW.pdf REPORT`.
It accepts only bounded one-page fixed CID-CFF fixtures and reports errors for
unsupported input. It is not a general hostile-PDF validation API. Raster/text
commands are the same pdftoppm144DPI and pdftotext used by prior checkpoints.
The JSON files retain source/display/font/PDF/program/RGB hashes and resource
sizes; exporter evidence paths under temporary directories are audit artifacts,
not durable resource locations. `pdf_subset_uptake` tests all six immutable pairs.
