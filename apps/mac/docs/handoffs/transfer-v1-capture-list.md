# transfer-v1 handoff: `capture_list` (journaled-capture discovery after a Mac relaunch)

Owner-scoped contract handoff from the Mac side (lane `mac-capture-list-handoff`, parent `mac-claude-a`,
mac-m1max-a, 2026-09-12) to the bridge owner (Commander, FT-007, `crates/bridge`,
`docs/contracts/transfer-v1.md`). Written under the Commander ruling on issue #2 (comment 5646362851): the
capture-list gap "needs an owner-scoped contract handoff, not a duplicate bridge implementation". This document
proposes; it changes no contract file, no bridge code and no Mac product code. Nothing below is implemented or
measured unless it cites an existing test or evidence file.

Status: proposal awaiting the bridge owner's decision. Consumer: FT-003 Mac (`BridgeSession`, `ShellModel+Bridge`).

## 1. The gap, with its reproduction

transfer-v1 has ten request types (`crates/bridge/src/main.rs` `dispatch`; mirrored by
`TransferV1.Request` in `apps/mac/Sources/FlashTeXProtocol/TransferV1.swift`): `document_open`,
`document_edit`, `destination_pin`, `capture_submit`, `capture_convert`, `capture_validate`,
`capture_prepare_insert`, `capture_applied`, `capture_status`, `capture_reject`. Every capture request is keyed by
a `capture_id` the caller must already know. There is no request that enumerates the journal.

Consequence on the Mac: `BridgeSession.captures` (surfaced as `ShellModel.bridgeCaptures`) is an in-memory
list filled only by this process's own `capture_submit` calls. It survives an *in-process* bridge relaunch
(`RealBridgeTests.testKilledBridgeIsRelaunchedWithJournalAndDestinationIntact`,
`BridgeRecoveryTests.testBridgeCrashIsRelaunchedAndStateReconciled`) but not an *app* relaunch: a fresh
`ShellModel` attached to the same `--store` knows no capture IDs, so it cannot even ask `capture_status`.

Reproduction (real helpers, no provider): `apps/mac/Tests/FlashTeXMacTests/CaptureAcceptanceTests.swift`,
`testAppRelaunchBetweenReceiptAndInsertKeepsOneJournaledCaptureWithoutDuplicates`, evidence
`docs/evidence/capture-acceptance-2026-09-12T1340Z.md` (run 2, 4/4, load 10.94/29.27/25.05):

1. Shell A attaches the real `flashtex-bridge` on store S, pins a destination, and a paired companion delivers
   capture `acceptance-relaunch-1` through the nearby listener → `capture_received {durable:true}`; one journal
   file `S/acceptance-relaunch-1.json`; one `bridgeCaptures` row (`.received`).
2. Shell A detaches ("quit"): the bridge process ends, the store directory survives, still one journal file.
3. Shell B (new `ShellModel`, "relaunch") attaches the same store. Observed and asserted at line 229:
   `model2.bridgeCaptures.isEmpty` — *"finding: no capture listing in transfer-v1; the relaunched shell shows
   nothing until a resend"*. `capture_status {acceptance-relaunch-1}` still answers (no proposal / prepared /
   applied, not rejected), proving the journal is intact and the only missing piece is discovery.
4. Only after the companion re-sends the identical capture (and the Mac has re-pinned the identical
   destination) does the row reappear, with the original receipt and still one journal file.

The journaled capture is therefore durable and idempotent, but invisible to the user until the companion
happens to resend. A companion that was put away, or that already deleted its local copy after the durable
acknowledgement, leaves the capture stranded in the journal.

The existing ledger-guarded restart reconciliation (`BridgeSession.reconcile`: edit-ledger `recovery_export` →
bridge `capture_status` per pending receipt → `recovery_import`) does not cover this case by design: the edit
ledger only has entries for captures that reached `capture_prepare_insert`. A capture that is journaled (with
or without a proposal) but never prepared has no ledger entry, so no Mac-side store can name it.

