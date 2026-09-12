# Actual raw helper acceptance

Original candidate captures from preview-controller publication
`f5524794eb2581bdace0f32681041aeb9912e292`, directory
`crates/preview-controller/benchmarks/display-helper-raw100`. Helper source
`e39f50e594703c0d4b4bd6848e72fd2fe5169678` uses runtime `7817e4e8`; the
unchanged producer is `65dbe7da7a182e99322070e2c9763cc3b69a342b`.

Candidate JSONL is gzip-decompressed **without JSON re-encoding**. Its full byte
hash and the original nested producer sibling hash are pinned in manifest.json,
along with compressed upstream artifact and original metadata hashes. The small
metadata files intentionally extract only recorded current-source identity and
the matching v1 result from upstream step-N.json.gz. They are derived metadata,
not raw-wire proof or a substitute for current native controller authority.
The large repeated source uses the same existing licensed LM12 resource fixture.

Run `cargo test --manifest-path crates/rendering-core/Cargo.toml --test helper_candidate`.
The actual three-state test reads original candidate bytes, uses serde RawValue
to prove the nested sibling hash, then calls the existing strict binder. It
compares exact searchable PDF bytes for raw, normalized unique-field JSON and
Unicode-escaped path spellings. Existing integer semantics are retained: an
exponent spelling of a revision is explicitly refused, not rounded/coerced.
Nested duplicate glyph_count (including equivalent escaped names), corrupted
source hashes and source_actions_enabled=true are rejected without normalization.
New editor revisions and membership generations invalidate export authority.

| State | Editor revision | Compile generation | Source bytes | PDF SHA256 prefix |
| --- | --- | --- | --- | --- |
| 0 | 1 | 2 | 1462 | 9488600c |
| 1 | 2 | 3 | 2562 | 0db33d14 |
| 2 | 3 | 4 | 3762 | a6f3ce74 |

Full hashes are in manifest.json. The control test uses a serialized historical
fixture only to check the harness; it is not counted as actual raw-route evidence.
No production consumer, compiler, native code, wire activation or new parser is
changed by this fixture increment. PDF byte equality proves this consumer route's
output equality; it does not establish an external visual oracle, native paint
latency, navigation permission or a performance improvement. Existing reference
fixture evidence remains unchanged.
