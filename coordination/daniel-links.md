# daniel-links handoff

Agent / task / branch: daniel-links (FT-037, revision 2) / typed,
source-identity-bound hyperlink destination, page-target, and rectangle
model for document export (`flashtex-link-annotations`) /
`agent/daniel-links/link-annotations`
State: ready for integration
Owned paths: `crates/link-annotations/**`, `coordination/daniel-links.md`
Rev 2 input main SHA (assignment base): `83f65e08ae60b312609f49ec0a2b6cae25c8e353`
Main integrated through (merge-base): `e5901797e8a7ebdd8d714ecdee6793e1097515a9`
Exact tested commit SHA (rev 2): `193b363c1d1f48d27380575d50174a5a4e87996e`
(branch `agent/daniel-links/link-annotations`; `cargo build`, `cargo test`,
`cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` were
all run against exactly this commit inside `crates/link-annotations`.)
Prior rev 1 tested commit SHA: `dd9e99329040fd4d4190e8772bcc99b36943fb84`
(URI allowlist module `src/uri.rs` has zero diff since that commit — the
rev 1 security work is preserved byte-for-byte in rev 2.)

## Typed contract

Standalone, additive crate. No dependency on any other FlashTeX crate and no
crate depends on it yet — nothing to migrate. Zero external dependencies.
Edition 2024, `publish = false`, matching the house style of
`crates/document-style`.

This crate models and validates hyperlink annotations. **It has no
navigation side effects**: it never opens, resolves, or fetches a URI or a
file, and performs no network or filesystem I/O anywhere in its code.

Public surface (re-exported from `flashtex_link_annotations`):

- `uri::{validate_uri, UriScheme, ValidatedUri, UriError, MAX_URI_LEN}`
  `validate_uri(&str) -> Result<ValidatedUri, UriError>` checks a URI's
  scheme against an **explicit allowlist** — `UriScheme::{Http, Https,
  Mailto}` — never a denylist. Any scheme not matched in
  `UriScheme::from_lowercase` (including `javascript:`, `data:`, `file:`,
  and anything unrecognized) fails as `UriError::SchemeNotAllowed`.
  Bounded: `text.len() > MAX_URI_LEN` (4096 bytes) is checked before any
  scan, so a hostile or absurdly long input fails immediately; the rest of
  validation is a single linear scan with no regex and no recursion.
  Rejects control characters (`UriError::ControlCharacter`), missing scheme
  (`UriError::MissingScheme`, e.g. protocol-relative `//host/path`), and
  syntactically invalid scheme tokens (`UriError::InvalidSchemeSyntax`).
- `target::{LabelId, LabelSet, InternalTarget, LabelError, TargetError,
  MAX_LABEL_LEN}`
  `LabelId::parse(&str) -> Result<LabelId, LabelError>` validates a label
  reference (non-empty, <= 256 bytes checked before any scan, no control
  characters). `InternalTarget::resolve(LabelId, &LabelSet) ->
  Result<InternalTarget, TargetError>` is the only way to build an
  `InternalTarget`; it returns `Err(TargetError::Unresolved(label))`
  whenever `label` is not present in the caller-supplied `LabelSet`. There
  is no code path that produces an `InternalTarget` for an unresolved
  label — never a silent dangling destination.
- `geometry::{Point, Rect, RectError}`
  `Rect::new(Point, width, height) -> Result<Rect, RectError>` rejects
  non-finite origins/extents and negative extents.
- `span::{SourcePos, SourceSpan, SpanError}`
  `SourceSpan::new(SourcePos, SourcePos) -> Result<SourceSpan, SpanError>`
  rejects an end offset preceding the start offset.
