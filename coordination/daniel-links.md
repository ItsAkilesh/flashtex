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
**Exact tested commit SHA: `4538dae3740c1dafeeb4555827837c2f523fb8b2`**
(branch `agent/daniel-links/link-annotations`, current tip; `cargo test`,
`cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` were
all run against exactly this commit inside `crates/link-annotations`, all
green. Verified on this branch with `git cat-file -e
4538dae3740c1dafeeb4555827837c2f523fb8b2^{commit}` and `git merge-base
--is-ancestor 4538dae3740c1dafeeb4555827837c2f523fb8b2 HEAD`.)
Rev 3 test-suite commit SHA (superseded by the tip above): `ea79df238095b57564d869cdffe10ef57aa17de1`
Prior rev 2 tested commit SHA: `193b363c1d1f48d27380575d50174a5a4e87996e`
Prior rev 1 tested commit SHA: `dd9e99329040fd4d4190e8772bcc99b36943fb84`

**This document was repaired on 2026-09-12.** An audit found that this file
said "see rev 2 notes below for the full API description" while the rev 3
commit (`3ff59769`) had deleted that entire section without replacing it,
leaving zero type or function signatures documented. The `## Typed
contract` section below is rebuilt from the crate's current source
(`crates/link-annotations/src/`) as of the exact tested SHA above, not
copied forward from rev 2 unchecked — every signature was individually
re-verified against source. See "What changed vs. rev 2" below for the
diff between what rev 2 documented and what the crate exposes now.

## Rev 3 objective

"Revision-bound link annotations: bounded adversarial and stale-identity
acceptance tests." Two new integration test files were added under
`crates/link-annotations/tests/`: `tests/adversarial.rs` and
`tests/staleness_acceptance.rs`. The rev 3 test-suite commit itself
(`ea79df23`) changed no production code — `src/**` was byte-identical to
rev 2 at that commit. **A further commit was then made on top of rev 3**
(see "Undocumented bugfix" below) that does change production code in
`src/span.rs`; the tested SHA above includes that commit.

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

## Undocumented bugfix (post-rev-3, found by audit mid-review)

Commit `4538dae3740c1dafeeb4555827837c2f523fb8b2` ("Fix silent u32 underflow
in `SourceSpan::len()` on inverted spans") landed on this branch after the
rev 3 test-suite commit and was not previously written up anywhere. What it
changed:

- **The bug:** `SourceSpan`'s `start`/`end` fields used to be `pub`. A
  caller could build `SourceSpan { start, end }` as a struct literal,
  bypassing the ordering check in `SourceSpan::new` entirely. Handing such
  an inverted span to `SourceSpan::len()` (`end.offset - start.offset`)
  then either panicked with "attempt to subtract with overflow" under
  `cargo test` (debug), or silently wrapped to a nonsense huge `u32` (e.g.
  4294967201) under `cargo test --release` — a wrong value with no error
  raised anywhere.
- **The fix:** `SourceSpan::start` and `SourceSpan::end` are now private.
  Two accessor methods were added — `start(&self) -> SourcePos` and
  `end(&self) -> SourcePos` — so `SourceSpan::new` is once again the *only*
  way to construct a `SourceSpan` from outside the module, restoring the
  `end.offset >= start.offset` invariant for every live value and making
  the subtraction in `len()` safe by construction rather than merely
  untested.
- **Blast radius:** `src/source.rs` and the test bodies in `src/annotation.rs`
  and `tests/annotation.rs` were touched only to switch `span.start.offset`
  / `span.end.offset` field access to `span.start().offset` /
  `span.end().offset` method calls — no behavior changed in those files.
  `src/uri.rs`, `src/target.rs`, `src/geometry.rs`, and `src/page.rs` are
  untouched by this commit.
- **Public API impact:** this is a real, breaking shape change to
  `SourceSpan` (fields → accessor methods). No consumer exists yet (see
  "Consumer situation" below), so nothing downstream needed updating, but
  the contract below reflects the new (current) shape, not the rev
  2/rev-3-test-suite shape.
- A regression test, `span::tests::inverted_span_at_u32_extremes_is_a_typed_error_not_a_wrapped_length`,
  now asserts the same boundary input returns a typed `SpanError::Inverted`
  in every build profile.

## What changed vs. rev 2's documented contract

Rev 2's handoff (`5edebe64`) documented a real, then-accurate contract for
its own tested commit (`193b363c`). Comparing it against the current
source (verified line-by-line, not assumed):

- **Still accurate, unchanged:** `uri::{validate_uri, UriScheme,
  ValidatedUri, MissingScheme/InvalidSchemeSyntax/SchemeNotAllowed/
  ControlCharacter variants}`, all of `target::{LabelId, LabelSet,
  InternalTarget, TargetError}`, all of `geometry::{Point, Rect,
  RectError}`, all of `page::{PageIndex, PageTarget}`, all of
  `annotation::{LinkAnnotation, LinkDestination}` (`src/uri.rs` is
  byte-identical since rev 1; `src/target.rs`, `src/geometry.rs`,
  `src/page.rs` are byte-identical since rev 2).
- **Omitted by rev 2, present in source both then and now:**
  `UriError::Empty` and `UriError::TooLong { len, max }` were never listed
  as named variants in rev 2's prose (rev 2 described the length bound in
  words but did not name the `TooLong` variant, and did not mention
  `Empty` at all). Both are real, live variants and are enumerated below.
  This is exactly the class of defect the audit is checking for — a
  consumer cannot `match` on a variant it was never told about.
