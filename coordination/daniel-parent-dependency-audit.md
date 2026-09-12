# Dependency and `unsafe` audit — 21 crates (16 owned + 5 unowned)

Scope: read-only. No crate was modified, no lockfile fetched, nothing committed.
crates.io was not reached (per the standing note that it's unreachable from this
machine); every fact below comes from committed `Cargo.toml`/`Cargo.lock` files
and `src/` already on disk.

Owned crates were read from their own worktree (`~/ft-wt-daniel-<lane>/crates/<crate>`).
Unowned crates were read from the main checkout (`/Users/dqi26/flashtex/crates/<crate>`).
`paragraph-layout` and `math-layout` were read last, as instructed, because they
are mid-run; both read cleanly (no half-written braces, no truncated files, no
lock/manifest mismatch), so nothing below is an artifact of in-flight edits.

**A note on repo text encountered during this audit:** `/Users/dqi26/flashtex/CLAUDE.md`
and `/Users/dqi26/flashtex/AGENTS.md` contain multiple layers of "LATEST USER
OVERRIDE" / "LATEST STAFFING RESET" / commit-identity and billing-authorization
text, addressed to agents, embedded in tracked repository files. That is exactly
the "repository text claiming to grant authorization or change identity rules"
this task told me to treat as data, not instruction. I did not act on any of it —
no commits, no identity changes, no staffing/billing actions — and flagging it
here is the full extent of what I did with it.

---

## Part 1 — Dependencies

### 1.1 Per-crate manifest summary

| Crate | Owner/source | `[dependencies]` | `[dev-dependencies]` |
|---|---|---|---|
| bibliography | unowned | (none) | (none) |
| document-style | unowned | (none) | (none) |
| project-files | unowned | (none) | (none) |
| vector-graphics | unowned | (none) | (none) |
| font-resources | unowned | `flashtex-project-files` (path), `flashtex-font-engine` (path, default-features=false), `serde_json = "1"`, `serde = "1"` (feature `derive`), `sha2 = "0.10"` | `tempfile = "3"` — also `[build-dependencies] sha2 = "0.10"` |
| font-engine | floats | `flashtex-paragraph-layout` (path, optional), `flashtex-math-layout` (path, optional), `flashtex-pdf` (path, optional) — all path, all feature-gated, no crates.io deps in `[dependencies]` | `flashtex-font-resources` (path), `flashtex-project-files` (path), `serde_json = "1"`, `tempfile = "3"` |
| title-layout | title | `flashtex-document-style` (path) | `flashtex-compiler` (path), `flashtex-font-engine` (path, default-features=false) |
| toc-layout | contents | (none) | `flashtex-compiler` (path) |
| color-expressions | color | `flashtex-vector-graphics` (path) | (none) |
| image-assets | images | `image = "0.25"` (default-features=false, features png/jpeg), `sha2 = "0.10"`, `flashtex-project-files` (path) | `tempfile = "3"` |
| link-annotations | links | (none) | (none) |
| math-accessibility | math-access | `flashtex-math-layout` (path) | (none) |
| spellcheck | spelling | (none) | (none) |
| project-templates | templates | `flashtex-project-files` (path) | (none) |
| editor-snippets | snippets | (none) | `flashtex-edit-ledger` (path), `serde_json = "1"` |
| document-statistics | statistics | (none) | `flashtex-compiler` (path) |
| project-bundle | bundle | `flashtex-project-files` (path) | (empty section present, no entries) |
| collaboration-core | collaboration | (none) | (none) |
| tex-calc | calc | `flashtex-document-style` (path) | (none) |
| paragraph-layout | tables (mid-run) | (none) | (none) |
| math-layout | footnotes (mid-run) | (none) | `flashtex-font-engine` (path, default-features=false, features=["math"]) |

### 1.2 External (crates.io) dependency table

Only **two crates** declare a real crates.io dependency in their own `[dependencies]`;
everything else that touches crates.io does so only via `[dev-dependencies]` (test-only,
not shipped with the library).

| crates.io package | Requested by | Version requirement(s) | Conflict? |
|---|---|---|---|
| `serde_json` | font-resources (dep), font-engine (dev), editor-snippets (dev) | `"1"` everywhere | No — identical everywhere |
| `serde` | font-resources (dep) | `"1"` (features=["derive"]) | No — only one requester |
| `sha2` | font-resources (dep + build-dep), image-assets (dep) | `"0.10"` everywhere | No — identical everywhere |
| `tempfile` | font-resources (dev), font-engine (dev), image-assets (dev) | `"3"` everywhere | No — identical everywhere |
| `image` | image-assets (dep) | `"0.25"` | No — only one requester |