- `source::{RevisionId, RevisionError, ContentHash, SourceIdentity,
  SourceIdentityError, Staleness, MAX_REVISION_LEN}` **(new in rev 2)**
  `SourceIdentity::bind(RevisionId, source: &str, SourceSpan) ->
  Result<SourceIdentity, SourceIdentityError>` is the only way to build a
  `SourceIdentity`: it validates the span against the real `source` text
  (rejects an end past `source.len()`, rejects either offset that does not
  fall on a UTF-8 character boundary — `SourceIdentityError::NotCharBoundary`
  — so a range can never split a multi-byte character), then hashes exactly
  those bytes (FNV-1a, non-cryptographic, drift-detection only — not a
  security boundary). `SourceIdentity::check_fresh(&RevisionId, &str) ->
  Result<(), Staleness>` re-validates the same span against a *current*
  revision/source and returns a typed reason
  (`RevisionChanged`/`ContentChanged`/`RangeInvalid`) the moment either the
  revision id differs or the bytes at that exact range no longer hash the
  same — there is no path that reports a stale binding as fresh.
- `page::{PageIndex, PageTarget}` **(new in rev 2)**
  Pure data, no I/O: `PageTarget { page: PageIndex, rect: Rect }` is what a
  PDF exporter needs to build a `/GoTo` destination for a resolved internal
  link.
- `target::{LabelId, LabelSet, InternalTarget, LabelError, TargetError,
  MAX_LABEL_LEN}` **(rev 2: `LabelSet` now maps labels to `PageTarget`)**
  `LabelId::parse(&str) -> Result<LabelId, LabelError>` validates a label
  reference (non-empty, <= 256 bytes checked before any scan, no control
  characters). `LabelSet::insert(LabelId, PageTarget) -> Option<PageTarget>`
  registers where a label resolves to for export.
  `InternalTarget::resolve(LabelId, &LabelSet) -> Result<InternalTarget,
  TargetError>` is the only way to build an `InternalTarget`; it returns
  `Err(TargetError::Unresolved(label))` whenever `label` is not present in
  the caller-supplied `LabelSet`, and otherwise carries that label's
  `PageTarget`. There is no code path that produces an `InternalTarget` for
  an unresolved label — never a silent dangling destination.
- `geometry::{Point, Rect, RectError}`
  `Rect::new(Point, width, height) -> Result<Rect, RectError>` rejects
  non-finite origins/extents and negative extents.
- `span::{SourcePos, SourceSpan, SpanError}`
  `SourceSpan::new(SourcePos, SourcePos) -> Result<SourceSpan, SpanError>`
  rejects an end offset preceding the start offset. (Offset-only shape
  check; UTF-8 boundary and in-bounds checks against real source text now
  live in `source::SourceIdentity::bind`.)
- `annotation::{LinkAnnotation, LinkDestination}` **(rev 2: `span` field
  replaced by `source: SourceIdentity`)**
  `LinkDestination::{External(ValidatedUri), Internal(InternalTarget)}`;
  `LinkAnnotation { rect: Rect, source: SourceIdentity, destination:
  LinkDestination }` built via `LinkAnnotation::external(..)` /
  `::internal(..)`, both of which only accept already-validated/-resolved
  destinations and an already-bound `SourceIdentity`.

## Validation

`cd crates/link-annotations && cargo test` -> 58 unit tests (in `uri`,
`target`, `geometry`, `span`, `annotation`, `source`, `page`) + 8
integration tests (`tests/annotation.rs`) + 1 doctest, all passing. `cargo
clippy --all-targets -- -D warnings` -> 0 warnings. `cargo fmt --check` ->
clean.

Rev 2 tests (new):
- `source::tests::stale_when_revision_id_differs_even_if_bytes_are_identical`,
  `annotation::tests::annotation_source_is_detectably_stale_against_a_different_revision`,
  `tests/annotation.rs::source_identity_is_detectably_stale_against_a_different_revision_of_the_same_bytes`
  — a link bound at one revision is a hard, typed `Staleness::RevisionChanged`
  when checked against another, even with byte-identical source.
- `source::tests::stale_when_source_bytes_at_the_span_changed_under_the_same_revision`,
  `tests/annotation.rs::source_identity_is_detectably_stale_when_the_text_under_the_span_changes`
  — same revision id, edited bytes under the same span -> `ContentChanged`.
