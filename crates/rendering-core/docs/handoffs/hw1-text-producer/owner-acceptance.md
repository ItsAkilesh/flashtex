# Handoff to existing mac-render-text-gaps owner

Owner ACK: GH2 comment5647033583, session a946f9a05b0f8ba2b,
branch agent/mac-render-pipeline/text-gaps from9aaec57a. Remote branch was not yet
published at this review checkpoint; an ACK is not an adopted fix.

Use renderer f261b36c `actual-candidate/` as the exact bounded acceptance packet:
- source-audit.json: original9aa producer, cumulative f464 compiler patch THEN
  separate61bd comment delta, three-arm9ce5 adapter, all383 exact source hashes;
- evidence.json: binary a13141bc3922e545b64947088a3e0c549c375628d83d3da9483d638f77c401cd,
  exact fonts/metrics hashes and cmap/source assertions;
- cases/{five,scripts,space,escaped,ligature,comment}: actual unchanged requests,
  raw runtime output and emitted v2. This is modified producer evidence.

Preserve plain labels and comment joining, original GIDs/full-font identity and
whole-expression source spans. Correct SPACE fontdimen placement rather than
slot32 width; bind brace text encoding explicitly; use existing TFM lig/kern or
an explicit supported text policy. Do not merely change extracted text. Scripts
must retain their actual resource-profile warnings until matching resources and
metrics are verified. Never relabel original recovered missing-license results.

The driver uses an isolated flat official metric stage with LICENSE copied under
required GUST-FONT-LICENSE.TXT, plus pinned Roman10/Roman12/LatinModernMath OTFs.
The README and source audit record stage assumptions; there is no host-install or
native-parity claim. Reuse existing owner resource interfaces; no duplicate local
spacing, shaping, font loader or parser implementation is being developed here.

Acceptance priorities: actual original-GID/font/source semantics, visibly correct
spacing/ligatures against matching reference pixels, explicit unsupported cases
and preserved diagnostic/currentness behavior. PDF container byte identity is
not acceptance. Publish exact source/vendor pins and outputs for independent
review before authoritative producer adoption.