## 2. Consumer-side requirement

After an app relaunch (new process, same private store) the shell must be able to obtain, from the bridge
alone and without any companion resend:

- the set of journaled captures that belong to the current project and (optionally) the current target
  path, in a stable order, each with its durable identity and status, so that the review UI can show them as
  *pending review* and let the user decide;
- enough of each record's durable binding to tell the user whether the capture's target can be restored
  exactly or requires a new selection/capture, without the Mac guessing;
- a byte-exact digest of the journaled request so the Mac can recognise a companion resend as the same
  capture (and reject a differing one) before any bridge round trip;
- bounded replies (page size, truncation flag, cursor) so a large journal never produces a > 12 MiB line or
  blocks the UI.

Non-requirements, stated so the bridge owner does not over-build: the Mac does not need image bytes, proposal
text, context excerpts or prepared-edit bodies in the listing — `capture_status` already returns proposal /
prepared / applied / rejected per ID, and the listing only needs to be a discovery index that feeds
`capture_status`. The listing must never insert, convert, prepare, reject or otherwise mutate anything.

## 3. Proposed request / reply (additive to transfer-v1)

Envelope, ID rules, 12 MiB frame bound and error shape are unchanged. All new fields are snake_case.

### Request `capture_list`

```json
{
  "project_id": "demo",
  "store_id": "3f1c…64 hex…",
  "destination": { "path": "main.tex" },
  "since_receipt": "000000000000000017",
  "limit": 64,
  "include_terminal": false
}
```

| Field | Required | Meaning |
|---|---|---|
| `project_id` | yes | Session identity, same value the Mac passes to `document_open`/`destination_pin`. Only records whose durable `destination_binding.project_id` equals it are listed; legacy records without a binding are listed too, flagged `binding_missing:true` (they cannot be attributed to a project and can never be prepared — `storage_recovery::legacy_missing_binding_is_readable_but_cannot_prepare_or_redirect`). |
| `store_id` | no | Store identity check (see §4 "wrong store"). When present it must equal the journal's identity; the reply always echoes the actual value. |
| `destination` | no | Destination binding filter. Exactly one of: `{"path": "<relative path>"}` — durable, works after relaunch, matches `destination_binding.path`; or `{"destination_id": "<id>"}` — resolved against this process's in-memory anchors only (§4 "unbound destination"). Omitted: all paths of the project. |
| `since_receipt` | no | Cursor: the `receipt` of the last row already seen. Rows strictly after it are returned. Omitted: from the beginning. |
| `limit` | no | 1…256, default 64. |
| `include_terminal` | no | Default false: omit `inserted` and `rejected` rows (the ledger already reconciles receipts; the UI wants pending work). True: include them, for audit views and tests. |

### Reply `capture_list`

```json
{
  "project_id": "demo",
  "store_id": "3f1c…",
  "captures": [
    {
      "receipt": "000000000000000018",
      "capture_id": "acceptance-relaunch-1",
      "status": "journaled",
      "request_sha256": "9e0b…",
      "image_mime_type": "image/png",
      "image_bytes": 1234,
      "instructions_bytes": 15,
      "destination_id": "dest-1",
      "base_revision": 1,
      "destination_binding": {
        "project_id": "demo", "path": "main.tex", "revision": 1,
        "start_byte": 5, "end_byte": 5, "source_sha256": "f93b…"
      },
      "binding_missing": false,
      "has_proposal": false,
      "proposal_sha256": null,
      "prepared": null,
      "applied": null,
      "rejected": false,
      "reason": null
    }
  ],
  "next_receipt": null,
  "truncated": false
}
```

Row fields:

