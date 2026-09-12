# daniel-links handoff

Agent / task / branch: daniel-links (FT-037 dispatch) / typed, source-mapped
hyperlink destination and rectangle model for document export
(`flashtex-link-annotations`) / `agent/daniel-links/link-annotations`
State: ready for integration
Owned paths: `crates/link-annotations/**`, `coordination/daniel-links.md`
Input main SHA (assignment base): `53fee3012b2902ca05bd31766defa515b3044cec`
Exact tested commit SHA: `dd9e99329040fd4d4190e8772bcc99b36943fb84`
(branch `agent/daniel-links/link-annotations`; `cargo build`, `cargo test`,
and `cargo clippy --all-targets -- -D warnings` were run against exactly
this commit inside `crates/link-annotations`, plus `cargo fmt --check`.)

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
- `annotation::{LinkAnnotation, LinkDestination}`
  `LinkDestination::{External(ValidatedUri), Internal(InternalTarget)}`;
  `LinkAnnotation { rect: Rect, span: SourceSpan, destination:
  LinkDestination }` built via `LinkAnnotation::external(..)` /
  `::internal(..)`, both of which only accept already-validated/-resolved
  destinations.

## Validation

`cd crates/link-annotations && cargo test` -> 41 unit tests (in `uri`,
`target`, `geometry`, `span`, `annotation`) + 5 integration tests
(`tests/annotation.rs`) + 1 doctest, all passing. `cargo clippy
--all-targets -- -D warnings` -> 0 warnings. `cargo fmt --check` -> clean.

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
scheme; `LabelSet` is a flat set supplied by the caller — this crate does
not itself discover or track document labels. No consumer wired yet.

## Needs from others

- An FT integration owner to decide which crate (likely `compiler` or a
  future PDF-annotation adapter) constructs `LinkAnnotation` values from
  parsed `\href`/`\url`/`\ref`/`\label` source and calls `validate_uri` /
  `InternalTarget::resolve` at that boundary. This crate deliberately does
  not parse LaTeX or walk the document tree.
- Confirmation of the scheme allowlist (`http`, `https`, `mailto`) is
  sufficient for the export targets in scope, or whether e.g. `tel:` should
  be added — adding a scheme is a one-line, explicit change in
  `UriScheme::from_lowercase`.

Next action: await review/integration assignment; no other crate depends on
this one yet, so there is nothing to coordinate for a breaking change.

Resource: allocation `daniel-claude20x-shared`; timebox 40 minutes.
Updated: 2026-09-12
