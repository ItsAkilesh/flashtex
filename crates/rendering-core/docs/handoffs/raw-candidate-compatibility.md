# Raw candidate compatibility — FT023 revision 12

The rendering consumer retains the original envelope payload and helper display
bytes with serde `RawValue` until existing typed serde validation. This is an
intentional stricter rejection of malformed inputs, not a new wire profile or
native activation. Existing byte, source, revision and immutable resource checks
remain authoritative. The helper caller still supplies current controller state;
event metadata cannot grant source-navigation authority.

## Reproduction and accepted representations

Run `cargo test --manifest-path crates/rendering-core/Cargo.toml --test helper_candidate`.
The tests use the published b9240b0 requested runtime sibling and 8876279 helper
source-state fixtures already pinned in `tests/fixtures/runtime-candidates`.
Raw, normalized unique-field JSON and equivalent escaped Unicode produce exactly
the same searchable PDF bytes. Revision 9007199254740991 remains accepted with
matching source snapshots. Greater values, floating/exponent integer tokens,
nonfinite coordinates and invalid Unicode are refused.

[Issue #32](https://github.com/flash-tex/flashtex/issues/32) tracks this correction.

The actual first font accepts a duplicate `glyph_count: 0` followed by its real
value under the previous Value intermediary: the invalid field was discarded
before typed validation. The new path refuses that input, escaped equivalent
field names, duplicate glyph coordinates, duplicate helper policy fields and
duplicate decoded source-version keys. The helper source map is bounded to 256
entries. Unknown helper extension fields remain ignored as before; this does not
claim blanket duplicate-key rejection for unknown extension data.

Normalizing a malformed duplicate `id` object into Value and serializing it first
irreversibly erases the conflict. The resulting unique-field JSON is accepted:
a downstream validator cannot reconstruct discarded bytes. Runtime must reject
its own typed correlation/source duplicates before transformation and retain
opaque render bytes through the helper so the rendering owner can reject nested
resource/geometry duplicates. No raw transport is activated by this change.

Helper whole-event Value validation preserves the old finite-number and depth
checks even in ignored extension fields; it is never serialized or used for
binding. The wire module retains its existing Value inspection only to produce unsupported
primitive diagnostics; final acceptance decodes the original RawValue payload.
This work makes no parser-speed or native paint claim. The pinned runner was deliberately rebased to b797b21a and rerun on all five
existing fixtures. Original PDF hashes and raster/text metrics exactly match the
previous subset20 baseline; reference pixel mismatches remain 0/602/1244/979/337.
The display-math reference ToUnicode limitation remains explicit. Evidence:
`../../tools/evidence/raw-payload-five-fixtures.json` (relative to crate root: `tools/evidence/`).

Validation checkpoint: 169 rendering tests passed (2 explicitly ignored); strict
all-target Clippy passed. After its borrow-only lint correction, focused helper7
and wire6 regression tests passed before publication. Runner exit3 is the expected
existing reference mismatch, not a newly failed duplicate-field correction.