| Field | Source in the journal today | Notes |
|---|---|---|
| `receipt` | **new**, see "ordering" | Opaque, lexically ordered string; the cursor value. |
| `capture_id` | `CaptureRecord.capture.capture_id` | |
| `status` | derived | `journaled` (received; `proposal` may or may not exist; `prepared`/`applied` absent, not rejected), `staged` (`prepared` present, `applied` absent), `inserted` (`applied` present), `rejected` (`rejected:true`), `failed` (the record could not be loaded — `invalid_journal`: integrity/identity/derived-state check failed, oversized, truncated; `reason` carries the non-secret message). The requested four statuses plus `rejected`, kept separate because it is a durable terminal state in the journal and the Mac already renders it distinctly (`BridgeSession.CaptureState.rejected`). |
| `request_sha256` | `CaptureRecord.request_sha256` (already the integrity digest of the canonical `capture_submit` payload) | Byte-exact identity of the journaled request: a resend with the same `capture_id` and this digest is the idempotent retry the bridge already honours; a different digest would be `capture_id_conflict`. |
| `image_mime_type`, `image_bytes`, `instructions_bytes` | `capture.image.mime_type`, decoded base64 length, `capture.instructions.len()` | Cheap descriptors for the UI row; never the bytes themselves. |
| `destination_id`, `base_revision` | `capture.destination_id`, `capture.base_revision` | As submitted. `destination_id` is informational after a relaunch (anchors are in memory). |
| `destination_binding` | `CaptureRecord.destination_binding` | The immutable binding the contract already defines; `null` for legacy records. |
| `binding_missing` | derived | `destination_binding == null`. |
| `has_proposal`, `proposal_sha256` | `proposal.is_some()`, sha256 of `proposal.latex` UTF-8 | Lets the Mac see whether a paid conversion already exists without transferring the text (fetch via `capture_status`). |
| `prepared` | `{edit_id, expected_revision}` from `CaptureRecord.prepared` | Enough to cross-check the edit ledger; the full edit comes from `capture_status`. |
| `applied` | `{edit_id, new_revision}` | Same shape as `capture_status.applied`. |
| `rejected` | `CaptureRecord.rejected` | |
| `reason` | only for `failed` | Non-secret error message; never file contents. |

Reply-level: `next_receipt` is the `receipt` of the last returned row when `truncated:true`, else `null`.
`truncated` is true when more matching rows exist beyond `limit`.

### Ordering (`receipt`)

The journal today is a directory of `<capture_id>.json` files with no receipt time or sequence
(`crates/bridge/src/store.rs`; `save` is an atomic temp-file persist + directory fsync). File mtimes are not
durable evidence. Proposal: an additive `receipt_sequence: u64` on `CaptureRecord` (`#[serde(default)]`, so
existing records deserialize with 0 and the `request_sha256` integrity check — which covers only
`record.capture` — is unaffected). At `Store::open` the owner scans its directory once for the maximum
existing sequence — reading only that field leniently, so a corrupt record neither aborts the open nor loses its slot (one owner per journal is already enforced by `.bridge.lock`) — and `receive` assigns
`max + 1` to each *new* record before `save`. `receipt` is the zero-padded decimal sequence (18 digits), so
lexical order equals numeric order. Legacy records (sequence 0) sort first, tie-broken by `capture_id` bytes;
the cursor for them is `"000000000000000000:" + capture_id`. Sequence numbers are never reused or reassigned;
a rejected or applied record keeps its sequence.

Scanning rules for the listing: consider only regular files named `<id>.json` whose `<id>` satisfies
`identifier`; ignore `.bridge.lock`, `.bridge.id`, and `NamedTempFile` leftovers (the fixture
`storage_recovery::stale_temporary_files_never_replace_committed_record_or_acknowledge_new_capture` already
creates such files). A record whose `Store::get` fails is listed as `failed` with `reason` rather than hiding
evidence or failing the whole listing; the listing never deletes or rewrites anything.

## 4. Refusals