- **Changed since rev 2 (the bugfix above):** `SourceSpan`'s `start`/`end`
  fields are no longer public; use `SourceSpan::start()` /
  `SourceSpan::end()` instead. Everything else in `source::*` (`RevisionId`,
  `RevisionError`, `ContentHash`, `SourceIdentity::bind`/`check_fresh`,
  `SourceIdentityError`, `Staleness`) is unchanged in shape since rev 2;
  only internal field-access syntax inside the crate was updated to match.
- **Not previously documented at all:** `LinkAnnotation::new` (the general
  constructor `external`/`internal` are built on top of), and the complete
  error-variant enumeration below.

## Typed contract

Standalone, additive crate. Zero external dependencies, edition 2024,
`publish = false`. Models and validates hyperlink annotations; performs no
navigation, network, or filesystem I/O anywhere. Every public item below
was checked directly against `crates/link-annotations/src/*.rs` at commit
`4538dae3740c1dafeeb4555827837c2f523fb8b2`; all of it is re-exported at the
crate root (`flashtex_link_annotations::*`), so callers never need to name
a submodule.

### `uri` — external URI validation (allowlist, not denylist)

```rust
pub const MAX_URI_LEN: usize = 4096;

pub enum UriScheme { Http, Https, Mailto }
impl UriScheme {
    pub fn as_str(&self) -> &'static str;
}

pub struct ValidatedUri; // fields private
impl ValidatedUri {
    pub fn scheme(&self) -> UriScheme;
    pub fn as_str(&self) -> &str;
}

pub enum UriError {
    Empty,
    TooLong { len: usize, max: usize },
    ControlCharacter { at: usize },
    MissingScheme,
    InvalidSchemeSyntax { scheme: String },
    SchemeNotAllowed { scheme: String },
}

pub fn validate_uri(text: &str) -> Result<ValidatedUri, UriError>;
```

`validate_uri` checks a URI's scheme against the explicit allowlist
`UriScheme::{Http, Https, Mailto}`, never a denylist: any scheme not
matched in `UriScheme::from_lowercase` (private) — including `javascript:`,
`data:`, `file:`, and anything unrecognized — fails as
`UriError::SchemeNotAllowed`. Order of checks: empty input is
`UriError::Empty`; `text.len() > MAX_URI_LEN` (4096 bytes) is
`UriError::TooLong`, checked before any scan; a control character anywhere
is `UriError::ControlCharacter { at }` at its byte offset; no `:` at all is
`UriError::MissingScheme`; a syntactically-invalid scheme token (not
`ALPHA *(ALPHA / DIGIT / "+" / "-" / ".")`, per RFC 3986) is
`UriError::InvalidSchemeSyntax`. Scheme matching is case-insensitive
(`HTTPS://...` is accepted as `Https`); case-folding is not a bypass of the
allowlist.