- `source::tests::rejects_span_that_splits_a_multibyte_character`,
  `source::tests::accepts_multibyte_span_aligned_on_character_boundaries`,
  `source::tests::stale_check_is_utf8_safe_when_multibyte_source_shifted`,
  `tests/annotation.rs::multibyte_source_ranges_never_split_a_character`
  — CJK (3-byte-per-character) source text; an offset landing inside a
  character is `SourceIdentityError::NotCharBoundary`, never truncated.
- `target::tests::resolves_known_label_and_carries_its_page_target`,
  `tests/annotation.rs::internal_link_resolves_against_document_labels_and_carries_a_page_target`
  — a resolved `InternalTarget` carries the exact `PageTarget` (page index +
  rect) its label was registered with.

Security-focused tests (allowlist, not denylist):
- `uri::tests::rejects_javascript_scheme`, `rejects_data_scheme`,
  `rejects_file_scheme`, `rejects_mixed_case_javascript_scheme` — each
  asserts `UriError::SchemeNotAllowed { scheme: .. }` for exactly that
  scheme; case-folding is proven not to open a bypass.
- `uri::tests::bounded_against_absurdly_long_hostile_input` — a 50,000,000
  byte string returns `UriError::TooLong` (asserted equal, with the actual
  length echoed back), proving the length gate runs before any scan rather
  than merely not crashing.
- `uri::tests::max_length_boundary_is_inclusive`,
  `rejects_uri_over_max_length` — exercise the `MAX_URI_LEN` boundary in
  both directions.
- `uri::tests::accepts_unicode_host_and_path`, `accepts_emoji_in_path`,
  `rejects_unicode_scheme` — Unicode is fine in the host/path, but a
  Unicode look-alike in the scheme token itself fails
  `InvalidSchemeSyntax` (scheme is ASCII-only per RFC 3986).
- `uri::tests::rejects_embedded_newline`, `rejects_embedded_nul` —
  malformed control-character input.
- `target::tests::fails_explicitly_on_unresolvable_label`,
  `does_not_resolve_against_unrelated_labels` — an unresolved or
  mismatched internal reference is `Err(TargetError::Unresolved(..))`
  containing the actual label, never a constructed `InternalTarget`.
- `target::tests::bounded_against_absurdly_long_label` — same length-gate
  argument for label ids.

All assertions above check concrete values (the returned error variant's
payload, `Rect::max_x`/`max_y`, `SourceSpan::len`, `UriScheme`, the actual
label string) rather than only that a call returned `Ok`/`Err`.

Incomplete behavior: no percent-encoding/normalization of URI paths (out of
scope — this crate validates the scheme and shape, not full RFC 3986
conformance of the rest of the URI); no host/path allowlisting beyond
scheme; `LabelSet` is a flat map supplied by the caller — this crate does
not itself discover or track document labels or page layout. The content
hash (FNV-1a) is drift-detection only, not collision-resistant — that is a
deliberate scope boundary, not a gap, since the allowlist (not the hash) is
the security control. No consumer wired yet.

## Needs from others

- An FT integration owner to decide which crate (likely `compiler` or a
  future PDF-annotation adapter) constructs `LinkAnnotation` values from
  parsed `\href`/`\url`/`\ref`/`\label` source, supplies the `RevisionId` +
  source text to `SourceIdentity::bind`, populates `LabelSet` with each
  label's `PageTarget` (page index from layout, rect from the label's
  bounding box), and calls `validate_uri` / `InternalTarget::resolve` at
  that boundary. This crate deliberately does not parse LaTeX, walk the
  document tree, or discover page layout itself.
- Confirmation of the scheme allowlist (`http`, `https`, `mailto`) is
  sufficient for the export targets in scope, or whether e.g. `tel:` should
  be added — adding a scheme is a one-line, explicit change in
  `UriScheme::from_lowercase`.
- Whether the eventual PDF exporter wants `check_fresh` invoked at export
  time (to detect an annotation computed against a now-stale document
  revision) or only as an internal consistency check during incremental
  recompilation — this crate exposes the check either way but does not call
  it itself.

Next action: await review/integration assignment; no other crate depends on
this one yet, so there is nothing to coordinate for a breaking change.

Resource: allocation `daniel-claude20x-shared`; timebox 40 minutes (rev 2).
Updated: 2026-09-12