| Case | Code | Behaviour |
|---|---|---|
| `store_id` present and not equal to this journal's identity | `store_mismatch` | Nothing listed. Identity proposal: a `.bridge.id` file (32 random bytes hex) created by `Store::open` when absent, read otherwise; also echoed in every `capture_list` reply so the Mac can pin it at attach and detect a store swap (different `--store`, restored backup) across a relaunch. |
| `destination.destination_id` given but no in-memory anchor of that ID (e.g. any time after a relaunch before a re-pin) | `destination_unbound` | Explicit, so the Mac learns to filter by `path` after a relaunch and by `destination_id` only for live pins. The anchor's *validity* is not required — an invalid anchor still filters; validity is reported per row by comparing bindings on the Mac. |
| `destination` with both or neither key, or a `path` that fails `relative_path` | `bad_request` | |
| `limit` outside 1…256 | `invalid_limit` | No silent clamping. |
| `since_receipt` not a receipt this store could have issued (wrong shape) | `invalid_cursor` | A well-formed cursor past the end returns an empty list, not an error. |
| `project_id` fails `identifier` | `bad_request` | |
| Journal directory unreadable | `store_unavailable` | Transient for the Mac (`isTransient`), retried like a status failure. |

None of these refusals mutates state. The listing has no side effects on anchors, documents or records.

## 5. Idempotence and relation to `capture_status` and `recovery_import`

- `capture_list` is read-only and repeatable; two consecutive calls without intervening `capture_submit` /
  `capture_prepare_insert` / `capture_applied` / `capture_reject` return identical rows and cursors.
- `capture_status` stays the authoritative per-capture view (full proposal, prepared edit, receipt). A row
  from the listing is an index entry; the Mac always fetches `capture_status` before showing a proposal or
  cross-checking an edit, exactly as it does today for ledger-pending receipts.
