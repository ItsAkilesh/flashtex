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
attribution to this commit.