### `target` — internal (in-document) label references

```rust
pub const MAX_LABEL_LEN: usize = 256;

pub struct LabelId; // wraps a String, field private
impl LabelId {
    pub fn parse(text: &str) -> Result<LabelId, LabelError>;
    pub fn as_str(&self) -> &str;
}

pub enum LabelError {
    Empty,
    TooLong { len: usize, max: usize },
    ControlCharacter { at: usize },
}

pub struct LabelSet; // wraps HashMap<LabelId, PageTarget>, field private
impl LabelSet {
    pub fn new() -> LabelSet;
    pub fn insert(&mut self, id: LabelId, target: PageTarget) -> Option<PageTarget>;
    pub fn contains(&self, id: &LabelId) -> bool;
    pub fn get(&self, id: &LabelId) -> Option<&PageTarget>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
impl FromIterator<(LabelId, PageTarget)> for LabelSet { /* ... */ }

pub struct InternalTarget; // fields private
impl InternalTarget {
    pub fn label(&self) -> &LabelId;
    pub fn page_target(&self) -> PageTarget;
    pub fn resolve(label: LabelId, known: &LabelSet) -> Result<InternalTarget, TargetError>;
}

pub enum TargetError {
    Unresolved(LabelId),
}
```

`InternalTarget::resolve` is the *only* way to build an `InternalTarget`
(takes `label` by value, `known` by shared reference); it returns
`Err(TargetError::Unresolved(label))` whenever `label` is not present in
`known`, carrying the label back in the error — there is no code path that
produces an `InternalTarget` for an unresolved label. `LabelSet::insert`
follows `HashMap::insert` semantics (last registration for a given
`LabelId` wins; returns the previous `PageTarget`, if any).

### `geometry` — rectangles for the clickable area

```rust
pub struct Point { pub x: f64, pub y: f64 }
impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };
    pub fn new(x: f64, y: f64) -> Point;
}

pub struct Rect { pub origin: Point, pub width: f64, pub height: f64 }
impl Rect {
    pub fn new(origin: Point, width: f64, height: f64) -> Result<Rect, RectError>;
    pub fn max_x(&self) -> f64;
    pub fn max_y(&self) -> f64;
    pub fn is_empty(&self) -> bool;
}

pub enum RectError {
    NonFiniteOrigin { x: f64, y: f64 },
    NonFiniteExtent { width: f64, height: f64 },
    NegativeExtent { width: f64, height: f64 },
}
```

`Rect::new` rejects a non-finite origin (`NonFiniteOrigin`), a non-finite
width/height (`NonFiniteExtent`), or negative width/height
(`NegativeExtent`) — checked in that order. Never clamps or repairs bad
input.

### `span` — source positions and byte ranges (**shape changed post-rev-3**)

```rust
pub struct SourcePos { pub offset: u32, pub line: u32, pub column: u32 }
impl SourcePos {
    pub fn new(offset: u32, line: u32, column: u32) -> SourcePos;
}

pub struct SourceSpan; // start/end fields now PRIVATE (were pub through rev 3's test commit)
impl SourceSpan {
    pub fn new(start: SourcePos, end: SourcePos) -> Result<SourceSpan, SpanError>;
    pub fn start(&self) -> SourcePos;  // new post-rev-3
    pub fn end(&self) -> SourcePos;    // new post-rev-3
    pub fn len(&self) -> u32;
    pub fn is_empty(&self) -> bool;
}

pub enum SpanError {
    Inverted { start: SourcePos, end: SourcePos },
}
```

