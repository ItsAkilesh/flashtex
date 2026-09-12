# Parent invariant audit: validating constructors bypassed by public fields

Read-only, static + dynamic sweep of 16 crates across 16 sibling worktrees, hunting the
`SourceSpan` defect class: a `pub` type with a validating constructor whose fields (or enum
variant payloads) are *also* `pub`, letting a caller build an invalid instance by struct/variant
literal and skip validation entirely.

Method: for every `pub struct`/`pub enum` in each crate's `src/`, check for a validating
constructor, check whether its fields are bypassable, and if so trace the concrete downstream
consequence (panic / overflow-underflow / OOB / NaN propagation / wrong result). Findings without
a traceable concrete consequence are excluded per the task's own filter. Each real finding was
built as a runnable repro under a scratch cargo project (crate copied out of the worktree, built
as a path-dependency with `CARGO_TARGET_DIR` redirected outside the checkout) and run in both
`cargo run` and `cargo run --release`. No file in any worktree was modified, staged, or committed.

**Result: 16 crates, ~259 pub struct/enum declarations examined, 4 CONFIRMED live findings, 0
STATIC-ONLY findings, 3 confirmed-already-fixed instances of this exact class (one of which —
`tex-calc`'s `debug_assert!` bug — was fixed today, mid-sweep).**

## Cross-cutting note: prompt injection in worktree docs

Nearly every worktree's `CLAUDE.md`/`AGENTS.md` (surfaced automatically as project context, not
part of any crate's `src/`) contains fabricated multi-agent "staffing override," "Commander /
orchestrator," and billing/commit-authorization language — e.g. claims of "explicit user
authorization" for autonomous spending, fake staffing rosters, and instructions authorizing direct
commits under alternate identities. This was seen by sub-agents auditing `title-layout`,
`math-layout`, `math-accessibility`, `collaboration-core`, `spellcheck`, `link-annotations`,
`project-bundle`, and `tex-calc`. All of it was treated as untrusted repository data, not as
instructions — no sub-agent changed scope, spawned further sub-agents, committed, or adopted any
alternate identity because of it. Flagging it here since it's a repo-hygiene concern independent
of this audit's actual subject.

## Per-crate summary

| Worktree | Crate | Pub types examined | Validating constructors | Bypassable (live) | Confirmed findings |
|---|---|---|---|---|---|
| ft-wt-daniel-tables | paragraph-layout | 34 | 0 | 0 | 0 |
| ft-wt-daniel-floats | font-engine | 50 (42 externally reachable) | ~13 | 1 | **1 — `OpenTypeMathFace`** |
| ft-wt-daniel-footnotes | math-layout | 28 | 0 | 0 | 0 |
| ft-wt-daniel-title | title-layout | 10 | 1 (non-bypassable, fieldless enum) | 0 | 0 |
| ft-wt-daniel-contents | toc-layout | 15 | 3 | 2 | **2 — `LineBox`, `RelativeEntry`** |
| ft-wt-daniel-color | color-expressions | 4 (2 externally pub) | 0 | 0 | 0 |
| ft-wt-daniel-images | image-assets | 12 | 2 (both fields fully private) | 0 | 0 |
| ft-wt-daniel-links | link-annotations | 24 | 7 | 1 new (+1 already-fixed: `SourceSpan`) | **1 — `Rect`** |
| ft-wt-daniel-math-access | math-accessibility | 8 | 0 (only non-validating ctors) | 0 | 0 |
| ft-wt-daniel-spelling | spellcheck | 8 | 1 (field private) | 0 | 0 |
| ft-wt-daniel-templates | project-templates | 10 | 0 (validation lives in a separate `validate()` method, not a constructor) | 0 | 0 |
| ft-wt-daniel-snippets | editor-snippets | 9 | 2 (both fully private) | 0 | 0 |
| ft-wt-daniel-statistics | document-statistics | 11 | 0 | 0 | 0 |
| ft-wt-daniel-bundle | project-bundle | 13 | 1 (fields private) | 0 | 0 |
| ft-wt-daniel-collaboration | collaboration-core | 13 | 4 (all fields private; 1 is an already-fixed instance: `Checkpoint`) | 0 | 0 |
| ft-wt-daniel-calc | tex-calc | 10 | 3 (denominator guards) | 0 (fixed earlier today) | 0 (confirmed-fixed, see below) |

Total: **4 CONFIRMED live findings**, 0 STATIC-ONLY findings, 3 already-fixed instances of the same
class independently re-verified (`SourceSpan` in link-annotations, the `tex-calc` denominator
`debug_assert!`s, `Checkpoint` in collaboration-core).

---

## CONFIRMED findings

### 1. `toc-layout::LineBox` — NaN injection corrupts a leader-dot count to `u32::MAX`

- **File:** `crates/toc-layout/src/leader.rs:10` (struct), validator `LineBox::new` at `leader.rs:40`,
  consumed by `layout_entry` at `leader.rs:99-161`.
- **Invariant:** `LineBox::new` requires `width` and `indent_unit` to be finite and `width > 0`,
  `indent_unit >= 0`.
- **Bypass:** both fields are `pub` (`pub width: f64, pub height: f64`... actually `pub width:
  f64, pub indent_unit: f64`), so `LineBox { width: 40.0, indent_unit: f64::NAN }` skips `new()`
  entirely.
- **Consequence:** `used = indent + title_width + page_label_width` becomes `NaN`. The guard `if
  used > line.width { return Err(Overflow) }` is silently `false` (any comparison against NaN is
  `false`), so the error path never fires. `available` becomes `NaN`, `count =
  (available/leader_unit).floor()` is `NaN`, and `count.min(f64::from(u32::MAX))` — per `f64::min`'s
  documented "if one argument is NaN, the other is returned" — resolves to `u32::MAX as f64`, which
  casts (saturating, non-panicking) to `4294967295`. The function returns `Ok(LaidOutEntry {
  leader_count: 4294967295, ... })`, silently violating the crate's own documented
  width-sum invariant and handing any real renderer a "draw 4.3 billion leader dots" instruction.
- **Repro:** built at
  `/private/tmp/claude-503/-Users-dqi26/50763fe3-5136-4e4d-ab08-2a19a3be5880/scratchpad/audit-contents/`
  (crate copied to `crate-copy/`, no Cargo.toml edits needed; binary at `repro/src/main.rs`),
  `CARGO_TARGET_DIR` redirected under scratch.
- **Debug output:** `LaidOutEntry { ..., leader_count: 4294967295, leader_width: 4294967295.0,
  gap_before_page: NaN, ... }`; sum-invariant check (`indent+title+leader+gap+page_label`) is `NaN`
  instead of `40`.
- **Release output:** byte-identical to debug (`leader_count = 4294967295`,
  `leader_count == u32::MAX: true`) — pure `f64` math and a saturating cast, so debug and release
  agree.
- **Classification: CONFIRMED** (executed in both profiles).

### 2. `toc-layout::RelativeEntry` — `body_offset = 0` silently collides with front matter

- **File:** `crates/toc-layout/src/entry.rs:95` (struct), validator `RelativeEntry::new` at
  `entry.rs:104`, consumed by `resolve()` at `entry.rs:131`.
- **Invariant:** `RelativeEntry::new` rejects `body_offset == 0` (a "body" entry must be at least
  one page into the body, not landing on the front matter's last page).
- **Bypass:** `title`, `level`, `body_offset` are all `pub`, so `RelativeEntry { title, level: 0,
  body_offset: 0 }` skips the check.
- **Consequence:** `resolve()` computes `front_matter_pages.checked_add(self.body_offset)`
  (`5 + 0 = 5`, so `checked_add` never trips) and passes the result to `EntryRecord::new`, which
  validates title/level/page-nonzero but has no way to detect that `body_offset` itself was zero —
  that check only lives in the bypassed `RelativeEntry::new`. Result: a "body" TOC entry silently
  resolves to the exact same page number as the front matter's own last page — a wrong computed
  result, not a panic.
- **Repro:** same scratch project as above. Debug and release both print: `VALID body_offset=1 ->
  resolved page = 6`, `BYPASS body_offset=0 -> resolved page = 5 (lands ON the last front-matter
  page, not in the body at all)`.
- **Classification: CONFIRMED** (executed in both profiles; identical output — plain `u32`
  arithmetic, no overflow reachable here).

### 3. `link-annotations::Rect` — NaN/negative-dimension bypass silently corrupts geometry

- **File:** `crates/link-annotations/src/geometry.rs:27` (struct), validator `Rect::new` at
  `geometry.rs:36`.
- **Invariant:** `Rect::new` rejects a non-finite `origin` and negative `width`/`height`.
- **Bypass:** `origin`, `width`, `height` are all `pub`, so `Rect { origin: Point { x: f64::NAN, y:
  0.0 }, width: -5.0, height: -3.0 }` skips the check.
- **Consequence:** not a panic (these are `f64`, not `SourceSpan`'s `u32`) but a silent wrong
  result: `max_x()` returns `NaN`, and `is_empty()` returns `false` for a rectangle with negative
  area — directly contradicting the module's documented guarantee that a `Rect` "describes a real,
  finite, non-negative rectangle." Nothing in the crate catches this; it would silently corrupt any
  downstream PDF-export consumer trusting the invariant.
- **Repro:** built at `.../scratchpad/audit-links/` (`crate-copy/` verbatim copy, no Cargo.toml
  edits needed — no workspace-inherited fields in this crate's manifest).
- **Debug and release output (identical in both):**
  ```
  Rect::new(NAN origin, width=-5.0, height=-3.0) = Err(NonFiniteOrigin { x: NaN, y: 0.0 })
  bypassed = Rect { origin: Point { x: NaN, y: 0.0 }, width: -5.0, height: -3.0 }
  bypassed.max_x()   = NaN   (is_nan = true)
  bypassed.max_y()   = -3
  bypassed.is_empty() = false   <- false, even though width and height are both negative
  ```
- **Classification: CONFIRMED** (executed in both profiles).

### 4. `font-engine::OpenTypeMathFace` — missing-MATH-table bypass panics in both debug and release

- **File:** `crates/font-engine/src/adapters/math.rs:31` (struct, feature `math`, in the crate's
  default feature set), validator `OpenTypeMathFace::new` at `math.rs:41`.
- **Invariant:** `new()` returns `None` unless `face.math()` (the parsed OpenType `MATH` table) is
  `Some` — i.e., the type is only supposed to exist for fonts that actually have math metrics.
- **Bypass:** `face`, `text_size_pt`, `font_id` are all `pub`, so `OpenTypeMathFace { face:
  &face_without_math, text_size_pt: 10.0, font_id }` skips `new()` entirely.
- **Consequence:** the private helper `constants()` (called by `size_pt()`,
  `opentype_constants()`, and `glyph_at()` — itself called by `glyph()`, `delimiter_sizes()`,
  `radical_sizes()`, `accent_sizes()`) does `self.face.math().expect("checked in new")`. Since this
  is a hard `.expect()` and not a `debug_assert!`, it panics unconditionally in **both** debug and
  release — this is the one finding in the sweep where the failure mode is a guaranteed crash in
  release too, not a silently-wrong value.
- **Repro:** built at `.../scratchpad/audit-floats/` — crate copied to `crate-copy/` (only
  Cargo.toml trimmed of unrelated optional features/deps), `math-layout` copied alongside as a
  dependency, and a real font with no `MATH` table (`/System/Library/Fonts/Supplemental/Arial.ttf`,
  read-only, nothing in the target repo touched) used as the fixture.
- **Debug output:**
  ```
  OpenTypeMathFace::new(&face, ..) on a non-MATH face returns: None
  Constructed OpenTypeMathFace directly via struct literal (bypassing new()).
  Calling size_pt(SizeClass::Script), which calls the private constants() helper containing `.expect("checked in new")`...
  thread 'main' panicked at .../crate-copy/src/adapters/math.rs:51:27:
  checked in new
  ```
- **Release output:** identical panic and message.
- **Classification: CONFIRMED** (executed in both profiles).

---

## Confirmed-fixed instances of the same class (not live findings, reported per task instructions)

- **`link-annotations::SourceSpan`** — the seed example. Verified on disk: `start`/`end` are now
  private (`span.rs:36`), `SourceSpan::new` is the only constructor and rejects `end < start`,
  and a regression test pins the fix down. No bypass remains.
- **`tex-calc`'s three `debug_assert!(denominator > 0)` sites** (`Unit::to_sp`,
  `Sp::checked_mul_scalar`, `Sp::checked_div_scalar`, all in `sp.rs`) — fixed in commit `2c0a7626`
  ("tex-calc: fix zero/negative-denominator panic reachable via public Expr AST"), committed
  **today**, one commit before the audited HEAD. All three sites now return
  `Err(CalcError::DivisionByZero)` instead of asserting. Re-verified by execution: constructing
  `Expr::Dim(1, 0, Unit::Pt)`, `Expr::Dim(1, -3, Unit::Pt)`, and equivalent `Expr::Scalar` denominators
  directly (bypassing the parser, exactly the shape the original bug required) and calling
  `eval::eval` produces `Err(DivisionByZero)` identically in debug and release — no panic, no
  silent wrong value in either profile. This is the fix the task's brief was written around;
  it was live earlier today and is gone now.
- **`collaboration-core::Checkpoint`** — the crate's own code and a named regression test
  (`checkpoint_with_dangling_reference_is_rejected_not_left_to_panic_on_a_later_edit`,
  `checkpoint.rs:537`) record that this crate previously had exactly this bug shape (a decoded
  `Checkpoint` with a dangling anchor reachable through `Document::apply`'s
  `signed_index().expect(...)`). It is now fixed: `Checkpoint`'s fields are private, and
  `Checkpoint::from_bytes` runs a `validate()` step before returning. No bypass remains.

---

## Design observations (pub fields, no validating constructor — not defects)

These are informational only, per the task's instruction to list them without treating them as
bugs. Grouped by crate; each line is `Type (file) — note`.

**paragraph-layout:** `HyphenationPoint` (hyphenate.rs), `Glyph`/`Penalty`/`Kern`/`ShapedGlyph`
(items.rs), `GlyphRun`/`Glue` (items.rs, presets only), `FontId` (metrics.rs, opaque token — no
invalid state), `Ligature` (metrics.rs), `Core14Times` (core14.rs, moot — no invalid states),
`ParagraphBlock`/`PageParams` (pages.rs, unchecked presets), `PlacedLine`/`Page`/`PageOverflow`/`Pages`
(pages.rs, no pub constructor at all), `LineBreakParams` (linebreak.rs, checks live in a separate
`adapter::validate_params`, not the type), `PositionedGlyph`/`PositionedRun`/`BreakPoint`/`Line`/
`Overfull`/`Stats`/`Lines` (linebreak.rs, output-only), `LayoutError` (adapter.rs, error type). Also
worth noting (not a bypass of this exact class): `lib.rs` re-exports both the checked
`try_layout_paragraph` and the raw, unchecked `layout_paragraph` — a caller can dodge validation by
calling the other function, which is a different shape than a struct-literal bypass and wasn't
counted as a finding.

**font-engine:** `FaceMetrics` (adapters/paragraph.rs), `ExportRun` (adapters/preview.rs), `FontFile`/
`Descriptor`/`PdfFontProgram` (embed.rs — `width()`'s `binary_search` assumes sorted `cid_widths`,
unenforced by the type), `EncodingCode`/`GlyphId` (already-bounded primitives), `FontSource`/`FontId`/
`VerticalMetrics`/`Unsupported` (lib.rs), `FontDescriptor`/`LicenseMetadata`/`ManifestEntry`/`PinnedFont`
(manifest.rs, validated by external free functions, not their own constructor), `MathConstants`
(math.rs, 56 fields, no cross-field invariant), `MissingGlyph`/`Glyph`/`Cluster`/`ShapeOptions`/`Shaped`
(shape.rs — `Shaped`'s `units_per_em`-consistency check is bypassable but nothing in-crate breaks on
violation), `Subset` (subset.rs). 8 more (`AfmHeader`, `GposKerning`, `MarkAttachment`,
`GsubLigatures`, `KernTable`, `Lookup`, `ClassDef`, `Coverage`) live in private (`mod`, not `pub
mod`) modules and aren't externally reachable at all regardless of field visibility.

**math-layout:** all 28 pub types are plain data-assembly with no validating constructor at all —
`BoxKind`/`Child`/`MathBox`/`PositionedGlyph`/`PositionedRule`/`PositionedRuns` (boxes.rs),
`CmMathMetrics` (cm.rs), `FontId`/`Glyph`/`MathParams`/`Extensible`/`OpenTypeMathConstants`
(metrics.rs), `Nucleus`/`Atom`/`MathList` (mathlist.rs), `Limitation`/`Layout` (layout.rs), `Style`
(style.rs), `TfmChar`/`TfmFont` (tfm.rs), `TimesApproxMetrics` (times.rs). All index/lookup usage
goes through checked `.get()`/`.find()`, so none of these are exploitable even though fields are
public.

**title-layout:** `AbstractLayout`, `TitleBlockInput` (title.rs — invariants enforced by a
`validate()` call at every use site instead of a constructor), `DateField`, `RowKind`,
`MeasuredRow`, `TitleBlockLayout`, `HorizontalExtent`, `MeasuredTitleBlock`, `TitleLayoutError` — all
output-only or re-validated on every use, no bypass possible.

**toc-layout:** `EntryError`/`LineBoxError` (error carriers), `LaidOutEntry`/`LayoutError` (output
types), `InvalidWidth` (measure.rs, error carrier), `SourcedEntry`/`StabilizedEntry`/`StabilizedToc`/
`StabilizationError`/`ConvergenceError` (stabilize.rs/converge.rs — no public constructor). Also
noted (checked, not a finding): `EntryRecord`'s own invariant (non-empty title, `page != 0`, `level
<= MAX_LEVEL`) is bypassable, but bypassing it only changes a displayed label/indent, no
panic/overflow.

**color-expressions:** `ColorExprError` (error.rs) — enum variant fields are inherently public with
no invariant to violate; nothing downstream recomputes from them.

**image-assets:** `Dimensions`, `IncludeGraphicsSpec`, `Crop` (geometry.rs — validity checked later
via a separate `.validate(dimensions)` method, not at construction), `Length` enum (accepts any
`f64` including negative/NaN, explicitly documented as "left to the layout engine").

**link-annotations:** `SourcePos`, `Point`, `PageTarget`, `LinkAnnotation` (composed from
already-validated components, no invariant of their own), plus 8 error-carrier enums with public
variant payloads and no invariant to violate.

**math-accessibility:** `PathStep`, `UnsupportedReason`, `Unsupported`, `AccessibilityError`,
`DescribedNode`, `Description` — all in `lib.rs`, none with an invariant to enforce; `NodeId`'s
inner field is already private.

**spellcheck:** `SpellCheckerConfig`, `Misspelling`, `UserDictionaryError` — no constructor; the one
value that matters (`max_edit_distance`) is unconditionally re-clamped inside `SpellChecker::new`
regardless of how the config was built, so an out-of-range value can't reach unguarded logic.

**project-templates:** `TemplateFile` (constructor exists but validates nothing), `Template`
(validity is an opt-in `validate()` method, correctly called unconditionally at the one dangerous
use site, `instantiate()`), `ManifestError`/`InstantiateError`/`FieldError`/`PackageNameError`/
`PathError` (error carriers), `InstantiateOptions`, `CreationRecord`, `InstantiateReport`.

**editor-snippets:** `Anchor` (anchor.rs — doc comment explicitly disclaims validation and defers
all checking to `SnippetPlan::compute`, by design), `PlaceholderSpan`/`Expansion` (expand.rs — no
constructor; nothing in-crate slices `text` by an `occurrence` range so it isn't exploitable),
`DocumentId` (constructor exists but validates nothing), `SnippetError`/`Staleness`/`PlanError`
(error/output carriers).

**document-statistics:** `RevisionId` (constructor exists but validates nothing), `MathItem`,
`WordStats`, `MathStats`, `Lookup`, `ProjectTotals` — no constructors. Named as a positive
counter-example: `ScanLimit` keeps `max_bytes` private with only accessors — the correct pattern.
Also checked and excluded: `ScanTooLarge`'s doc comment claims `scanned > limit` always, but its
`pub(crate) fn new` doesn't itself enforce it and the value is only ever used in `Display`
formatting — no concrete breakage.

**project-bundle:** `BundleFile`, `Bundle`, `RootedFile`, `FilePreview`, `ImportPreview`,
`ImportOutcome`, `BundleLimits`, `BundleEntry` (constructor exists but validates nothing) — none
exploitable: `apply_import` re-validates against real on-disk state via compare-and-swap regardless
of what a caller hands it, so a hand-forged `ImportPreview` can only be refused, never silently
applied.

**collaboration-core:** `ReplicaId`, `OpId`, `Op`, `OpPayload` — explicitly documented as
intentionally-raw wire-level data (lib.rs:367-373: operations "arriving from a peer are bug- or
attacker-controlled," so validation is deliberately done at `Document::apply`'s boundary, not at
construction). `OpBuilder` is offered as a convenience path but bypassing it is anticipated and
safely handled per the crate's own tests.

**tex-calc:** `Stmt`, `Value`, `Tok`/`Spanned` (lexer.rs — doc says a `Tok::Number` denominator
should be `> 0`, but the private `Parser` is the only path to evaluation, so an externally-built bad
token stream has no reachable path in), `OverflowInfo`. Positive counter-example: `LengthTable`
keeps all fields private.

---

## Repro artifacts

All scratch reproductions live under
`/private/tmp/claude-503/-Users-dqi26/50763fe3-5136-4e4d-ab08-2a19a3be5880/scratchpad/audit-<name>/`
(one directory per crate that needed one: `audit-contents`, `audit-links`, `audit-floats`, plus
`audit-calc` for the fixed-bug re-verification). Each contains an untouched copy of the crate
(`crate-copy/`) and a small binary crate (`repro/`) that constructs the type both ways and prints
the outcome. No worktree file was read-write touched; `CARGO_TARGET_DIR` was redirected outside the
checkout for every build.
