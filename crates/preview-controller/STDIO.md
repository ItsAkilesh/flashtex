# Native helper protocol v1

Run `flashtex-preview-controller CONFIG.json` as a local child. The JSON config
contains `session_id` (unique per launch), `project_id`, `entry_path`, `store_paths`
(1–256 initialized ledger directories), and optional `compiler_path` pointing to
the original FlashTeX compiler. Missing compiler leaves editing available. Ledger
source is authoritative; importing/exporting `.tex` and project membership remain
native adapter responsibilities.

Use dedicated native reader/writer workers; never synchronously wait for stdin,
stdout, fsync, compilation or replies on the UI thread. Input is JSON Lines with
`protocol_version:1`, configured `session_id`, a nonempty `id` up to 128 bytes,
`type`, and `payload`. A single actor serializes source operations while separate
threads drain pipes; compilation updates are polled independently every 2ms.

Admission is bounded: 1MiB input frames, 16 waiting operations, 8 output frames of
at most 16MiB each. Full input queues return a correlated `error` stating the
operation was not admitted. Drain responses before retrying that operation.
Output overflow/stall ends the helper with delivery uncertainty; recover source
and pending receipts before retrying writes. Normal stdin EOF drains queued
responses for at most two seconds, then exits; it does not promise pending compile
completion. Never interpret successful pipe writing as a durable edit receipt.

Responses use `type:ready`, `type:result`, `type:error`, or asynchronous
`type:update` with a null request ID. Result/error IDs correlate to requests.
Preview updates include compile request/revision, exact document `source_versions`,
compiler envelope and runtime/controller timings. Native code must compare its
current editor versions at paint time and suppress outdated updates. Native
keystroke/queue/paint timings remain additional measurements.

Operations:

- `document`: `{path}` returns authoritative document/hash/revision.
- `edit`: `{path, expected_revision, expected_sha256, text}` returns durable source
  and a separate `preview_error`; that error never means the edit was undone.
- `compile`, `restart`, `close`: empty payload. Restart uses the configured binary.
- `review`: `{edit: PreparedEdit}` checks current source and returns the exact edit
  plus an unpredictable session-local `approval_token`. At most 128 reviews remain.
- `apply_reviewed`: `{approval_token, user_approved:true}` is sent only from the
  actual user approval handler after displaying that exact edit. The helper cannot
  prove a human approved; automatic conversion must never manufacture this call.
  It returns a durable receipt/current source and separate preview status. Reusing
  the same token is idempotent through the durable edit ledger.
- `retire_review`: `{approval_token}` frees session-local review capacity.
- `recovery`: `{path}` returns pending applied transactions after uncertain delivery
  or restart. Tokens are not persisted, but source and receipts are.
- `confirm_receipt`: `{path, receipt}` only after exact bridge acknowledgement.

The helper has no provider access, automatic approval, native rendering or fallback
TeX engine. Tests use real durable stores and real helper subprocesses; the optional
original-compiler test also exercises streamed positioned results.

Editor queries are lexical project navigation, not a claim of TeX macro expansion:

- `snapshot`: `{}` returns `project_id` and the complete `source_versions` map.
- `complete`: `{source_versions, category, prefix, limit}` with category `label`,
  `citation` or `command`, literal prefix and limit 1–100. Returns matching project
  names and source locations, with explicit per-item location truncation flags.
- `navigate`: `{source_versions, path, byte_offset}` returns a source origin and
  lexical definitions or null. Offsets must lie on UTF-8 scalar boundaries.

Both queries require the exact current full version map and return it alongside
results. Refresh after a stale-version error and recheck current editor versions
before applying a result. Definition/occurrence locations are capped at 100 with
explicit truncation flags. This interface does not supply every package's built-in
completion vocabulary yet.

Durable editor history uses the shared ledger implementation:

- `history_status`: `{path}` returns undo/redo labels and retention usage.
- `undo` / `redo`: `{path, command:{command_id, expected_revision,
  expected_sha256}}`. Command IDs must be unique for a new action and reused
  unchanged when retrying that same action after uncertain delivery.
- `apply_group`: `{path, command:{command_id, expected_revision, expected_sha256,
  label, edits:[{start_byte,end_byte,removed_text,replacement}]}}`. Ranges refer to
  the same original snapshot and must not overlap; this is ordinary user editing,
  not a substitute for the reviewed capture approval boundary.

Responses contain the ledger `history` result, including current durable document,
command revision, retry flag and undo/redo availability, plus separate preview
status. Undo advances source revision; it does not rewrite old revision numbers.
Source, history and permanent retry IDs persist together. History capacity errors
are explicit; native retention UI is still required before history is full.

