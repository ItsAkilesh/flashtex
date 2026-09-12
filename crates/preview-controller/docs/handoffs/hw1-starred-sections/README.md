# HW1 starred sectioning candidate

Exact base: `467538c77e84e4c3febc5308d49b53e1ca7f3d65`. This is an isolated candidate for the existing compiler owner; authoritative compiler files are unchanged by this branch.

The parser accepts a star before a braced section/subsection title, preserves the existing section/subsection counters and current label value for starred headings, and represents the heading with an empty number. Compiler layout omits empty heading numbers rather than emitting a blank text item and spacing. Ordinary section numbering remains unchanged. Consumers of Heading.number should treat the empty string as unnumbered; the render-pipeline owner must review its heading projection before repinning.

Tests cover counter/reset and preceding label preservation, two-argument macro titles with exact UTF-8 argument spans, and incremental/clean equality when stars are added and removed. Initial unmodified-base tests reproduced the brace errors. The first UTF-8 probe used Japanese text and also reported existing missing Core14 glyph warnings; the focused test uses accented Latin to isolate source-span behavior. No missing-glyph warning was suppressed.

This does not implement hfill, normalfont, preamble lengths, additional environments or full HW1 parity. Source/reference homework files remain immutable. Validation results will be recorded alongside the patch when the running suite completes.

Focused candidate validation PASS: 3 new starred-section tests and all 9 existing reference/figure tests. Full-suite process 81665 remains running at the existing scaling benchmark; 41 library tests passed before that benchmark. Do not claim full-suite success from this checkpoint.

Final validation: 41 library tests and the existing scaling benchmark passed (benchmark 101.57 s). The broad command then stopped at an omitted scratch `protocol/fixtures/compile-request.json`; this setup failure remains preserved in green.log. Restoring exact-base protocol and tex-corpus fixtures allowed all 49 integration tests to pass, with 3 existing ignored tests. Strict Clippy all-targets passed. The candidate patch also passes reverse-apply check against the tested scratch files. No test source or product source was changed to bypass the missing fixture. These are compiler candidate checks, not native or PDF parity acceptance.
