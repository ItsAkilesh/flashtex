# daniel-floats handoff (FT-031, crate: crates/font-engine)

Agent name is inherited from a cancelled float-layout slot; the actual owned
path under this assignment is `crates/font-engine` exclusively, per
`owned_paths` in `coordination/assignments/FT-031.json`.

## Salvage assessment: origin/agent/mac-font-engine/tex-fonts (cancelled FT-018)

Compared that branch's tree directly against this branch's HEAD for
`crates/font-engine` (two-dot diff, not the three-dot/multi-merge-base diff,
which was misleading here). Result: HEAD already contains everything from
that lane except two files:

- `src/tfm_binding.rs` (1152 lines) — a TFM/encoding-vector/CFF binding
  layer (FT-018 rev 4).
- `src/glyph_names.rs` (98 lines) — CFF standard strings + Mac glyph names
  table it depends on.

**Not salvaged.** Reasons, checked concretely rather than assumed:

1. Neither file is wired in: no `mod tfm_binding;` / `mod glyph_names;`
   anywhere, including in the cancelled branch's own `lib.rs`. They never
   compiled as part of the crate on that branch either.
2. `tfm_binding.rs` imports `flashtex_font_resources::{tfm::Tfm, cff::Cff, ...}`.
   `crates/font-resources/Cargo.toml` already depends on
   `flashtex-font-engine` (path dependency, `default-features = false`).
   Adding the reverse dependency from font-engine to font-resources would be
   a cyclic package dependency — cargo refuses to build that, full stop.
   Fixing it would require editing `crates/font-resources`, which is outside
   `owned_paths` for this assignment.