`SourceSpan::new` rejects `end.offset < start.offset` as
`SpanError::Inverted`. Because the fields are now private, this is the only
way to construct a `SourceSpan`, which is what makes `len()`'s
`end.offset - start.offset` panic-free/wrap-free by construction (see
"Undocumented bugfix" above). `SourcePos` itself is a plain, fully-public,
`Copy` struct: offset-only shape checking of the range lives here;
in-bounds and UTF-8-character-boundary checking against real source text
lives in `source::SourceIdentity::bind`.

### `source` — binding a value to an exact revision + byte range

```rust
pub const MAX_REVISION_LEN: usize = 256;

pub struct RevisionId; // wraps a String, field private
impl RevisionId {
    pub fn parse(text: &str) -> Result<RevisionId, RevisionError>;
    pub fn as_str(&self) -> &str;
}

pub enum RevisionError {
    Empty,
    TooLong { len: usize, max: usize },
    ControlCharacter { at: usize },
}

pub struct ContentHash; // wraps a u64 (FNV-1a), field private
impl ContentHash {
    pub fn value(&self) -> u64;
}

pub enum SourceIdentityError {
    SpanOutOfBounds { end: u32, source_len: usize },
    NotCharBoundary { offset: u32 },
}

pub enum Staleness {
    RevisionChanged { expected: RevisionId, found: RevisionId },
    RangeInvalid,
    ContentChanged,
}

pub struct SourceIdentity; // fields private
impl SourceIdentity {
    pub fn bind(revision: RevisionId, source: &str, span: SourceSpan) -> Result<SourceIdentity, SourceIdentityError>;
    pub fn revision(&self) -> &RevisionId;
    pub fn span(&self) -> SourceSpan;
    pub fn content_hash(&self) -> ContentHash;
    pub fn check_fresh(&self, current_revision: &RevisionId, current_source: &str) -> Result<(), Staleness>;
}
```

`SourceIdentity::bind` is the only way to build a `SourceIdentity`: it
rejects a span whose end runs past `source.len()`
(`SourceIdentityError::SpanOutOfBounds`) or whose start or end does not
fall on a UTF-8 character boundary in `source`
(`SourceIdentityError::NotCharBoundary`, checked at both the start offset
and the end offset independently), then hashes exactly the bytes in that
range with a non-cryptographic FNV-1a hash (drift detection only, not a
security boundary). `SourceIdentity::check_fresh` re-validates the same
span against a *current* `(revision, source)` pair: revision mismatch is
checked first and returns `Staleness::RevisionChanged` even if the bytes
are byte-identical; only if the revision matches does it re-slice and
re-hash, returning `Staleness::RangeInvalid` if the span no longer fits/
aligns, or `Staleness::ContentChanged` if it fits but hashes differently.
`Ok(())` only when the revision matches and the hash matches.

### `page` — exportable page + rectangle targets

```rust
pub struct PageIndex; // wraps a u32, field private
impl PageIndex {
    pub fn new(index: u32) -> PageIndex;
    pub fn value(&self) -> u32;
}

pub struct PageTarget { pub page: PageIndex, pub rect: Rect }
impl PageTarget {
    pub fn new(page: PageIndex, rect: Rect) -> PageTarget;
}
```

Pure data: what a PDF exporter needs to build a `/GoTo` destination for a
resolved internal link. No I/O, no page-count lookup.

### `annotation` — the link annotation itself

```rust
pub enum LinkDestination {
    External(ValidatedUri),
    Internal(InternalTarget),
}

pub struct LinkAnnotation {
    pub rect: Rect,
    pub source: SourceIdentity,
    pub destination: LinkDestination,
}
impl LinkAnnotation {
    pub fn new(rect: Rect, source: SourceIdentity, destination: LinkDestination) -> LinkAnnotation;
    pub fn external(rect: Rect, source: SourceIdentity, uri: ValidatedUri) -> LinkAnnotation;
    pub fn internal(rect: Rect, source: SourceIdentity, target: InternalTarget) -> LinkAnnotation;
}
```