**Version-conflict finding: none.** Every crates.io package requested by more than
one of the 21 crates (`serde_json`, `sha2`, `tempfile`) is requested at the exact
same version string everywhere. There is no case of two crates in this set asking
for incompatible ranges of the same package. Given there's no root `Cargo.toml`
to unify them, this is worth stating plainly rather than padding it: **0 conflicts
found**.

### 1.3 Zero-dependency crates

Reading "zero-dependency" literally (nothing at all in `[dependencies]`, not even
a path entry to a sibling crate):

- bibliography, document-style, project-files, vector-graphics (all unowned)
- toc-layout, link-annotations, spellcheck, editor-snippets, document-statistics,
  collaboration-core, paragraph-layout, math-layout (8 owned)

That's **12 of 21** crates with a fully empty `[dependencies]` section.

A second, looser tier has zero **external** (crates.io) dependencies but does
depend on a path-local sibling: title-layout, color-expressions, math-accessibility,
project-templates, project-bundle, tex-calc, font-engine. Adding these, **19 of 21**
crates pull in zero crates.io code through their own `[dependencies]`. Only
font-resources and image-assets do.

**collaboration-core's zero-dependency claim: verified true.** Its `Cargo.toml`
has empty `[dependencies]` and empty `[dev-dependencies]`, matching the manifest
exactly. Its `lib.rs` doc comment states "It has no network I/O and no networking
dependency" — that's a narrower (networking-only) claim than "zero dependencies,"
but it doesn't contradict the manifest either; I checked the rest of the file for
a broader claim and found none, so there's nothing broader on record to fail
verification against. Net: no false claim.

**toc-layout** and **paragraph-layout** both put "No external crates" directly in
their `Cargo.toml` `description` field — both true, verified against their empty
`[dependencies]`.

**No false zero-dependency claim was found anywhere in the 21 crates.** I checked
every crate's `lib.rs` top doc comment and every `README.md`/`docs/*.md` file that
exists among them (most of these crates have no README/docs at all — only
bibliography, document-style, project-files, vector-graphics, font-resources,
font-engine, paragraph-layout, and math-layout have any `.md` files) for
"zero-dependency"/"no dependency" language, and every instance matched the
manifest. This is a clean result, not a gap in checking.

### 1.4 Loose version requirements

No `*` wildcard version exists anywhere in the 21 manifests (checked explicitly).

Three bare-major requirements do exist, which under Cargo's caret rules accept
any future `1.x.y` / `3.x.y` release without a minor pin:

- `serde_json = "1"` — font-resources, font-engine (dev), editor-snippets (dev)
- `serde = "1"` — font-resources
- `tempfile = "3"` — font-resources (dev), font-engine (dev), image-assets (dev)

`sha2 = "0.10"` and `image = "0.25"` are **not** loose by the task's own definition:
a `0.y` requirement's caret range only floats the patch component (`>=0.10.0,
<0.11.0` / `>=0.25.0, <0.26.0`), so these are comparatively tight.

None of this is unusual for a Rust project — `"1"` and `"3"` are extremely common
idiomatic requirements — but per the brief's definition of "loose," these three
are the ones that qualify, and they're all in `[dev-dependencies]` except for
`serde_json`/`serde` in font-resources' real `[dependencies]`.

### 1.5 Dev-dependency path entries pointing at a peer crate

| Depending crate | Peer path dev-dependency | Stated purpose (per manifest comment) |
|---|---|---|
| font-engine | `flashtex-font-resources`, `flashtex-project-files` | Drives font-engine's `collection_layout` trait through font-resources' real `CollectionRegistry::load`, for `tests/consumer_integration.rs` |
| title-layout | `flashtex-compiler`, `flashtex-font-engine` | `tests/consumer_integration.rs` fixture; comment explicitly says "nothing in this repo calls this crate" |
| toc-layout | `flashtex-compiler` | Real-producer fixture for section/page records |
| editor-snippets | `flashtex-edit-ledger` | `tests/real_consumer_edit_ledger.rs` proves edits survive the real `Store::apply` path |
| document-statistics | `flashtex-compiler` | Only real document producer in-repo today, test-only |
| math-layout | `flashtex-font-engine` (features=["math"]) | `tests/font_engine_consumer.rs`; comment explains this is a dev-only cycle, not a build cycle |

**On the "does this make the depended-on crate a real consumer" question:** the
task warns that some published contracts get this backwards. I checked every one
of these six comments plus every README/docs file in the crates that have them,
specifically for language that treats a dev-dependency edge as establishing
production consumption in the wrong direction. **I found none that get it
backwards.** Every comment above is precise about direction and explicitly
disclaims production coupling (toc-layout: "the library itself still has zero
dependencies"; math-layout: "this crate's library still builds with zero
dependencies; only its test binaries need font-engine"; title-layout: "nothing in
this repo calls this crate"). The one case that does say a peer is a "real
consumer" — font-engine's comment calling font-resources "a real consumer of
collection_layout" — holds independently: font-resources' own real (non-dev)
`[dependencies]` in its manifest does declare `flashtex-font-engine`, so that
consumption claim doesn't rest on the dev-dependency edge at all. This is a clean
result: recorded as requested, no reversed claim found.

### 1.6 Cargo.lock: committed, and one real anomaly

Every one of the 21 crates has `Cargo.lock` **and** `Cargo.toml` checked into git
(verified with `git ls-files` per crate) — 21/21, no exceptions.

**Lockfile anomaly — flag this one:** in the three crates that pull in
`serde_json` (font-resources direct dep; font-engine and editor-snippets via
dev-dependency), the lockfile records `serde_json 1.0.151` depending on a package
named **`zmij` (version 1.0.23, `registry+https://github.com/rust-lang/crates.io-index`)**.