- `recovery_import` belongs to the edit-ledger helper and reconciles *issued edits* (`staged` / `inserted`
  rows) against the Mac's durable transaction ledger. The listing does not replace or reorder that step: the
  Mac runs `reconcile` first (as today), then lists. For `staged` / `inserted` rows the listing only adds a
  cross-check: a `staged` row whose `edit_id` has no ledger transaction is reported as needing explicit
  reconciliation (the bridge issued an edit this Mac never recorded — the contract's "reconcile before a new
  attempt" case); an `inserted` row with a matching confirmed transaction is displayed as `.confirmed`.
- `journaled` rows are the new information. Listing one does not change what `capture_submit` of the same
  payload does (idempotent original receipt) nor what a differing payload does (`capture_id_conflict`).
- `capture_reject` remains the only way a user retires a stranded `journaled` capture; the listing does not
  expire, prune or auto-reject anything, and a `rejected` row stays in the journal (listed with
  `include_terminal:true`).

## 6. What the Mac client would do with it (state-machine sketch, not implemented)

New `BridgeSession.CaptureState.pending` ("journaled before this session; not yet reviewed here").

```
attach(store)
  → openLedger (existing)                 durable document adopted/aligned
  → reconcile (existing)                  ledger-pending receipts settled through capture_status/recovery_import
  → document_open (existing)
  → capture_list {project_id, store_id?, destination:{path: activePath}, limit:64}   ← NEW, after open
      per row, merged into `captures` by capture_id (never a duplicate row; rows this session already
      created via its own capture_submit keep their state):
        journaled, !binding_missing  → .pending   note "received before relaunch; review to continue"
        journaled,  binding_missing  → .needsReselection  note "legacy record: capture again"
        staged  (edit_id in ledger)  → left to reconcile's outcome (.prepared/.applied/.confirmed)
        staged  (edit_id unknown)    → .needsReselection + captureNote "issued edit never recorded here — resolve"
        inserted                     → .confirmed if the ledger has the confirmed receipt, else note (audit)
        rejected                     → .rejected
        failed                       → .failed   note = reason (evidence kept, nothing hidden)
      truncated:true → repeat with since_receipt until false (bounded: at most 8 pages per attach; beyond
                       that captureNote says "more journaled captures; open the capture list to load more")
      any refusal/transport failure → captureNote; bridgeCaptures unchanged; Edit > Retry Bridge Reconciliation
                       also re-runs the listing. Never blocks attach or typing (runs on the session queue).

user action on a .pending row (review sheet lists it with image descriptors and binding summary):
  "Restore destination" — enabled only when sha256(activeText) == binding.source_sha256 and
      editorRevision == binding.revision (the ledger aligns revisions across relaunches when nothing changed):
      destination_pin {destination_id (from the row), path, revision, start_byte, end_byte} → the bridge
      accepts only an identical binding (lib.rs `pin` / `capture_anchor`;
      bridge test `exact_anchor_rehydration_allows_review_after_restart`) → row becomes .received and the
      normal flow continues: Convert (explicit, provider-gated, never automatic) → proposal → review sheet →
      explicit Approve → capture_prepare_insert → local verification → ledger apply → adopt → capture_applied.
  otherwise the row shows "target changed since the capture; capture again or reject" — the contract's
      reselection rule (transfer-v1 "Review, insertion and crash reconciliation": a stale/invalid destination
      needs a new capture ID/destination) — with Reject (capture_reject) as the only bridge call offered.
  has_proposal:true → "Show proposal" fetches capture_status and queues the proposal in the existing review
      sheet (no provider call); insertion still requires the restored destination and explicit approval.

invariants: no listing result ever calls capture_convert, capture_prepare_insert or the ledger's apply;
  no auto-insert; no auto-reject; the companion resend path is unchanged (a resend of a .pending row's
  capture_id with the same request_sha256 is acknowledged with the original receipt and leaves one row).
```

Mac-side data model changes this implies (for the parent, not done here): `TransferV1.Request.captureList`
(`"capture_list"` → reply `"capture_list"`), `TransferV1.CaptureList{Request,Reply,Row}` Codable structs,
`BridgeSession.list(path:since:limit:)`, `CaptureState.pending`, a merge step in `attachBridgeAndWait`
after `session.open`, and review-sheet rows for `.pending`. `ShellModel+Bridge.swift` is parent-retained;
the listing merge should live in a new `ShellModel+CaptureList.swift` with a single call site hook.

## 7. Bounded acceptance plan — bridge owner (`crates/bridge`)

All with the existing deterministic fixtures (`tests/bridge.rs::setup`/`capture`, `tests/storage_recovery.rs::
bridge`/`capture`/`record_path`, `tests/cli.rs::run`/`request`); no provider, no network. Suggested names:

1. `capture_list_after_reopen_shows_journaled_capture_once` (storage_recovery): receive → drop bridge →
   reopen store → `capture_list {project_id}` = one `journaled` row with the same `request_sha256` as the
   file's, `receipt` non-empty, `truncated:false`; a second identical `receive` keeps one row and the same
   `receipt`.
2. `capture_list_status_transitions` (bridge.rs, fake converter): journaled → convert (`has_proposal`,
   `proposal_sha256`) → prepare (`staged`, `prepared.edit_id`) → confirm (`inserted`, hidden unless
   `include_terminal`) ; separate capture → reject (`rejected`, hidden unless `include_terminal`).
3. `capture_list_orders_by_receipt_and_paginates` (bridge.rs): 5 captures, `limit:2` → 2 rows +
   `truncated:true` + `next_receipt`; following the cursor twice yields the remaining 3 in receipt order with
   `truncated:false`; cursor past the end → empty, no error; `limit:0` / `limit:257` → `invalid_limit`.
4. `capture_list_legacy_and_corrupt_records_are_flagged_not_hidden` (storage_recovery): a legacy record
   without `destination_binding` and without `receipt_sequence` → `binding_missing:true`, sorted first;
   a truncated/non-JSON record and a wrong-digest record → `failed` rows with `reason`, other rows still
   listed; stale temp files and `.bridge.lock` ignored; the listing leaves every file byte-identical
   (reuse the evidence-preservation assertions of
   `truncated_and_non_json_records_fail_closed_without_overwriting_evidence`).
5. `capture_list_refusals` (bridge.rs): `store_id` mismatch → `store_mismatch`; `destination_id` after
   reopen (no anchor) → `destination_unbound`; after an exact re-pin the same filter lists the row;
   `path` filter excludes other paths; foreign `project_id` → empty list (legacy rows still flagged).
6. `capture_list_wire_round_trip` (cli.rs): one `capture_submit` + `capture_list` over stdin/stdout, reply
   `type:"capture_list"`, ID preserved; malformed `destination` → `bad_request` with the request ID.
7. `receipt_sequence_survives_reopen_and_never_reuses` (storage_recovery): sequences 1..3, reject 2, reopen,
   receive a 4th → sequence 4; listing order 1,2(rejected, terminal),3,4.

Contract text: one additive subsection in `docs/contracts/transfer-v1.md` ("Journal listing"), the row/reply
tables above, the read-only guarantee, and the note that `receipt` order is receipt order, not review order.

## 8. Bounded acceptance plan — Mac side (parent lane, after the bridge ships)

Existing tests to extend (real helpers XCTSkip without env; fake bridge `Fixtures/fake_bridge.py` gains a
`capture_list` handler over its in-memory journal so the fake-bridge suites stay hermetic):

- `BridgeClientTests.testEveryRequestTypeRoundTripsWithSnakeCaseKeys`: add `capture_list` request/reply
  encoding (snake_case keys, `next_receipt` null vs string, `truncated`).
- `CaptureAcceptanceTests.testAppRelaunchBetweenReceiptAndInsertKeepsOneJournaledCaptureWithoutDuplicates`:
  flip line 229 — after the relaunch `bridgeCaptures` contains exactly one `.pending` row for
  `acceptance-relaunch-1` with the journal's `request_sha256`; the companion resend still returns the
  identical receipt and leaves one row (now `.received`); nothing inserted (`assertNothingInserted`).
- `RealBridgeTests`: new `testRelaunchedShellListsJournaledCaptureAndRestoresIdenticalDestination` — list,
  "Restore destination" re-pin on identical text/revision → `.received`; prepare still `proposal_missing`
  (no provider); a changed buffer → row `.needsReselection`, no pin sent.
- `BridgeRecoveryTests.testBridgeCrashIsRelaunchedAndStateReconciled`: after the in-process relaunch the
  listing merge must not duplicate rows the session already holds.
- `ShellModelBridgeTests` (fake bridge + fake ledger): a listed `staged` row whose `edit_id` is absent from the
  ledger surfaces as needing explicit reconciliation and never applies; a listed row with `has_proposal:true`
  shows the proposal only on "Show proposal" and inserts only on explicit approval.
- Load-aware timing: none needed; the listing is one round trip. Any timing claim must record `uptime`.

Mac product-file diffs are for the parent (`ShellModel+Bridge.swift` hook), not this lane.

## 9. Cross-reference (not in scope here)

The acceptance evidence's finding 2: `NearbyWire.captureInputErrorCodes`
(`apps/mac/tools/nearby-client/Sources/NearbyClient/NearbyWire.swift`) omits
`destination_reselection_required`, so `NearbyError.needsNewCapture` is false for the real bridge's refusal of
a capture bound to a changed anchor, and the reference CLI does not phrase it as "capture again". Lane
`mac-nearby-errors` (`origin/agent/mac-nearby-errors/nearby-errors`) owns that fix; this handoff only notes
that the `capture_list` design assumes the same code stays the bridge's answer for a stale binding, and the
Mac's "Restore destination" gate above is what keeps the companion from ever seeing it for a listed capture.

## 10. Limitations of this handoff

- Nothing is implemented or benchmarked; sizes (limit 256, 18-digit receipt) are proposals for the owner to
  adjust.
- The `store_id` file and `receipt_sequence` field are additive and legacy-tolerant by construction, but the
  owner decides whether the journal schema version stays 1.
- Proposal text is deliberately not in the listing; if the owner prefers a single round trip for the review
  sheet, `include_proposals:true` could add the `capture_status` proposal object per row under the existing
  size bounds (≤ 64 KiB LaTeX each × 256 rows is still under the 12 MiB frame, but only just).
