# Current helper geometry lookup

`BoundHelperCandidate::read_only_hit_test` reuses the existing exact `PageIndex`
against an immutable, fully source/font-validated `PipelineCff`. The private
constructor accepts that validated object; the existing public generic constructor
still validates its original profile. No public unchecked index is introduced.

The index is constructed lazily once per bound candidate, using only supplied
cluster rectangles/carets and rule geometry. Its residency is bounded by the
already validated display-list limits and released with the candidate; no global
cache is added. Every query compares the complete caller-supplied authoritative
`CurrentHelper` snapshot before accessing the index. Session, compile generation,
membership, editor versions, and source text remain separate freshness gates.

Actual f5524794 step0 contains the required hit geometry. Its ffi hit preserves
`main.tex[48..51]` and the producer's logical caret1. This does not identify a
particular raw TeX byte within the ligature. The existing escaped producer fixture
also retains the full two-byte `\%` span while returning logical caret0. That
second test uses explicitly synthetic helper correlation metadata around existing
producer geometry; it is not an actual helper/native transport replay.

A missing rectangle is not manufactured from glyph advances. A missing caret
retains whole-cluster selection. Returned hits grant no editing or navigation
permission and do not enable `source_actions`; consumers must requery using fresh
controller authority before later use. Multi-document native integration and
caret-to-TeX editing remain separate gates.