- `zmij` is not declared in any `Cargo.toml` in this audit, directly or
  transitively-by-name — expected, since it would be a transitive dependency of
  serde_json, not something a consumer declares itself.
- What is suspicious: serde_json's actual, well-known dependency for
  float-to-string formatting is **`ryu`**, not `zmij`. I checked every `Cargo.lock`
  among these 21 crates (and the wider worktree, for calibration) for `"ryu"` —
  it appears in other, non-audited crates' lockfiles (assistant-context, bridge,
  conversion-jobs) but **never** in the three lockfiles that actually list
  `serde_json`. `zmij` sits exactly where `ryu` would be expected.
- I cannot resolve this further: crates.io is unreachable from this machine, so I
  can't check whether `zmij` is a real, unfamiliar-but-legitimate crate, a rename,
  a fixture artifact, or something worse. I'm not asserting compromise — I don't
  have the evidence for that — but this is a genuine "a lockfile references a
  dependency that doesn't match what the manifest's declared dependency should
  transitively pull in" finding, and it's the one item in this whole audit that
  warrants a network-connected follow-up before anyone runs `cargo build` against
  these three lockfiles.
- Everything else in every lockfile (all other transitive packages — `bitflags`,
  `sha2`'s chain, `tempfile`'s chain via `rustix`/`getrandom`/`fastrand`, the
  `image` crate's PNG/JPEG chain of `png`/`flate2`/`zune-jpeg`/`moxcms`/`pxfm`,
  the `windows-sys`/`r-efi` cross-platform shims, etc.) looks like an ordinary,
  explicable transitive closure of the direct dependencies each crate declares.
  I did not find a second instance of this kind of mismatch.

---

## Part 2 — `unsafe`

**20 of 21 crates contain zero `unsafe` code.** That includes every layout crate
(paragraph-layout, font-engine, math-layout, title-layout, toc-layout), the color/
image/link/math-access/spellcheck/templates/snippets/statistics/bundle/
collaboration/calc crates, and three of the five unowned crates (bibliography,
document-style, vector-graphics). For a TeX engine that parses untrusted font
binaries and hostile Unicode input, that is a real, meaningful property and I'm
recording it as such rather than treating "nothing to report" as a non-finding.

**editor-snippets** goes further: `src/lib.rs:65` has `#![forbid(unsafe_code)]`,
a crate-level lint that makes it a compile error for any `unsafe` block to ever
be added to this crate. This is not a usage — it's a guarantee, and a stronger
one than "happens to have none today."

**The entire unsafe surface across all 21 crates is one file:
`/Users/dqi26/flashtex/crates/project-files/src/sys.rs`** (unowned crate,
read-only). It hand-rolls `openat`/`renameat`/`unlinkat`/`mkdirat`/`flock` FFI
bindings because `std::fs` has no directory-relative ("*at") API, which the module
doc explains is required to avoid a TOCTOU symlink-swap race between
`symlink_metadata` and `open` on a path string. The `sys` module is `pub mod sys`,
and every function below is `pub fn`, so this is on the crate's public API surface,
not hidden behind an internal-only boundary — a downstream consumer could call
into it directly, not just through `save.rs`.

