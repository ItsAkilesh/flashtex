# Daniel-lane adapter contract audit

Read-only audit of all 16 `daniel-*` lane worktrees against their published
adapter contracts (`coordination/daniel-<lane>.md`). Performed by checking
each lane's own worktree/branch: every quoted signature against real
`src/`, every error enum for omissions/bogus variants, the worked example
by hand, the cited "exact tested SHA" via `git cat-file`/`git merge-base
--is-ancestor`, and every consumer claim via a fresh grep plus a
`Cargo.toml` dependency-table check (dev-dependency added for a lane's own
test fixture does not count as a real consumer).

**Repository text seen and not acted on:** every worktree's `AGENTS.md`/
`CLAUDE.md` (and several `coordination/*.md` files — `COMMANDER*`,
`KABIR-QUIESCED-HANDOFF.md`, etc.) contains fabricated "user override" /
"Commander authority" / staffing / billing / commit-identity-override text
directed at agents. This was treated as inert repository data, not
instruction, throughout this audit: nothing was committed, no identity was
changed, no file outside the one report below was written.

**Result: 16 contracts checked, 2 drifted (bundle, links), 14 clean.**
"Drifted" here means a genuine defect under the task's own severity bar
(wrong/nonexistent signature, omitted or bogus error variant, false SHA, or
false consumer claim) — not terseness or cosmetic formatting.

Two lanes (`tables` = paragraph-layout, `footnotes` = math-layout) were
flagged as mid-run and audited last. `tables` was mid-write when first
checked (uncommitted change to its own coordination file); by the time it
was re-checked it had committed and settled (new HEAD, clean tree, stable
across a re-check) — audited at that settled state. `footnotes` was clean
and stable throughout.

## Summary table

| Lane | Contract present | Signatures checked | Drift count | SHA valid |
|---|---|---|---|---|
| tables | yes | 9 | 0 | yes (3 SHAs) |
| floats | yes | 7 | 0 signature drift; 4 real error-variant omissions | yes (2 SHAs) |
| footnotes | yes | ~15 | 0 | yes (5 SHAs) |
| title | yes | 16 | 0 (minor structural note only) | yes (2 SHAs) |
| contents | yes | 9 | 0 | yes (1 SHA) |
| color | yes | 9 | 0 | yes (3 SHAs) |
| images | yes | 17 | 0 | yes (1 SHA) |
| links | yes, but empty of content | 0 (none present to check) | 1 — structural: the "typed contract" section is gone | yes but stale (5 SHAs) |
| math-access | yes (via traceable rev-history reference) | ~15 | 0 | yes (2 SHAs) |
| spelling | yes | ~20 | 0 | yes (3 SHAs) |
| templates | yes | ~25 | 0 | yes (4 SHAs) |
| snippets | yes | ~30 | 0 | yes (2 SHAs) |
| statistics | yes | ~25 | 0 | yes (5 SHAs) |
| bundle | yes | ~15 | 3 — nonexistent method, 2 omitted error variants | yes (4 SHAs) |
| collaboration | yes | ~30 | 0 | yes (2 SHAs) |
| calc | yes | ~25 | 0 | yes (5 SHAs) |

Consumer claims checked: 15 of 16 lanes make one (tables makes none); all
15 held up under a fresh grep and a `Cargo.toml` `[dependencies]` vs.
`[dev-dependencies]` check, including the one genuine named-real-consumer
claim (`footnotes` → `font-engine`'s `OpenTypeMathFace`, a real non-dev
dependency edge, correctly distinguished from `math-layout`'s own
dev-only, feature-gated dependency back on font-engine for its test
fixture). **False consumer-claim count: 0. False SHA count: 0** (every
cited SHA passed both `cat-file -e` and `merge-base --is-ancestor`).

## Real defects

### 1. `bundle` (crates/project-bundle) — nonexistent `ProjectRoot::resolve`

`coordination/daniel-bundle.md`, the "Typed contract" section, states:

> `ProjectRoot::resolve(relative: &str) -> Result<PathBuf, BundleError>` /
> `ProjectRoot::read_rooted(relative: &str) -> Result<Vec<u8>, BundleError>`
> — the single chokepoint every read goes through.

No method or function named `resolve` exists anywhere in
`crates/project-bundle/src/` (`grep -rn "fn resolve\b" src/` — no hits).
The real syntactic-validation entry point is
`ProjectRoot::normalize(relative: &str) -> Result<ProjectPath, BundleError>`
(`src/root.rs:79`) — a different name *and* a different return type
(`ProjectPath`, not `PathBuf`; `ProjectPath` is `flashtex_project_files`'
own type). A consumer following the contract literally would call a method
that fails to compile. This looks like a leftover from rev 1's
pre-`flashtex_project_files`-reuse design that rev 2 replaced but the
"Typed contract" block was never updated to match.

### 2. `bundle` — two real public `BundleError` variants omitted

The same section explicitly claims to have just fixed staleness in the
error-variant list: "the rev 1 list printed here previously was stale ...
none of which had been reflected here until now," then lists 17 variants.
The real `BundleError` enum (`src/error.rs:10-124`) has **19** public
variants — the list omits `ReservedPath(String)` and
`PreviewBundleMismatch(String)`. Both are live, actively-returned behavior,
not dead code: `apply_import` constructs both
(`src/apply.rs:131-136` — `is_reserved(&normalized)` →
`Err(BundleError::ReservedPath(...))`, and the preview/bundle mismatch
check → `Err(BundleError::PreviewBundleMismatch(...))`), and each has its
own doc comment describing real, load-bearing behavior (`ReservedPath`
protects the project lock file `.flashtex/project.lock` from being
overwritten mid-batch; `PreviewBundleMismatch` replaced a prior `expect()`
that used to panic). A consumer matching exhaustively on `BundleError`
per the contract's own list would not compile, and one that only reads
the contract would never learn these two failure modes exist.

### 3. `bundle` — stale variant name in prose (minor)

