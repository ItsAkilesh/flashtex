# Isolated HW1 membership evidence review

Pinned3354e31381eb3a9e6db68eeecdc326a437152e69. Read-only script
`tools/audit_hw1_membership.py` verifies the four membership evidence hashes,
records compressed/original request and result hashes, and checks immutable HW1
sourcef725e23897df3c9645bde1ceaab0876d78ea6ef4a44fd0e80d60a5161fac9d9e.
The diagnostic multiset changes119→107 with exactly12 errors removed and none
added. Each removed source span equals one of12 emitted membership items and
slices the exact UTF8 command `\in`. Other107 diagnostics remain; this is not a
whole-document compatibility or PDF fidelity result.

Provenance limit: the available shared source request has ID
`hw1-starred-acceptance`, while the membership output has ID `hw1-membership`.
The owner confirms original membership stdin was generated but not retained, and
its standalone binary was rebuilt in place for the combined candidate, so its
original binary hash is unavailable. This audit does not substitute/normalize the
shared request into exact candidate stdin proof. It establishes artifact hashes,
source-span and diagnostic relationships, not an independently reproducible exact
binary/request execution. Combined0d3bfdfc evidence is separate and not audited
here. No rerun, compiler edit, production adoption or Symbol/PDF parity claim.