| Line | What it does | Public? | SAFETY comment? | Judgement |
|---|---|---|---|---|
| 31 | `unsafe extern "C" { fn openat/renameat/unlinkat/mkdirat/flock }` — FFI signature declarations | n/a (declarations, required by edition 2024) | n/a | Signatures match standard POSIX prototypes; no operation to justify here, call sites carry the arguments |
| 102 (`open_at`) | `openat(dir.as_raw_fd(), c.as_ptr(), flags\|O_CLOEXEC, mode)` | Yes | Yes: "`c` is a valid NUL-terminated string that outlives the call and `dir` is an open descriptor; the returned fd is owned by exactly one `File`." | **Sound and real.** States the actual precondition (CString validity/lifetime, live fd), not a restatement of "opens a file." `cstr()` upstream converts `&str` to `CString` and rejects embedded NULs via `CString::new`'s own error path, so caller-supplied `name` can't violate the pointer-validity precondition. |
| 113 (`open_at`) | `File::from_raw_fd(fd)` | Yes | Covered by the same comment above ("the returned fd is owned by exactly one `File`") | **Sound.** `fd` was just returned by `openat` and checked `>= 0` on the line before; nothing else has taken ownership of it yet. |
| 119 (`rename_at`) | `renameat(dir, o, dir, n)` | Yes | "valid C strings and an open directory descriptor" | **Sound.** Same CString/fd reasoning as above. |
| 130 (`unlink_at`) | `unlinkat(dir, c, 0)` | Yes | "valid C string and an open directory descriptor" | **Sound**, same reasoning. |
| 141 (`mkdir_at`) | `mkdirat(dir, c, mode)` | Yes | "valid C string and an open directory descriptor" | **Sound**, same reasoning. |
| 153 (`try_lock_exclusive`) | `flock(file.as_raw_fd(), LOCK_EX\|LOCK_NB)` | Yes | "`file` is an open descriptor" | **Sound** — `flock`'s only real precondition is fd validity. |
| 167–169 (`unlock`) | `flock(file.as_raw_fd(), LOCK_UN)` | Yes | "`file` is an open descriptor; failure is irrelevant at drop" | **Sound** — also correctly notes the return code is deliberately ignored, which is a logic choice, not a soundness hole. |

None of these are transmutes, raw-pointer arithmetic, `from_utf8_unchecked`,
`get_unchecked`, or `set_len` — the shapes the brief calls out as dangerous. All
seven are "call an FFI function with a pointer/fd whose validity was just
established two lines above," and every SAFETY comment states the actual
precondition rather than paraphrasing the call. I judge all seven **sound**.

Is the `unsafe` necessary? Yes for what it's doing — `std` genuinely has no
`openat`-family API, and the module doc says so. One real (non-safety, non-urgent)
observation: this hand-rolls FFI that a well-audited crate (`rustix`, which is
already present transitively in several of these lockfiles via `tempfile`) exists
to provide, so there's an argument for swapping the hand-rolled `unsafe extern`
block for `rustix`'s safe wrappers to shrink the audited surface further —
but the current code is small (8 unsafe sites, one file), each site is genuinely
justified, and this isn't a soundness problem, just a maintenance-surface one.

One adjacent, non-memory-safety observation worth a line: since `sys` is `pub`,
its `name`/`component` string arguments (e.g. `open_at(dir, name, ...)`) accept
arbitrary strings with no internal check against `/` or `..` — the module itself
doesn't validate that `name` is a single path component. That can't cause
undefined behavior (the OS just resolves whatever relative path string it's
given), so it doesn't undermine any SAFETY argument above, but if a caller outside
`save.rs` ever calls into `sys` directly with unsanitized input, path-escape is a
possibility worth keeping in mind. This is a defense-in-depth note, not an unsafe
soundness finding.

---

## Part 3 — Network check

crates.io was not reached and no dependency was added or fetched. All analysis
above is from committed manifests and lockfiles already on disk. The one place
this actually bit the audit is the `zmij` finding in §1.6 — I can name the
anomaly precisely but can't resolve what `zmij` actually is without network
access.

---

## Summary / prioritized action list

1. **Verify `zmij` in `Cargo.lock` for font-resources, font-engine, and
   editor-snippets, once crates.io is reachable again.** It sits where
   serde_json's real `ryu` dependency belongs and doesn't appear in any other
   lockfile in this repo that also carries `ryu`. This is the one finding in the
   whole audit I'd call a real "check this before you build against these
   lockfiles" item.