`external`/`internal` are thin wrappers over `new` that only accept an
already-validated `ValidatedUri` or an already-resolved `InternalTarget` —
there is no "unvalidated" `LinkDestination` variant, and no way to build a
`LinkAnnotation` whose `source` was not built via `SourceIdentity::bind`.

## Public error variants (complete enumeration)

Every `pub enum` in the crate that represents an error or a status result,
enumerated in full against current source (8 enums, 22 variants total):

1. `UriError` (6): `Empty`, `TooLong { len, max }`, `ControlCharacter { at }`,
   `MissingScheme`, `InvalidSchemeSyntax { scheme }`, `SchemeNotAllowed { scheme }`
2. `LabelError` (3): `Empty`, `TooLong { len, max }`, `ControlCharacter { at }`
3. `TargetError` (1): `Unresolved(LabelId)`
4. `RectError` (3): `NonFiniteOrigin { x, y }`, `NonFiniteExtent { width, height }`,
   `NegativeExtent { width, height }`
5. `SpanError` (1): `Inverted { start, end }`
6. `RevisionError` (3): `Empty`, `TooLong { len, max }`, `ControlCharacter { at }`
7. `SourceIdentityError` (2): `SpanOutOfBounds { end, source_len }`,
   `NotCharBoundary { offset }`
8. `Staleness` (3): `RevisionChanged { expected, found }`, `RangeInvalid`,
   `ContentChanged`

(Two value enums that are not errors also exist and are re-exported:
`UriScheme { Http, Https, Mailto }` and `LinkDestination { External(_),
Internal(_) }` — listed here only so this is not mistaken for an omission.)

## Worked example

Two real, currently-passing pieces of code — not paraphrased — that
exercise the actual public signatures above. The first is this crate's own
module-level doctest (`src/lib.rs`, runs under `cargo test` as 1 of the 95
passing tests):

```rust
use flashtex_link_annotations::{
    LinkAnnotation, LinkDestination, Point, Rect, RevisionId, SourceIdentity, SourcePos,
    SourceSpan, validate_uri,
};

let source = "see \\href{https://example.com}{here} for more";
let rect = Rect::new(Point::new(72.0, 700.0), 120.0, 12.0).unwrap();
let span = SourceSpan::new(SourcePos::new(4, 1, 5), SourcePos::new(37, 1, 38)).unwrap();
let revision = RevisionId::parse("rev-1").unwrap();
let identity = SourceIdentity::bind(revision.clone(), source, span).unwrap();
let uri = validate_uri("https://example.com").unwrap();
let link = LinkAnnotation::external(rect, identity, uri);
assert!(matches!(link.destination, LinkDestination::External(_)));

// Checked against a different revision, the same bytes are still stale.
let other_revision = RevisionId::parse("rev-2").unwrap();
assert!(link.source.check_fresh(&other_revision, source).is_err());

// A denied scheme never becomes a link destination.
assert!(validate_uri("javascript:alert(1)").is_err());
```

The second exercises `LinkAnnotation::internal`, `LabelSet`, and
`InternalTarget::resolve`, extracted verbatim from
`tests/annotation.rs::internal_link_resolves_against_document_labels_and_carries_a_page_target`
(1 of 8 passing tests in that file):

