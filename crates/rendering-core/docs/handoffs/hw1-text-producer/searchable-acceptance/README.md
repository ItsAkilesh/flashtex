# Existing searchable exporter acceptance of published Text output

Consumes unchanged owner65ce2dd9 six after/display.json artifacts and original
f261 requests, through existing pipeline_cff_probe binary41d9cc1f (full hash in
evidence). Its source and PDF/font dependencies have no Git diff from tested
fce554f1. No producer rebuild, parser, writer, font substitution or native run.

Five single-font cases pass immutable CFF/source validation and export actual
PDF bytes. Installed Poppler26.01.0 extracts five labels, and, escaped braces and
ffi as expected. All produced PDF/display/input/resource hashes are retained.
Whole-expression source provenance remains separate from extracted literal text.

Whitespace finding: for both owner-after space output AND original f261-before
space output, pdftotext default and -layout return `ab`, while -raw returns `a b`.
The mode dependence predates the Text fix; it is not demonstrated new loss caused
by replacing the explicit space glyph with glue. No universal search/copy failure
is claimed. The PDFs preserve visibly positioned separate characters; actual
reference pixels and platform selection behavior have not been tested here.
Existing PDF/producer owners should decide the explicit searchable whitespace
policy and reproduce this case before adding marked content or changing spacing.

A one-byte source mutation under the same path is refused with
ValidationError("source identity mismatch") and produces no PDF. The scripts
case stops at the example's explicit one-font precondition, before PipelineCff;
it does not prove core multi-font incompatibility. Its owner artifact includes
host Roman8 beyond the isolated pinned three-font stage, so no substitute was used.

Commands: pipeline_cff_probe DISPLAY REQUEST FONT LICENSE PREFIX --searchable;
pdftotext [-layout|-raw] PREFIX.pdf OUTPUT. The examples require the current
immutable font/license bytes pinned in evidence. Compare actual geometry/pixels
for fidelity; PDF compression, metadata, object ordering and byte identity are
not acceptance criteria.
