# flashtex-project-files

Original Rust project file layer for FlashTeX. Zero external crates, edition
2024. Owner: `mac-project-files` (Claude Code subagent, parent `mac-claude-a`).

It answers four questions the single-file Mac shell cannot today: *which files
make up this project*, *what exactly is in them* (content identity), *how do I
save without losing someone else's write*, and *what changed on disk while the
editor had a buffer open* — plus a crash-recovery journal for unsaved buffers.

```sh
cargo test --manifest-path crates/project-files/Cargo.toml
cargo clippy --manifest-path crates/project-files/Cargo.toml --all-targets -- -D warnings
```

Tests use only temporary directories under the system temp dir; they never
touch the repository.

## API

### `ProjectPath` (`path.rs`)

`ProjectPath::normalize("./ch/../a.tex")` → `a.tex`. Always relative, forward
slashes, no `.`/empty segments. Rejected with `PathError`: absolute (`/`, `~`),
drive/colon, backslash, NUL/control characters, empty, and `..` that would
leave the root. `resolve_in(base_dir, raw)` resolves relative to a directory
inside the project. This is at least as strict as runtime-v1 ("project-relative,
no parent traversal") and transfer-v1 (no backslash/colon/NUL).

### Scanner (`scan.rs`)

`scan_references(text) -> Vec<Reference>` finds `\input`, `\include`,
`\bibliography` (one reference per comma item), `\addbibresource`,
`\includegraphics[..]{..}` and bare `\input name`. Each reference carries
`span` (whole command) and `argument_span` as zero-based, end-exclusive UTF-8
byte offsets, matching runtime-v1 `source`. `%` comments, `\verb`, and
`verbatim`/`comment`/`lstlisting`/`minted`/`Verbatim` bodies are skipped;
`\inputfoo` never matches `\input`. Arguments containing `\` or `#` are
returned with `literal == false` (they need macro expansion, which this crate
does not do).

### Graph (`graph.rs`)

```rust
let graph = ProjectGraph::discover(root, &ProjectPath::normalize("main.tex")?)?;
let graph = ProjectGraph::discover_with(root, &entry, &overlay)?; // unsaved buffers win
graph.files()         // ProjectFile { path, kind, source, text, sha256, bytes, references }
graph.edges()         // Edge { from, to, reference }
graph.diagnostics()   // Diagnostic { severity, message, path, span, argument_span, kind }
graph.documents()     // runtime-v1 [{path, text}] — Tex files, entry first, DFS order
graph.documents_including_bibliography()
graph.compile_payload(project_id, revision)   // Json object
graph.compile_envelope(id, project_id, revision) // one JSON Lines `compile` request
```

Resolution rules (TeX working-directory semantics: every name is resolved
against the project root, not the including file's directory):

| Reference | Candidates tried in order | Kind |
|---|---|---|
| `\input{x}`, `\include{x}` | `x.tex`, then `x` (just `x` if it already ends in `.tex`) | Tex, scanned recursively |
| `\bibliography{x}` | `x.bib` (or `x` if it ends in `.bib`) | Bibliography, loaded, not scanned |
| `\addbibresource{x}` | `x` if it has an extension, else `x.bib` | Bibliography |
| `\includegraphics{x}` | `x` if it has a known extension, else `x.pdf,.png,.jpg,.jpeg,.eps,.svg` | Graphic, hashed, not loaded |

Diagnostics (`DiagnosticKind`): `MissingFile{target, tried}` (error; warning
for graphics), `InvalidPath{target, error}` (escaping `..`, absolute, bad
characters), `EscapesRootViaSymlink`, `Cycle{chain}`, `UnresolvableReference`
(macro in argument, warning), `InvalidUtf8`, `ReadError`, `DepthExceeded`
(64 levels). Every reference diagnostic carries the referencing file plus
`span`/`argument_span`.

Ordering is deterministic: depth-first from the entry in source reference
order; a file reached twice (diamond) appears once, at its first visit; the
cycle-closing reference is recorded as an edge and a diagnostic but not
revisited. `DiscoverError` covers only the cases where nothing useful can be
built: root not a directory, entry missing/unreadable/not UTF-8.

### Content identity (`sha256.rs`, `revision.rs`)

`sha256(bytes)`, `sha256_hex(bytes)`, streaming `Sha256::update/finalize`,
`sha256_to_hex`, `sha256_from_hex`. Hand-written FIPS 180-4; tested against
"abc", "", the 56-byte two-block vector, one million `a`, and every chunking of
a 300-byte input.

`RevisionTracker`: `observe(path, text)` returns `(FileRevision{revision,
sha256, bytes}, changed)`; a file's revision starts at 1 and increments only
when its hash changes. `project_revision()` is 0 until something is observed
and then +1 per content change or `remove`; the same sequence of observations
always yields the same numbers. `observe_graph(&graph)` records every text
file of a discovery. Use `project_revision()` as the runtime-v1 `revision`.

### Atomic save (`save.rs`)

```rust
let receipt = save_atomic(root, &path, text, Expected::Hash(last_known), force)?;
// SaveReceipt { path, bytes, sha256, mtime }
```

Write to `.<name>.flashtex-tmp-<pid>-<n>` in the same directory, `fsync`,
copy existing permissions, `rename` over the target, then `fsync` the
directory (best effort). Parent directories are created. `Expected::NewFile`
means "nothing should be there"; `Expected::Any` skips the check.

Without `force`, the on-disk state must match `expected` or the save is
refused with `SaveError::Conflict(Box<SaveConflict>)`: `kind`
(`ModifiedExternally`, `DeletedExternally`, `AlreadyExists`), `ours`
(expected hash), `theirs` (hash on disk, `None` if deleted), `mtime`, `size`.
Nothing is written on refusal. With `force` the check is skipped.

### External-change detection (`watch.rs`)

```rust
let snap = Snapshot::take(root, graph.text_paths())?;   // hashes every path
let diff = snap.diff()?;                                  // rehash only if mtime/size moved
diff.changes  // ExternalChange { path, kind: Created|Modified|Deleted, before, after }
diff.conflicts(&dirty_paths)  // Conflict { path, kind, local_dirty, before, after }
snap.record_own_write(&receipt.path, receipt.bytes, receipt.mtime, receipt.sha256);
```

`ConflictKind`: `ModifiedExternally` (disk changed, buffer clean — safe to
reload), `DeletedExternally` (with `local_dirty` telling you whether unsaved
edits would be lost), `Both` (disk changed *and* the buffer has unsaved edits —
the three-way case). Creations are changes but not conflicts. A file rewritten
with identical bytes is not reported. Missing paths (e.g. a not-yet-created
include) are tracked so their creation is reported.

`Poller` wraps a snapshot: `poll()` returns changes and advances;
`run(interval, deadline, |result| keep_going)` loops on the calling thread.
There is deliberately no FSEvents dependency: the native app can call
`Snapshot::diff` from an FSEvents callback later and keep the same conflict
semantics.

### Crash recovery (`recovery.rs`)

```rust
let journal = RecoveryJournal::new(root);
let entry = journal.record(&path, unsaved_text, Some(base_hash))?;  // atomic
let listing = journal.list()?;            // entries sorted by path + malformed files
let check = journal.check(&entry)?;       // CurrentState::{Missing, MatchesBase, MatchesJournal, Diverged(hash)}, safe
journal.restore_to_disk(&entry, force)?;  // same conflict rules as save_atomic; discards on success
journal.discard(&path)?;
```

Files live at `<root>/.flashtex/recovery/<sha256_hex(path)>.json`:
`{schema_version:1, path, text, text_sha256, base_sha256|null,
saved_at_unix_ms}`. Entries are written with the same temp+fsync+rename path
as saves. On read, `text_sha256` is verified, so a torn or truncated entry is
reported as malformed rather than restored. Malformed files are listed, never
deleted. `restore_to_disk` refuses (returns the `SaveConflict`) when the disk
has diverged from `base_sha256`, or when a never-saved file now exists, unless
`force`. A file that already equals the journal text is treated as restored
without a write.

Recommended editor policy: record every N seconds while dirty and on focus
loss; discard on successful save; on launch, `list()` and present each entry
with its `check()` result.

## Guarantees

- Paths in the graph, documents, receipts and journal are normalized
  `ProjectPath`s; nothing this crate reads or writes resolves outside `root`
  (including via symlink, which is diagnosed rather than followed).
- Discovery output (file order, edges, diagnostics, documents, compile
  envelope) is a pure function of the file tree plus overlay.
- All byte spans are UTF-8 byte offsets into the exact text the graph holds
  (overlay text when supplied, disk bytes otherwise).
- SHA-256 output matches the FIPS vectors; `text_sha256`/`SaveReceipt.sha256`
  are hashes of the exact bytes written.
- A save either fully replaces the target with the new bytes or leaves the
  previous file intact; a refused save writes nothing. Temp files are removed on
  failure. Permissions of an existing target are preserved.
- Snapshot/diff never misses a content change whose mtime or size changed;
  a change that leaves both identical is caught on the next hash (see below).
- The recovery journal never restores over diverged content without `force`
  and never reports a corrupted entry as valid.

## Non-guarantees

- **No cross-process locking.** Two processes saving the same file interleave
  at rename granularity; the hash check is a compare-before-write, not a
  compare-and-swap. A writer that lands between `check_expected` and `rename`
  is overwritten. Native integration should serialize saves per path on one
  queue and treat the receipt's hash as the new baseline.
- **Rename atomicity is the filesystem's.** On APFS (and HFS+, ext4, XFS)
  `rename(2)` within one directory is atomic with respect to other readers;
  the directory `fsync` is best effort and ignored if the filesystem refuses.
  Network and FAT volumes may not honor this. Cross-directory moves are not
  attempted.
- **mtime granularity.** A same-size rewrite within the filesystem's mtime
  resolution (1 ns on APFS in practice, but coarser elsewhere) is not
  rehashed by `diff`. Callers can force a rehash by taking a fresh
  `Snapshot::take`.
- **Not the compiler.** The scanner does no macro expansion, no catcode
  changes, no `\import`/`\subfile`/`\InputIfFileExists`, and does not follow
  references inside `\newcommand` bodies or conditionals. Arguments containing
  macros are reported, not resolved. `\include` inside a `\includeonly`
  exclusion is still discovered.
- Graphics are hashed for identity but never parsed or validated.
- The poller is blocking and single-threaded by design; scheduling is the
  caller's.
- The journal directory is inside the project root and is not hidden from
  other tools; `.flashtex/` should be added to the user's VCS ignore rules by
  the native app if desired.

## Relationship to sibling crates (main at c89ca86)

- `crates/project-index` (lexical labels/citations/commands navigation) never
  reads files; it takes `(path, revision, text)` per document. `documents()`
  plus `RevisionTracker::file(path).revision` is exactly that input, and the
  normalized `ProjectPath` strings satisfy its path rule (no `.`/`..`
  components remain after normalization).
- `crates/edit-ledger` is a private per-document durable store
  (`document.json` + applied capture-edit IDs, exclusive file lock). It is the
  authority for a document's *live* text and receipts; a `.tex` file is an
  export of it. This crate is the `.tex`-side layer the ledger explicitly
  leaves to the Mac owner: discovering the rest of the project, exporting the
  ledger's text with `save_atomic` (hash-checked against external edits, which
  the ledger does not watch), and reporting external changes/deletions via
  `Snapshot::diff`. Where a document is opened in the ledger, the
  `RecoveryJournal` here is redundant for it; the journal remains useful for
  buffers that are not ledger-backed (included files, `.bib`).
- `crates/document-runtime` submits complete unsaved project snapshots per
  revision; `compile_payload(project_id, tracker.project_revision())` is that
  snapshot.

## Native project integration proposal (follow-up 2)

Current state on `agent/mac-claude-a/mac-shell` (`DocumentFiles.swift`):
one `.tex` file is opened with `String(contentsOf:)`, becomes `main.tex` in
the compile request regardless of its real name, `isDirty` compares strings,
and `saveTex` uses `String.write(atomically:)` with no hash or conflict check.

Proposed replacement, coordinated with the parent (`mac-claude-a`) before any
`Package.swift`/shared file change:

1. **Project root, not file.** "Open" picks a `.tex` file; the app derives
   `root = fileURL.deletingLastPathComponent()` and `entry = fileName` (a
   later "Open Folder" can ask for the entry). Security-scoped bookmarks cover
   the root.
2. **A Rust `project-files` service over the existing JSON Lines pattern**
   (the same shape as the bridge process in transfer-v1), or an FFI surface if
   the parent prefers in-process: `project_open{root, entry}`,
   `project_discover{overlay:[{path,text}]}` → `{files, diagnostics, documents,
   revision}`, `project_save{path, text, expected_sha256|null, force}` →
   `SaveReceipt` or `SaveConflict`, `project_poll{}` → `[ExternalChange]`,
   `recovery_list/record/check/restore/discard`. Every payload here is already
   producible from the crate's public types; a thin JSON adapter is the only
   new code.
3. **Compile requests come from `documents()`.** The shell stops synthesizing a
   one-element `documents` array; `compile_payload(project_id,
   tracker.project_revision())` is the request body, so the compiler (issue
   #8 / FT-002 file-aware `\input`) receives every reachable file with its
   real path and the editor's unsaved text via `Overlay`. Diagnostics from
   discovery (missing include, cycle) are shown in the same diagnostics list
   as compiler output, mapped through `span` with the existing UTF-8 → UTF-16
   conversion.
4. **Save path.** `saveTex` → `project_save(expected: lastReceipt.sha256)`.
   On `SaveConflict` show ours/theirs hashes, mtime and size with
   "Overwrite" (force) / "Reload" / "Save As" actions; never auto-force.
   The receipt hash becomes the new baseline and is fed to
   `record_own_write` so the poller stays quiet.
5. **External changes.** A 2 s poll on a background queue (or FSEvents later
   feeding `Snapshot::diff`). `ModifiedExternally` with a clean buffer reloads
   silently and recompiles; `Both`/`DeletedExternally` present the conflict
   sheet. This replaces the string-compare `isDirty`.
6. **Crash recovery.** While dirty, `record` every 5 s and on resign-active;
   `discard` after a successful save; on launch, `list()` → recovery sheet
   with `check()` results ("safe to restore" / "file changed since").
7. **Multi-buffer editor** follows naturally: the overlay is the set of open,
   dirty buffers keyed by `ProjectPath`; the transfer-v1 bridge's `document_*`
   messages already use the same path/revision/hash vocabulary.

Sequencing: (a) parent agrees on process-vs-FFI and message names, (b) I add
the JSON adapter and a `flashtex-project-files` binary under this crate,
(c) parent wires `DocumentFiles.swift` behind a feature flag, (d) native
acceptance on this Mac: open a nested project, edit an included file, save
with an external modification, recover after a forced kill.