File-backed startup replaces `store_paths` with `project_root` and an existing
application-owned `private_ledger_root`. These modes are mutually exclusive.
The helper imports discovered source into private ledgers and preserves existing
ledger edits on restart, even when disk source has changed or disappeared.
`file_status:{path}` rereads disk and returns `matches_source`,
`differs_from_source`, `missing` or `unavailable`, plus discovery diagnostics.
It never implicitly reloads an external edit into the authoritative source.
`export:{path,expected_revision,expected_sha256,expected_disk_sha256}` writes
that exact durable source through the rooted project-files lock/save primitive.
The disk expectation is mandatory: a SHA-256 string for an existing file, or
explicit `null` for a new file. Stale source, conflicting disk content and symlink
components are refused. Success returns path, hash and byte count;
`export_available:true` advertises the operation, not a guarantee a save will pass.
An error can follow rename (including uncertain directory durability); inspect
disk before retrying. Cooperative writers serialize through the project lock.
Arbitrary external writers still have a recheck-to-rename race window; this is
not a universal filesystem compare-and-swap guarantee.

`configure_layout:{layout_capabilities:[...], renderer_support_confirmed:true}`
explicitly opts this client into negotiated runtime-v1 rules/font hints and submits
current source. Default startup remains legacy; an empty list restores legacy.
Call only after the native renderer implements the requested primitives. Each
preview reports `missing_layout_capabilities` if the compiler declined requests;
clients must surface that limitation. Accepted rule/font data remains unchanged
inside the compiler envelope, and switching capabilities invalidates retained
previews even when source text did not change. Negotiation does not establish
font-resource identity, pixel equality, or typing-to-painted-preview latency.

`search_literal:{source_versions,literal,max_matches,max_work,documents?}` performs
case-sensitive raw source search, including comments and verbatim text. Supply the
exact current version map; optional `documents` limits the search to named files.
`max_matches` is 1–1000 and `max_work` is 1–1,000,000 byte comparisons, including
query preprocessing. Results contain ordered UTF-8 byte ranges, `work_used` and
explicit `termination`: `complete`, `match_limit`, or `work_limit`. Partial results
must never be labeled exhaustive. Search is a bounded serialized operation; this
initial helper route does not implement mid-query user cancellation or regex.

`reload:{path,expected_revision,expected_sha256,expected_disk_sha256,user_approved:true}`
explicitly imports a reviewed disk snapshot into the durable source. Both source
identity and the freshly read disk hash must match. The helper takes the shared
project lock while reading and saving the ledger; arbitrary external writers can
still modify disk afterwards. The imported bytes are bounded UTF-8, and prior
source remains in persistent undo history. This operation never writes disk.
It returns document and separate preview status just like `edit`. A lost reply
requires reading `document` before retry; the old revision will be refused after
a successful reload. `user_approved` is a client responsibility, not proof of a
human action. Native UI must show the changes before confirming replacement.

`snapshot` additionally returns `membership_generation`. `open_document` and
`detach_document` require `{path,source_versions,membership_generation}` matching
that snapshot. Open imports an existing rooted UTF-8 file or restores its retained
private ledger, and returns `document`, current versions/generation and separate
preview status. Detach excludes a non-entry document for this session and returns
those membership fields with `document:null`. It never deletes a disk file or
ledger. Project restart restores all retained documents; persistent exclusions are
not implemented. Each membership change invalidates pending previews and index
snapshots. New includes are not yet discovered automatically after an edit.

A reply blocked in the output writer for two seconds terminates the helper even
when the output queue has not filled. The actor checks this deadline between
operations; it does not interrupt a filesystem call. Recover durable source after
uncertain delivery instead of assuming that the last operation was rejected.

`project_status:{max_documents?:1..256}` returns sorted active source metadata
(path, revision, SHA-256, UTF-8 byte count), current membership generation and
versions, total count and an explicit truncation flag. Default limit is 256.
`scope:"active_sources_only"` and `disk_tree_enumerated:false` distinguish this
from a complete disk tree: detached sources, assets and unopened disk paths are
not enumerated. No source text or disk contents are read by this query.
Compiler restart now advances index generation instead of resetting it, so a
previously invalid index snapshot cannot become valid again after restarting.

Optional startup `compiler_max_frame_bytes` accepts128..12,582,912 (12MiB),
default8MiB. Startup and compiler restart use the same limit. Ready advertises the
selected compiler limit and16MiB helper output limit. Twelve MiB leaves4MiB for
helper envelope/serialization headroom; the actual serialized helper bound is
still checked, so excess fails explicitly. This opt-in admits the measured~10MB
positioned result; it does not promise arbitrary document sizes or200ms latency.

The output queue remains8 frames with16MiB per-frame serialization bounds
(128MiB queued encoded payload, plus writer/producer buffers). Parsed JSON,
compiler memory and allocator overhead are additional; this is not an RSS cap.
The native client must continuously drain large replies off its UI thread.
Chunked or compact output requires a separate negotiated contract; no pages are
silently omitted under the current JSON protocol.