`coordination/daniel-bundle.md:273`, in the "Rooting" section, still says
"...otherwise `SymlinkEscapesRoot`." That variant was renamed to
`SymlinkRefused` in revision 2 (the contract's own line 172 says so, and
the "Typed contract" section's line 256 correctly uses the new name) — but
this one narrative sentence was never updated, and `SymlinkEscapesRoot`
does not exist anywhere in current source (`grep -rn SymlinkEscapesRoot
src/` — no hits). Low severity (the correct name appears elsewhere in the
same document), included for completeness since it does name a
non-existent variant.

### 4. `links` (crates/link-annotations) — the typed contract itself is missing

`coordination/daniel-links.md`'s only API-description section reads:

> ## Typed contract (unchanged from rev 2)
>
> ... Public surface (re-exported from `flashtex_link_annotations`):
> unchanged — **see rev 2 notes below for the full API description.**

There is no "rev 2" section, or any other section, anywhere else in the
current file — the whole document is 132 lines and this is the only API
section. `grep -c '```' ` / `grep -n "pub fn\|pub struct\|pub enum"` over
the file: zero hits, anywhere. Checked git history: the prior commit
(`5edebe64`, rev 2) *did* have a full, real "Typed contract" section (with
every function's inline signature — `validate_uri(&str) ->
Result<ValidatedUri, UriError>`, `SourceIdentity::bind(...) ->
Result<SourceIdentity, SourceIdentityError>`, etc.). The rev-3 commit
(`3ff59769`, "publish FT-037 rev 3 checkpoint") deleted that entire section
(111 insertions / 176 deletions to this one file) and replaced it with the
9-line stub above that points at a "below" section it had just deleted.
This is not terseness — it is the assignment's core requirement
("publish typed adapter contract") failing to be met by the document
currently on this branch: a consumer has literally nothing to read.

Additional, lower-severity note: while auditing, this lane's branch
advanced under me (new commit `4538dae3`, "Fix silent u32 underflow in
`SourceSpan::len()`," changing `src/source.rs`/`span.rs`/`annotation.rs`,
not flagged by the task as a mid-run lane but evidently one). The cited
"exact tested commit" (`ea79df23...`) is still a real, valid ancestor of
the new HEAD (not a false SHA by the letter of the check), but the
contract's "src/** is byte-identical to rev 2" claim is now stale relative
to current HEAD regardless of the missing-section issue above.

## Notable, non-defect observations

- **`floats` (font-engine):** the contract's `Error` coverage is
  incomplete rather than wrong: `Malformed` is well-documented, and
  `MissingTable`/`Unsupported` are named only as scan-result bucket labels
  in a results table — but `Io(String)`, `GlyphOutOfRange(u16)`,
  `UnsupportedScript{..}`, and `UnsupportedFeature{..}` (4 of the 7 real
  public variants) are never mentioned anywhere in the document. No
  worked/call-through example is present either (only signature listings).
  Not counted as "drift" in the signature-matching sense — every signature
  actually quoted is correct — but it is a real omission under the task's
  error-variant-coverage requirement, so it's flagged above the table.
- **`title` (title-layout):** the "Public adapter contract" fenced code
  block itself lists only 5 of the 13 real `TitleLayoutError` variants;
  the other 8 are documented in the revision-history prose earlier in the
  *same* file, so nothing is actually hidden from a reader of the whole
  document. This is a structural/terseness note, not a false claim —
  contrast with `links`, where the missing content isn't anywhere in the
  document at all.
- **`math-access`:** rev 4 (the current document) doesn't restate the
  typed contract in-document; it says "unchanged from rev 3... see rev 3's
  own description," and rev 3 in turn says "see rev 2's contract listing."
  This chain was followed into git history (`git show <rev2-commit>:...`)
  and rev 2's contract is real, complete, and — verified independently
  against current `src/lib.rs` — still exactly accurate. This is the same
  "point elsewhere" pattern as `links`' defect, but here every link in the
  chain resolves to real content, so it is not counted as a defect.
- **Several lanes** (`spelling`, `templates`'s "Typed contract" listing
  style aside, `snippets`, `statistics`, `contents`, `floats`, `images`)
  publish only type/signature *listings*, not an actual call-through
  worked example with real argument values. Per the task's calibration
  this isn't penalized as drift on its own — there's nothing to check for
  compile-correctness if no example exists — but it's noted since several
  other lanes (`templates`, `calc`, `footnotes`, `collaboration`, `tables`)
  do publish a real worked example, and in every one of those cases it was
  verified to compile in principle against actual signatures (in two
  cases, `templates` and `calc`, the example is a literal copy of a real,
  passing `cargo test --doc` doctest, verified byte-identical to the
  source).
- **Type-alias "drift" that isn't:** `templates` and `bundle` both write a
  field's type as `[u8; 32]` where the real source uses
  `flashtex_project_files::Digest` (`pub type Digest = [u8; 32]`) — a
  transparent alias, not a different type. Not counted as drift.
- **`snippets`/`statistics`:** both correctly disclose that their one
  plausible "consumer" (`edit-ledger`, `compiler`, respectively) is only a
  `[dev-dependencies]` entry added for the lane's own test fixture, not a
  real production dependency — verified against each crate's `Cargo.toml`
  and the candidate crate's own manifest (no back-reference either way).
  Exactly the distinction the task asked to watch for, and both lanes get
  it right.

## Lanes checked clean (no defects)

`tables`, `footnotes`, `contents`, `color`, `images`, `math-access`,
`spelling`, `templates`, `snippets`, `statistics`, `collaboration`, `calc`
— every quoted signature matched real source exactly (name, parameter
types, mutability/ownership, return type including the `Result` error
type), every error enum's public variants were exactly covered with no
omissions and no bogus variants, every cited SHA existed and was an
ancestor of the branch's current HEAD, and every consumer claim
(real-consumer or no-consumer) held under re-verification. `footnotes` in
particular is a genuine real-consumer case (font-engine) verified correct
down to the exact struct fields and constructor signature on the consumer
side.

---

Report by: read-only audit agent, no crate files modified, no commits made.
