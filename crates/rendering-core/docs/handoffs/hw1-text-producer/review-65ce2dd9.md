# Published Text owner review: 65ce2dd9

Exact owner commit65ce2dd9933d0bed40fd083c6cd4b39b90899e7b. Read-only source and
published JSON comparison; no build, producer rerun, native or pixel execution.
Root separately reviews compiler/cache/source integration and narrowing bounds.
No duplicate implementation or authoritative vendor adoption.

The targeted mechanisms address the demonstrated gaps: mathtext::shape_run
reuses the existing paragraph Shaper on the actual resolved text face, whose T1
TFM path runs the shared ligature/kern program and binds resulting characters
through that face's cmap. Interword space is fontdimen2 (plus sentence extra
space where applicable), not OT1 slot32. Braces use the T1 text metrics. The
layout sees an Ord placeholder carrying the shaped hbox's full metrics, then
substitutes that hbox. Original font/GID identities survive the consumer output.

Independent comparison of published artifacts:
- All nine declared font/metric/license hashes match f261's packet. This is a
  declaration cross-check, not independent verification of the Mac filesystem.
- Five non-script before frames equal f261 JSON exactly. Five labels and comment
  joining are unchanged after the fix.
- Space now separates `a` and `b` runs and places b at144879963; braces retain
  GIDs38/116/39 with corrected T1 placement; ffi becomes GID123 with text `ffi`.
- Script host Roman8 versus our Math fallback is disclosed, so that case is not
  a controlled identical-resource comparison against Linux.

Concrete evidence correction: the owner README says scripts have no change
beyond environment. Within its OWN before/after pair superscript2 baseline_y
changes138994622→139117365 (+122743 ticks); hit/caret top changes133437038→133559782.
That is a real additional geometry change (~0.1175 TeX points). It may reflect
correct text-box metrics, but cannot be accepted as unchanged or reference parity.
A matching reference geometry/pixel gate should determine correctness.

Remaining scope limits:
- No text-specific pdflatex raster oracle is published; the owner's README
  explicitly acknowledges that. Metric assertions do not establish pixel parity.
- Space is now glue with no emitted text cluster, as in paragraphs. Searchable
  PDF/extraction and navigation across the separated runs need an explicit gate;
  concatenating run.text yields `ab`, not `a b`. This is an output representation
  change, not by itself proof of a PDF extraction failure.
- The adapter consumes only SGlyph.advance, not x_offset/y_offset. The pinned
  successful TFM path supplies zero offsets. Broader fallback acceptance must
  either handle nonzero offsets or explicitly refuse them; no such case was
  reproduced by the six current probes. TFM fallback retains existing notices.
- Multi-glyph clusters produce an empty text interval for later glyphs in this
  adapter; preserve the current strict consumer's supported-cluster/extraction
  constraints rather than assuming the one-glyph ffi gate covers all shaping.
- Root owns independent review of u16 run-index narrowing/private-use handle
  bounds and compiler feature/repin/cache compatibility; those are not cleared
  by this metric review.

Recommendation: targeted fixes have concrete source/artifact support, but hold
full Text/profile fidelity claims pending the script geometry correction,
searchable-space gate and matching pixel oracle. Whole-PDF bytes are not the goal.