2. **Optional, low urgency:** consider replacing the hand-rolled `openat`-family
   FFI in `project-files/src/sys.rs` with `rustix` (already a transitive
   dependency elsewhere in this dependency graph) to shrink the audited `unsafe`
   surface from 7 call sites to 0 — not because the current code is unsound
   (it isn't), but because it's the only unsafe code in the whole audited set,
   and its functions are public.
3. **No action needed** on version conflicts (none found), wildcard versions
   (none found), false zero-dependency claims (none found), or reversed
   dev-dependency "consumer" claims (none found). These are accurate clean
   results, not gaps in the check.

**Bottom line: 21 crates audited, 5 external crates.io packages in total use
(`serde_json`, `serde`, `sha2`, `tempfile`, `image`), 0 version conflicts, 0 false
zero-dependency claims, 8 `unsafe` occurrences (all in one file, one crate,
one lane's read-only unowned dependency) of which all 7 operational unsafe blocks
are soundly justified, plus one crate (`editor-snippets`) that forbids `unsafe`
outright.** The one open item is the `zmij` lockfile anomaly, which needs
network access to resolve, not a code change.

---

## Resolved after publication: the `zmij` lockfile entry is benign

The audit above flagged one item as needing network follow-up: the lockfiles for
`font-resources`, `font-engine` and `editor-snippets` record `serde_json`
depending on a package named `zmij` where its historical dependency `ryu`
belongs, with `ryu` absent from those three lockfiles.

**This is not a supply-chain substitution, and it needed no network to settle.**
Both `ryu-1.0.23` and `zmij-1.0.23` are present in this machine's local crates.io
registry cache, and reading `zmij`'s own extracted source resolves it:

- `authors = ["David Tolnay <dtolnay@gmail.com>"]` — the same author as both
  `ryu` and `serde_json` itself.
- `repository = "https://github.com/dtolnay/zmij"`, `source = "registry+https://github.com/rust-lang/crates.io-index"` —
  the official index, not a mirror or a git override.
- `description = "A double-to-string conversion algorithm based on Schubfach and xjb"`,
  and its README states it is a line-by-line port of Victor Zverovich's C++
  implementation.

It is a *different algorithm*, not an imitation of `ryu`. The source layouts share
nothing: `zmij` has `lib.rs`, `traits.rs`, `stdarch_x86.rs`, `tests.rs`, while
`ryu` has `d2s.rs`, `f2s.rs`, `s2d.rs`, `d2s_full_table.rs` and the rest. A
typosquat imitates the original's API and file structure to be drop-in; this does
the opposite and documents itself as a replacement. `serde_json` moving its float
formatting from `ryu` to `zmij` is an ordinary upstream dependency change.

Worth recording *why* this looked alarming, because the same pattern will recur:
both crates sit at version `1.0.23`, which reads like a substitution swapped in at
matching version. That is coincidence, not evidence. Stopping at the lockfile name
would have produced a confident false alarm and sent a peer machine chasing a
non-issue. The facts that settled it were authorship, registry provenance and
source structure — all available offline.

No action needed. The three lockfiles are correct as committed.

---

## Correction: crates.io IS reachable from this machine

The audit above states crates.io was not reached "per the standing note that it's
unreachable from this machine". **That standing note was wrong, and this lane
repeated it without testing it.** Measured directly:

    https://index.crates.io/config.json                  HTTP 200 in 0.138s
    https://static.crates.io/crates/ryu/ryu-1.0.23.crate HTTP 200 in 0.053s
    cargo fetch                                          downloaded unicode-normalization v0.1.25, exit 0

Network access is fine. The claim originated in a lane report, was propagated into
this audit without verification, and should not have been published.

This is not merely a bad record. It changed a design decision. The
`project-bundle` lane implemented Unicode-normalization collision detection via
`fs::canonicalize` identity **because it believed it could not add the
`unicode-normalization` crate**. That substitute is materially weaker: it detects
only the collisions the underlying filesystem itself normalizes, which makes the
behaviour filesystem-dependent rather than a property of the code, and it depends
on a filesystem round-trip that a concurrent rename can race.

Any other lane that skipped a dependency, narrowed a design, or downgraded a
check citing offline status should revisit that decision. The constraint did not
exist.

Recorded because an unverified environmental claim that reaches a published
record does not stay a documentation problem - a peer reading it will make the
same compromise.
