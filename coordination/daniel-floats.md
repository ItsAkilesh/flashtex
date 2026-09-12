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