```rust
use flashtex_link_annotations::{
    InternalTarget, LabelId, LabelSet, LinkAnnotation, LinkDestination, PageIndex, PageTarget,
    Point, Rect, RevisionId, SourceIdentity, SourcePos, SourceSpan,
};

let mut labels = LabelSet::new();
let results_page = PageTarget::new(
    PageIndex::new(2),
    Rect::new(Point::new(50.0, 600.0), 400.0, 20.0).unwrap(),
);
labels.insert(LabelId::parse("fig:results").unwrap(), results_page);

let target = InternalTarget::resolve(LabelId::parse("fig:results").unwrap(), &labels).unwrap();
assert_eq!(target.page_target(), results_page);

let rect = Rect::new(Point::new(50.0, 60.0), 30.0, 10.0).unwrap();
let source = "a".repeat(30);
let span = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(20, 1, 21)).unwrap();
let identity =
    SourceIdentity::bind(RevisionId::parse("rev-1").unwrap(), &source, span).unwrap();
let link = LinkAnnotation::internal(rect, identity, target);

match &link.destination {
    LinkDestination::Internal(target) => {
        assert_eq!(target.label().as_str(), "fig:results");
        assert_eq!(target.page_target().page.value(), 2);
    }
    LinkDestination::External(_) => panic!("expected internal destination"),
}
```

## Consumer situation

No other crate in this repository depends on `flashtex-link-annotations`
(package name) / `flashtex_link_annotations` (import name), as either a
real `[dependencies]` edge or a `[dev-dependencies]` edge. Checked with:

```
$ grep -rn "link-annotations\|flashtex_link_annotations" crates/*/Cargo.toml
```

Output: **empty** (no matches in any other crate's `Cargo.toml`, dependencies
or dev-dependencies alike). A broader content grep across the whole
worktree turns up references only inside this crate's own files
(`crates/link-annotations/{Cargo.toml,Cargo.lock,src/lib.rs,tests/*.rs}`)
and this lane's own coordination bookkeeping
(`coordination/daniel-links.md`, `coordination/agents/daniel-links.json`,
`coordination/next/daniel-links.json`, `coordination/queues/daniel-links.json`,
`coordination/assignments/FT-037.json`,
`coordination/completions/daniel-links/FT-037-r2.json`,
`coordination/machines/daniel-new.json`) — none of which is a Cargo
dependency edge. There is also no workspace-root `Cargo.toml` tying crates
together implicitly. **Conclusion: this crate has zero real consumers as of
this commit; the "no consumer wired yet" status from rev 1/rev 2 still
holds.**

## Validation

`cd crates/link-annotations && cargo test` -> **95 tests, all passing**:
59 unit tests (in `uri`, `target`, `geometry`, `span`, `annotation`,
`source`, `page` — 58 through rev 3's test commit, +1 from the post-rev-3
regression test `inverted_span_at_u32_extremes_is_a_typed_error_not_a_wrapped_length`)
+ 8 integration tests (`tests/annotation.rs`) + 21 integration tests
(`tests/adversarial.rs`) + 6 integration tests (`tests/staleness_acceptance.rs`)
+ 1 doctest (`src/lib.rs`).

`cargo clippy --all-targets -- -D warnings` -> 0 warnings.
`cargo fmt --check` -> clean.

Incomplete behavior (unchanged from rev 2): no percent-decoding or
normalization of URI paths (deliberately out of scope — this crate
validates scheme and shape, not full RFC 3986 conformance of the rest of
the URI); no host/path allowlisting beyond scheme; `LabelSet` is a flat map
supplied by the caller. The content hash (FNV-1a) is drift-detection only,
not collision-resistant — the allowlist, not the hash, is the security
control. No consumer wired yet (see "Consumer situation" above).

## Needs from others

Unchanged from rev 2: an FT integration owner to decide which crate
constructs `LinkAnnotation` values from parsed `\href`/`\url`/`\ref`/`\label`
source and wires `SourceIdentity::bind` / `check_fresh` into the actual
compile/recompile pipeline; confirmation the `http`/`https`/`mailto`
allowlist is sufficient for the export targets in scope; whether the PDF
exporter should call `check_fresh` at export time or only as an internal
consistency check. Additionally now: an integration owner should note that
`SourceSpan`'s field access changed to methods (see "Undocumented bugfix")
before writing any code against this crate.

Next action: await review/integration assignment; no other crate depends on
this one yet.

Resource: allocation `daniel-claude20x-shared`; timebox 30 minutes (rev 3).
Updated: 2026-09-12
