# Proposed cluster ActualText boundary

Status: unimplemented consumer proposal; no protocol or PDF operator activation.
Owner API reviewed: PDF654f626 `v2::from_v2`/`ExactFont::cid_from_opentype`.

The current owner assigns each glyph its entire cluster text in ToUnicode. When
one cluster contains multiple glyphs, extracting each glyph can repeat that text.
When the same original GID appears with different cluster texts, the owner keeps
the first mapping and reports a conflict. Empty strings cannot recover text from
an invisible cluster. The current rendering-core adapter therefore accepts only
one glyph per nonempty cluster and one text per original GID/font. Real empty-ink
space glyphs are supported: ink absence is distinct from empty logical text.

A future owner-owned extension should expose typed balanced marked-content
operations wrapping one whole logical cluster: begin ActualText with exact UTF-16BE
text, paint all of that cluster's original GIDs at their existing absolute
positions, then end marked content. The text should be emitted once per cluster,
not once per glyph. Original GID subset identity and fallback ToUnicode remain
separate from occurrence-specific ActualText. Empty logical decoration should
carry explicit empty ActualText or an agreed artifact policy; an invisible
nonempty cluster needs a tested extraction representation, not fabricated ink.

Consumer requirements before opt-in:

- Keep stable page/item/cluster IDs, source path/hash/revision/UTF-8 ranges and
  original glyph identities attached to the whole group.
- Define visual paint order versus logical extraction order explicitly. Reject
  interleaved/noncontiguous cluster groups until an ordering policy is supported.
- Bound group count, nested marked-content depth, UTF-16 length and total output.
  Malformed/unbalanced groups, invalid Unicode and unsupported tags are errors.
- Teach the existing owner parser/writer/classifier the exact same operator set.
  Do not bypass the unsupported-content guard by using verbatim bytes.
- Preserve exact clip/paint/position and white export semantics. This extension
  changes extraction representation; it must not move glyphs or infer a font.

Acceptance should use actual exported PDF bytes and at least two independent
extractors, with repeated identical GIDs mapped to different source strings,
one source cluster containing multiple glyphs, combining marks, ligatures,
spaces, empty decoration and explicit RTL/reading-order cases. ToUnicode alone
must not be advertised as passing those cases. Raw byte, operator, extraction
and raster equality remain separate evidence.

Current tested supported case: an explicit consumer fixture with real unmodified
LM2.004 CFF bytes exports `H H fi`. The existing owner reader decodes the emitted
CID strings and ToUnicode to exactly those six source bytes; H/space repeat and
fi maps to one original ligature GID. This is a consumer test, not an original
LaTeX compilation or independent visual oracle. Actual current producer4888a67
still fails raw-font identity binding; no fixture repair is treated as original
output.
