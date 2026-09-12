# daniel-links handoff

Agent / task / branch: daniel-links (FT-037, revision 3) / typed,
source-identity-bound hyperlink destination, page-target, and rectangle
model for document export (`flashtex-link-annotations`) /
`agent/daniel-links/link-annotations`
State: ready for integration
Owned paths: `crates/link-annotations/**`, `coordination/daniel-links.md`
Rev 3 input main SHA (assignment base): `8e7546dfb4721c780036f03c21b14e6002062142`
(as recorded in `coordination/assignments/FT-037.json`)
Main integrated through (merge-base with `origin/main` at setup-time fetch):
`0294d2ca48cbfd497c6b549deb7575fbcec4a4b8` (this is the exact `origin/main`
second parent of the setup merge commit `69b8d2d7`; the local `origin/main`
remote-tracking ref has since advanced further via other agents' pushes
without a re-fetch on this branch, so it is not re-reported here as
reviewed/integrated content that was not actually pulled in).
Exact tested commit SHA (rev 3): `ea79df238095b57564d869cdffe10ef57aa17de1`
(branch `agent/daniel-links/link-annotations`; `cargo test`,
`cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` were
all run against exactly this commit inside `crates/link-annotations`.)
Prior rev 2 tested commit SHA: `193b363c1d1f48d27380575d50174a5a4e87996e`
Prior rev 1 tested commit SHA: `dd9e99329040fd4d4190e8772bcc99b36943fb84`
(`src/uri.rs` and `src/source.rs` are unchanged since rev 1/rev 2 respectively —
the rev 1 allowlist and rev 2 binding/staleness logic are preserved
byte-for-byte in rev 3; this revision is additive test coverage only.)

## Rev 3 objective

"Revision-bound link annotations: bounded adversarial and stale-identity
acceptance tests." No production code changed — `src/**` is byte-identical
to rev 2. Two new integration test files were added under
`crates/link-annotations/tests/`.

### `tests/adversarial.rs` — bounded adversarial acceptance

Attacks `validate_uri` (scheme/URI shape) and the source-binding path
(`SourceSpan::new`, `SourceIdentity::bind`), asserting the *exact* typed
error variant for each case, never a panic or a hang:

- Scheme with unusual casing plus an embedded space (fails
  `InvalidSchemeSyntax`, since a space is not a control character and
  survives the earlier control-character scan) and plus an embedded tab or
  other control character (caught earlier, as `ControlCharacter { at }`,
  at its exact byte offset).
- A scheme-like word with no colon at all (`javascript`, `https`, `mailto`,
  `data`, `file`) — all `MissingScheme`.
- Nested/repeated schemes: an allowed outer scheme (`http://...`) whose
  path contains a denied scheme-like word followed by its own colon is
  accepted as the outer scheme (the first colon in the string governs, full
  stop); the reverse nesting (`javascript://http:...`) is still rejected as
  `SchemeNotAllowed { scheme: "javascript" }`.
- Percent-encoded content that would decode to a blocked scheme
  (`%6A%61vascript://...`, and the fully-encoded form of `javascript:`):
  both fail `InvalidSchemeSyntax` because the raw `%` characters are not
  valid scheme syntax — **this function never decodes anything, so
  percent-encoding is not a bypass in either direction** (before the first
  colon it just fails syntax; after it, it's inert path content that's
  never re-inspected). See the module doc comment for the full answer.
- URI length: exactly `MAX_URI_LEN` bytes of an otherwise-valid `https`
  URI is accepted; one byte past it is `TooLong { len, max }`.
- An empty label (`LabelError::Empty`) and a 50,000,000-byte label
  (`LabelError::TooLong`, length bound checked before any scan).
- Byte ranges: inverted (`SpanError::Inverted`), past the end of the source
  (`SourceIdentityError::SpanOutOfBounds`), and landing mid-character at
  either the start or the end offset (`SourceIdentityError::NotCharBoundary`,
  both directions exercised separately).
- Rectangles with NaN width, negative-infinity height, negative width and
  height together, and a non-finite origin — each asserted against the
  precise `RectError` variant.

### `tests/staleness_acceptance.rs` — stale-identity acceptance suite

Written as a readable specification (module doc comment carries the full
matrix) of the `SourceIdentity::check_fresh` staleness contract, covering
every way a source can change relative to a bound identity:

| how the source changed | `check_fresh` result |
|---|---|
| revision id only | `Err(RevisionChanged { expected, found })` |
| content only (same revision) | `Err(ContentChanged)` |
| both revision and content | `Err(RevisionChanged { .. })` — revision is checked first, before the span is ever re-hashed |
| bound range no longer fits/aligns | `Err(RangeInvalid)` (both the "shrunk past the end" and "multibyte insertion misaligns a boundary" causes are exercised) |
| nothing changed | `Ok(())` |

The "both changed → `RevisionChanged`" row is called out explicitly as a
deliberate precedence contract, not an incidental implementation detail.

## Typed contract (unchanged from rev 2)

Standalone, additive crate. Zero external dependencies, edition 2024,
`publish = false`. Models and validates hyperlink annotations; performs no
navigation, network, or filesystem I/O anywhere.

Public surface (re-exported from `flashtex_link_annotations`): unchanged —
see rev 2 notes below for the full API description. This revision adds no
new public items; it only adds test coverage against the existing surface.

## Validation

`cd crates/link-annotations && cargo test` -> 58 unit tests (`uri`,
`target`, `geometry`, `span`, `annotation`, `source`, `page`) + 8
integration tests (`tests/annotation.rs`) + 21 integration tests
(`tests/adversarial.rs`, new in rev 3) + 6 integration tests
(`tests/staleness_acceptance.rs`, new in rev 3) + 1 doctest = **94 tests,
all passing**.

`cargo clippy --all-targets -- -D warnings` -> 0 warnings.
`cargo fmt --check` -> clean.

Incomplete behavior (unchanged from rev 2): no percent-decoding or
normalization of URI paths (deliberately out of scope — this crate
validates scheme and shape, not full RFC 3986 conformance of the rest of
the URI); no host/path allowlisting beyond scheme; `LabelSet` is a flat map
supplied by the caller. The content hash (FNV-1a) is drift-detection only,
not collision-resistant — the allowlist, not the hash, is the security
control. No consumer wired yet.

## Needs from others

Unchanged from rev 2: an FT integration owner to decide which crate
constructs `LinkAnnotation` values from parsed `\href`/`\url`/`\ref`/`\label`
source and wires `SourceIdentity::bind` / `check_fresh` into the actual
compile/recompile pipeline; confirmation the `http`/`https`/`mailto`
allowlist is sufficient for the export targets in scope; whether the PDF
exporter should call `check_fresh` at export time or only as an internal
consistency check.

Next action: await review/integration assignment; no other crate depends on
this one yet.

Resource: allocation `daniel-claude20x-shared`; timebox 30 minutes (rev 3).
Updated: 2026-09-12