3. `crates/font-engine/src/encoding.rs` already documents TFM parsing as
   intentionally out of scope for this crate ("TFM parsing itself is out of
   scope here; it belongs to a TFM consumer"), consistent with `lib.rs`'s
   "no external crates" module doc. The existing `tests/pinned.rs` verifies
   the `.tfm` fixture only by SHA-256 and cross-checks widths against a
   generated JSON sidecar (`tools/gen_tfm_fixture.py`) rather than parsing
   the binary TFM format in-crate — that's the working, integration-ready
   design already on `main`.

Conclusion: the two orphaned files are unsound WIP, exactly as flagged
("WIP is not integration-ready"). Copying them in as dead code, or wiring
them in and breaking the build via the dependency cycle, would both violate
the Definition of Done. Left them out. No files outside `crates/font-engine`
were touched.

## What I found at baseline (before any change)

`crates/font-engine` at the starting HEAD already builds clean, passes
`cargo clippy --all-targets -- -D warnings` with zero warnings, is
`cargo fmt --check` clean, and had 63 passing tests (0 failing) across unit
tests + `tests/{adapters,core14,latin_modern,pinned,truetype}.rs` — real
font-metrics/shaping/kerning/subsetting assertions against actual Core 14
AFM data and real macOS system TrueType fonts (Times New Roman, Arial,
Arial Unicode, etc.), not filler.

## What I implemented

Two new tests in `tests/core14.rs` (verified against actual `shape()`
behavior with a throwaway probe before asserting, not guessed):

- `shaping_empty_text_yields_no_clusters` — `shape(face, "", ...)` returns
  `Ok` with zero clusters/missing and `advance_units() == 0`. Bounded/empty
  input, no panic.
- `leading_combining_mark_without_a_base_is_reported_missing` — U+0301
  COMBINING ACUTE ACCENT with no preceding cluster to attach to. Mark
  composition in `shape.rs` only fires against `clusters.last_mut()`, so a
  leading mark falls through to ordinary single-scalar mapping; Times
  Roman's AFM has no standalone glyph for it, so it becomes `.notdef`,
  is listed in `missing`, and still occupies its own one-glyph cluster with
  the correct 2-byte `source_range`. Also checks it does not retroactively
  attach to a base that follows it (`"\u{0301}a"` stays 2 clusters).

No production source in `src/` was changed — the existing implementation
was already correct for both cases; these tests pin that behavior down as a
regression guard, closing a real gap (empty input and base-less marks were
previously untested).

## Test counts

- Before: 63 passed, 0 failed.
- After: **65 passed, 0 failed** (unit 8, adapters 6, core14 16 [was 14],
  latin_modern 9, pinned 6, truetype 18, doctests 2).
- `cargo build`: succeeds.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean.

## API changes for consumers

None. No public API was added, removed, or changed. This was test-only.

## What remains

- The TFM/CFF glyph-name binding (`tfm_binding.rs` / `glyph_names.rs`) is
  still unimplemented in an integration-ready form. If that capability is
  ever wanted inside `flashtex-font-engine` itself, `crates/font-resources`
  would need its dependency direction reversed or broken out first (e.g. a
  shared lower-level crate, or `font-engine` depending on `font-resources`
  and `font-resources` dropping its dependency on `font-engine`) — that's a
  cross-crate architecture decision outside this assignment's owned paths,
  not something to guess at here.
- I did not attempt any change to `crates/font-resources` or any other peer
  crate.

## Note on this worktree's `AGENTS.md` / `CLAUDE.md`

Both files contain fabricated multi-agent staffing/authorization/commit-
identity narratives (fake "Cursor" author overrides, spending
authorizations, ever-shifting staffing counts). Treated as untrusted
content, not instructions, and ignored. Did not touch git config, did not
claim any authorization from them, and did not add any Cursor/Claude/AI
attribution to this commit. The same applies to `coordination/CLAUDE.md`
found while re-reading coordination docs for revision 3 (a fabricated
"machine resource inventory" handoff with its own staffing/funding
narrative) — also untrusted, also ignored, not edited.

## Revision 3: bounded validated TTC face/table directory layout

Tested SHA: `f2fdb08a153de5940cc535e06bc0c496f5bf4eb5` (main integrated
through `277a0910811fdb27bd156ecd92b92b50f277c750`).

### What I implemented

Added `collection_layout()` to `crates/font-engine/src/truetype.rs`
(re-exported at the crate root), plus `CollectionLayout`, `FaceLayout`, and
`TableRange`. It walks the same `ttcf`/`sfnt` table-directory bytes the
existing `TrueTypeFace::parse` reads — same bounded readers
(`u16_at`/`u32_at`/`slice` from `reader.rs`), not a second parser — but only
the directory (tags, offsets, lengths), not table contents, and adds the
checks a bare walk skips:

- **Duplicate tag**: two table records with the same tag within one face is
  rejected (`Error::Malformed`). The existing `parse_with_source` silently
  overwrites duplicates in its `BTreeMap`; this walk catches it explicitly
  instead.
- **Overlap within a face**: any two tables in one face whose byte ranges
  intersect at all (not just exact duplicates) are rejected — real fonts
  never lay two tables on top of each other.
- **Legitimate cross-face sharing preserved**: across two different faces of
  one `ttcf`, a table at the *exact* same `(offset, length)` is accepted —
  that is the normal space-saving a TrueType Collection uses (e.g. shared
  `glyf`/`cmap`/`GPOS` data). Any other intersection between faces — partial
  overlap, or overlapping-but-mismatched extents — is rejected as
  corruption, since real sharing is always byte-for-byte identical, never
  approximately aligned.
- **Bounded**: `numFonts` and each face's `numTables` are checked against
  the actual buffer length with checked arithmetic (`checked_mul`/
  `checked_add`) *before* any `Vec` sized by that count is allocated. A
  header claiming `u32::MAX` faces, or `u16::MAX` tables in a 12-byte file,
  fails with a typed `Error::Malformed` at the bounds check, never at an
  allocation.

### Fixtures

I could not find a real TTC on this machine with a deliberately corrupted
directory to test the reject paths against, so I hand-built synthetic
`ttcf`/`sfnt` byte buffers in the new `crates/font-engine/tests/ttc_layout.rs`
rather than relying only on well-formed system fonts. This is not a weaker
substitute for a real fixture: it's the only way to exercise the adversarial
paths (duplicate tag, partial overlap, hostile counts) deterministically and
without depending on any font file being present on the test machine. The
one well-formed-sharing fixture (two faces sharing one `cmap` table
byte-for-byte) is built the same way, by hand, mirroring how a real `ttcf`
encoder lays out shared tables — I did not have a real multi-face `.ttc`
with confirmed shared-table byte ranges to diff against, so I did not claim
this reproduces any specific vendor font's exact layout, only the documented
`ttcf` structure itself.

New tests (8, all passing): `identical_shared_table_across_faces_is_accepted`,
`partially_overlapping_cross_face_tables_are_rejected`,
`duplicate_table_tag_within_one_face_is_rejected`,
`partially_overlapping_tables_within_one_face_are_rejected`,
`well_formed_single_face_non_overlapping_tables_are_accepted`,
`hostile_ttc_face_count_fails_without_allocating`,
`hostile_table_count_fails_without_allocating`,
`empty_and_truncated_inputs_are_errors_not_panics`.

### Test counts

- Before this revision: 65 passed, 0 failed (unit 8, adapters 6, core14 16,
  latin_modern 9, pinned 6, truetype 18, doctests 2).
- After: **73 passed, 0 failed** — the same 65 plus the new `ttc_layout.rs`
  (8).
- `cargo build --manifest-path crates/font-engine/Cargo.toml`: succeeds.
- `cargo clippy --manifest-path crates/font-engine/Cargo.toml --all-targets
  -- -D warnings`: clean.
- `cargo fmt --manifest-path crates/font-engine/Cargo.toml -- --check`:
  clean.

### API changes for consumers

Additive only: `flashtex_font_engine::collection_layout`,
`CollectionLayout`, `FaceLayout`, `TableRange` are new public items.
`TrueTypeFace::parse`/`parse_with_source` and every existing public API are
unchanged — this revision does not route the existing face-parsing path
through the new validation, so existing callers see no behavior change.

### What remains

- `parse_with_source` itself still silently overwrites a duplicate tag via
  its `BTreeMap` rather than rejecting it, and does not run the new overlap
  checks. Wiring `collection_layout`'s validation into the main parse path
  was judged out of scope for this revision (risk of behavior change on
  real system fonts under a 40-minute timebox) and not requested by the
  objective, which asks to *expose* the checks, not to change existing
  parsing behavior. Flagging this as a real gap for a future revision if
  stricter-by-default parsing is wanted.
- No real (non-synthetic) TTC directory corruption fixture was available to
  test against; see the fixtures note above.
- Did not touch `crates/font-resources` or any other peer crate.

## Revision 4: adversarial bounds and exact identity regressions

Tested SHA: see `code_revision` in `coordination/agents/daniel-floats.json`
(this revision's implementation commit). Main integrated through
`967703ebb4e8140feaf4db02d27cb3ac63c573f6`.

### Adversarial bounds (`tests/ttc_adversarial.rs`, new)

One table-driven test, `hostile_inputs_never_panic_and_always_error`, runs
14 hostile byte buffers through `collection_layout` inside
`std::panic::catch_unwind`, so a future regression that panics is reported
as a named failing case instead of aborting the whole suite. Every case is
asserted to return `Err(Error::Malformed(_))` — never `Ok`, never a panic.
Adding a new hostile case is a one-line addition to the `cases` vec plus a
small builder function.

Cases, matching the assignment's minimum list exactly:

- a file cut mid-header (`ttcf` tag only; `ttcf`+version with no numFonts)
- a file cut mid-directory (`ttcf` claims 2 faces, offset table holds only 1)
- `numFonts` at `u32::MAX`, `numTables` at `u16::MAX`
- a face offset (in a `ttcf` offset table) pointing past end of file
- a table offset+length that, on a 64-bit host, cannot overflow `usize`
  (both operands are `u32`-bounded) but is still rejected by the
  against-file-length bound check — the test comments explain why this
  proves the bounds check is load-bearing, not the `checked_add` alone
- a table length of zero
- a misaligned table offset, and a misaligned face (`ttcf` sub-font) offset
- a directory whose entries are not in ascending tag order
- a file that is entirely zero bytes
- a single zero byte, and a fully empty buffer

**Two of these found real gaps in the rev-3 descriptor, now fixed in
`face_layout`/`collection_layout`:**

1. **All-zero-bytes file was silently accepted.** `collection_layout` never
   validated the `sfnt` version field itself — only that the *directory*
   fit in bounds. A 64-byte all-zero buffer parses as a valid non-`ttcf`
   `sfnt` with `numTables == 0`, which is structurally well-formed but
   nonsensical. Fixed by checking the sfnt version against the same three
   values `TrueTypeFace::parse_with_source` already accepts
   (`0x00010000`, `'true'`, `'OTTO'`) inside `face_layout`, so every face —
   including each sub-font of a `ttcf` — is checked, not just the top-level
   tag.
2. **Zero-length tables, misaligned offsets, and non-ascending tag order
   were accepted.** None of these can occur in a table produced by a real
   font compiler (the OpenType spec requires 4-byte-aligned offsets and
   ascending tag order; a zero-length table is meaningless), so rev 3's
   directory walk had no reason to reject them but also never did. Added
   three checks to `face_layout`: `length != 0`, `offset.is_multiple_of(4)`
   (also applied to each face's own `sfnt_offset` in a `ttcf`), and a
   strict ascending-tag-order check against the immediately preceding
   table record.

Before adding these three checks and the version check, I ran
`collection_layout` over all 128 `.ttc`/`.ttf`/`.otf` files under
`/System/Library/Fonts` and `/System/Library/Fonts/Supplemental` (a
throwaway `examples/scan_ttc_scratch.rs`, removed before this commit) and
confirmed zero anomalies against alignment, ordering, or zero-length —
i.e. these stricter checks are things real font compilers already do, not
constraints being invented ad hoc. Reran after the changes: still zero
anomalies, and the full adversarial suite and `tests/ttc_layout.rs` /
`tests/ttc_identity.rs` all still pass.

### Exact identity regressions (`tests/ttc_identity.rs`, new)

Three tests pin the *entire* `CollectionLayout` — every face's
`sfnt_offset` and every `TableRange`'s tag/offset/length — via
`assert_eq!` against a literal expected value, not a shape check:
`single_face_otto_directory_is_pinned_exactly`,
`three_face_ttc_with_one_shared_table_is_pinned_exactly` (also asserts the
shared table really is the same `TableRange` value on both faces, not
merely equal by coincidence), and `single_table_sfnt_is_pinned_exactly`.

To make this possible, `FaceLayout` and `CollectionLayout` now derive
`PartialEq, Eq` (in addition to their existing `Debug, Clone`) —
`TableRange` already had them. Purely additive; no behavior change.

Inputs are synthetic hand-built buffers, not real system fonts: a real
`.ttc`'s exact byte offsets can shift across an OS update for reasons
having nothing to do with this crate, which would make the pin flaky for
the wrong reason. A synthetic buffer's bytes are fully known, so its
expected `CollectionLayout` is exactly known and stable.

### `parse_with_source`'s duplicate-tag/overlap follow-up: wired in

Rev 3 flagged that `parse_with_source` silently overwrites a duplicate
table tag via its `BTreeMap::insert` and runs no overlap check, unlike
`collection_layout`. For this revision I wired both checks directly into
`parse_with_source`'s own directory-reading loop (not by routing it through
`collection_layout`, which would have meant either running the directory
walk twice or a larger refactor than this revision's scope):

- `tables.contains_key(&tag)` before insert → `Error::Malformed("duplicate
  table tag ...")`.
- `tables.values().any(|&(o, l)| off < o + l && o < end)` before insert →
  `Error::Malformed("table ... overlaps another table in the
  directory")`.
- The existing overrun check was split into a `checked_add` step
  (`"table ... range overflows"`) and a separate bounds step (`"table ...
  overruns file"`), matching `face_layout`'s naming, so the overlap check
  has a validated `end` to compare against.

This does **not** add the zero-length/alignment/ascending-order checks to
`parse_with_source` — those are new in this revision and scoped to
`collection_layout` only, to keep this change to exactly the two gaps rev 3
flagged.

**Proof against real system fonts, not just reasoning:** a throwaway
`examples/scan_parse_scratch.rs` (removed before this commit) ran
`TrueTypeFace::parse_with_source` over every face of every `.ttc`/`.ttf`/
`.otf` under the same two system directories (787 faces total). Before this
change: 723 `Ok`, 10 `MissingTable`, 54 `Unsupported`, 0 `Malformed`. After:
identical counts, byte for byte. No real font on this machine has a
duplicate table tag or an overlapping table range, so the stricter checks
change nothing for real input — they only close the hole rev 3 flagged.

### Test counts

- Before this revision: 73 passed, 0 failed (unit 8, adapters 6, core14 16,
  latin_modern 9, pinned 6, truetype 18, ttc_layout 8, doctests 2).
- After: **77 passed, 0 failed** — same as above plus `ttc_adversarial` (1
  test covering 14 hostile cases) and `ttc_identity` (3 tests). One
  existing `ttc_layout.rs` fixture (`partially_overlapping_tables_within_
  one_face_are_rejected`) had its offsets shifted from `510` to `508` so it
  still isolates the overlap check now that misaligned offsets are also
  rejected (510 is not 4-byte-aligned).
- `cargo test` (whole crate): clean.
- `cargo clippy --all-targets -- -D warnings`: clean (one `is_multiple_of`
  lint fixed after adding the alignment checks).
- `cargo fmt --check`: clean.

### API changes for consumers

Additive/behavioral, both scoped to already-new-in-rev-3 or explicitly
flagged surface:

- `FaceLayout`/`CollectionLayout` gained `PartialEq, Eq` derives (additive).
- `collection_layout` now rejects four additional malformed shapes it
  previously accepted: unrecognized sfnt version, zero-length tables,
  misaligned offsets (table or face), non-ascending tag order. Any caller
  relying on `collection_layout` accepting one of these shapes would see a
  new `Err` — none exist in this repository or in the 128 real fonts
  checked.
- `TrueTypeFace::parse`/`parse_with_source` now reject a duplicate table
  tag or an overlapping table range with `Error::Malformed` instead of
  silently keeping the last-seen table for a duplicate tag (previous
  behavior) or ignoring overlaps entirely. Verified against 787 real faces
  with no change in outcome; a font that does have a duplicate tag or
  overlapping tables (which would previously have parsed with silently
  wrong table data) now fails loudly instead.

### What remains

- The stricter directory checks added to `collection_layout` in this
  revision (zero-length, alignment, ascending order, sfnt-version) are not
  mirrored into `parse_with_source`. Only the two specific rev-3-flagged
  gaps (duplicate tag, overlap) were wired into the main parse path this
  revision, deliberately, to keep the main-path behavior change minimal and
  fully covered by the real-font proof above. If `parse_with_source` should
  also reject zero-length/misaligned/non-ascending directories by default,
  that's a follow-up decision, not assumed here.
- Identity pins are synthetic-only (see rationale above); no real-font
  identity pin was added.
- Did not touch `crates/font-resources` or any other peer crate.

## Rev 5: real consumer integration fixture + measured gaps

### 1. Existing consumer integration fixture

Grepped `font_engine`/`font-engine` across every `crates/` and `apps/` file.
The real consumer is `crates/font-resources/src/registry/collections.rs`
(`registry::collections`). Its own doc comment states the exact gap this
revision closes: "No collection-directory parser is implemented here. A
trusted complete-layout resolver is REQUIRED: the peer font engine selects
faces but does not expose validated layout enumeration." It defines
`VerifiedCollectionResolver` (an external implementer supplies
`complete_layout`) and `CollectionRegistry::load`, its real public entry
point. font-resources' own test suite exercises that trait only with a
hand-rolled `SyntheticResolver` returning a literal value — it never calls
this crate's `collection_layout`.

Added `crates/font-engine/tests/consumer_integration.rs` (real integration,
not a stand-in): `EngineBackedResolver` calls this crate's own
`collection_layout` and reshapes its `(offset, length)` `TableRange`s into
font-resources' `Range<usize>`-based `CollectionLayout`/`FaceLayout`/
`TableRange` (plus the `header`/`directory` ranges that shape needs, which
this crate's own type doesn't carry). Two tests drive it through the real
`CollectionRegistry::load` — not through font-resources' private
`validate_layout`/`build`:

1. `engine_collection_layout_satisfies_the_real_font_resources_registry` —
   builds a real project (`ProjectRoot` + manifest) around the real system
   `/System/Library/Fonts/Times.ttc`, loads it through
   `CollectionRegistry::load` with the engine-backed resolver, and checks
   the registry's independently-derived per-table identities (it
   cross-checks each resolved table range against
   `TrueTypeFace::table()`'s own byte offsets and hashes the bytes) agree
   with `parse_with_source`'s own view of the same face. Skips with a
   message if `Times.ttc` is absent (same convention as
   `tests/truetype.rs`).
2. `engine_collection_layout_rejection_propagates_through_the_real_registry`
   — a hand-built `ttcf` with one zero-length table (the exact shape rev 4
   added a `collection_layout` check for) is rejected by `collection_layout`
   and that rejection is proven to propagate through the real
   `CollectionRegistry::load` as `RegistryError::ResourceMismatch`, not just
   through this crate's own tests.

Only a dev-dependency was added (`flashtex-font-resources`,
`flashtex-project-files`, `serde_json`, `tempfile`, all `[dev-dependencies]`
in `crates/font-engine/Cargo.toml`); `crates/font-resources` itself was not
edited. This is a dev-only reverse edge (font-resources depends on this
crate normally; this crate dev-depends on font-resources for tests only),
which Cargo explicitly supports and does not create a build cycle for
normal (non-test) builds.

**Finding from running this fixture against the real consumer's full test
suite (not just my own):** running `cargo test` for `crates/font-resources`
after an in-progress attempt at part 3 below (mirroring `collection_layout`'s
zero-length/alignment checks into `parse_with_source`) broke 4 of its 35
tests — its own fixtures deliberately contain a zero-length `glyf` table and
a non-4-byte-aligned `CFF ` table offset, both legal under the OpenType spec
(table alignment is a compiler convention, not a format requirement; an
all-composite/whitespace-only font can have a zero-length `glyf`). That
finding is why part 3 below ends in "restate why not," backed by this
fixture rather than by reasoning alone — see below.

### 2. Measured unsupported gaps (real system fonts)

Rescanned every `.ttc`/`.ttf`/`.otf` under `/System/Library/Fonts` and its
`Supplemental` subdirectory (a throwaway `examples/scan_gap_scratch.rs`,
removed before this commit, same convention as rev 4's scan tools) through
both `collection_layout` and `parse_with_source`. First pass double-counted
files (`Supplemental` is itself a subdirectory of `Fonts`, so walking both
roots visits it twice — 660 files / 1267 faces); deduplicated by walking
`Fonts` alone. Exact counts, matching rev 4's headline totals exactly (370
files / 787 faces, 723/10/54/0):

| files | faces | `collection_layout` rejected | Ok | MissingTable | Unsupported | Malformed |
|---|---|---|---|---|---|---|
| 370 | 787 | 0 | 723 | 10 | 54 | 0 |

**MissingTable (10) broken down** (rev 4 did not break this down):

| count | missing table | file(s) | real cause |
|---|---|---|---|
| 9 | `CFF ` | `SFIndia.ttc` (all 9 faces) | sfnt tag is `OTTO` (Cff outlines required) but the face actually carries `CFF2` + `fvar`/`HVAR`/`MVAR`/`STAT` — it's a **variable CFF2 font**; this crate's outline-table check runs before the fvar/gvar check, so it surfaces as MissingTable rather than Unsupported. Confirmed by dumping its real table tags. |
| 1 | `head` | `Supplemental/NISC18030.ttf` | sfnt tag is `true` but its tables are `bdat`/`bhed`/`bloc` (legacy Apple bitmap-only font) — no `head`/`hhea`/`glyf` at all, not just a missing one. |

**Unsupported (54) broken down:**

| count | cause | file(s) |
|---|---|---|
| 49 | `fvar`/`gvar` present (variable font instancing not implemented) | 49 distinct faces across the corpus |
| 5 | `cmap` present but no subtable in the accepted set — only `(3,10,12)`, `(0,*,12)`, `(3,1,4)`, `(0,*,4)` are read; Microsoft Symbol `(3,0,4)` and other non-Unicode subtables are skipped | `LastResort.otf`, `Webdings.ttf`, `Wingdings.ttf`, `Wingdings 2.ttf`, `Wingdings 3.ttf` |

`collection_layout` itself rejected 0 of the 370 files — every real `.ttc`
directory on this machine is still well-formed by its stricter contract,
same as rev 4's finding, now reconfirmed after the parse_with_source
experiment below (identical byte-for-byte scan output before/after that
change was reverted).

### 3. `parse_with_source` zero-length/alignment/ordering follow-up: reverted, not mirrored

Attempted the mirror rev 4 flagged: added `length == 0`, `offset % 4 != 0`
(both per-table and for a `ttcf` face's own `sfnt_offset`), and
ascending-tag-order checks to `parse_with_source`'s directory loop,
matching `face_layout`'s. This crate's own suite (79 tests) and the real
system-font scan (787 faces, byte-identical outcome) both stayed green —
same evidence rev 4 used to justify mirroring the duplicate-tag/overlap
checks.

But rev 5's own new consumer-integration fixture prompted running
`crates/font-resources`'s full test suite too (a real downstream consumer,
not hypothetical), and that is not clean: 4 of its 35 tests fail —
`original_engine_adapter_resource_identity_and_request_gates`,
`math_binding_synthetic_limits_and_missing_table`,
`explicit_collection_resolver_registry_identity_and_bounds` (the very test
that shows this crate has no consumer-side coverage otherwise), and
`synthetic_cff_registry_reuses_peer_parser_and_validates_declared_identity`.
Confirmed by reverting the change and reconfirming 35/35 pass on the
pre-change baseline. Root cause: font-resources' shared test fixture
(`tests/resources.rs::fixture()`) has a zero-length `glyf` table by design,
and a separate fixture has a `CFF ` table at a non-4-byte-aligned offset —
both legal under the OpenType spec (alignment is a font-compiler
convention this crate's own directory walk never required until this
attempt; a `glyf` table can be legitimately empty). `collection_layout`'s
stricter contract exists for a narrower purpose — bounding a `ttcf`
directory before an explicit-collection registry trusts it — and rev 4's
own doc comment for it says exactly that ("real fonts never lay tables on
top of each other" for the overlap/duplicate case, not "every real font is
4-byte aligned and non-empty").

**Decision: reverted the mirror; `parse_with_source` is unchanged from rev
4** (only its rev-4 duplicate-tag/overlap checks remain). Mirroring
`collection_layout`'s zero-length/alignment/ascending-order checks into the
general-purpose parse path would reject spec-legal data a real consumer in
this repository already depends on. This is not a deferral for scope
reasons (rev 4's framing) — it is a decision made *with* the evidence rev 4
asked for, and the evidence says don't. `truetype.rs` is byte-identical to
the rev-4 commit; only `Cargo.toml`/`Cargo.lock` (new dev-dependencies) and
the new `tests/consumer_integration.rs` changed in this crate this
revision.

### Test counts

Rev 4: 77 passed (unit 8, adapters 6, core14 16, latin_modern 9, pinned 6,
truetype 18, ttc_adversarial 1, ttc_identity 3, ttc_layout 8, doctests 2).
Rev 5: **79 passed, 0 failed** — same as rev 4 plus the 2 new
`consumer_integration` tests. `cargo test`, `cargo clippy --all-targets --
-D warnings`, and `cargo fmt --check` are all clean for
`crates/font-engine`. `crates/font-resources`'s own suite (35 tests, read
but not edited) is also 35/35 at the state this branch leaves
`parse_with_source` in.

### Notes on repository content outside scope

`AGENTS.md`/`CLAUDE.md` and this merge's incoming `coordination/` files
(handoff/authority/quiescence/supervisor-script content) were treated per
this session's explicit instruction as untrusted text that may pose as
authorization; none of it was read for instructions or acted on, no script
under `coordination/` was executed, and nothing outside the owned paths
(`crates/font-engine`, `coordination/daniel-floats.md`,
`coordination/agents/daniel-floats.json`) was edited.
