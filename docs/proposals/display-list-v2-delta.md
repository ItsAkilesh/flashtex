# Proposal: `display-list-v2-delta` — a bounded, opt-in delta sibling for `display-list-v2`

Status: **PROPOSAL ONLY (producer side, mac-render-pipeline lane; written by the
mac-display-delta-proposal lane on `agent/mac-render-pipeline/delta-proposal`
from `agent/mac-render-pipeline/unified` @ 9aaec57a). Not on the wire. No
producer code, no runtime/helper code, no edits to Commander-owned contract files
(`docs/contracts/*`, `crates/preview-controller/*`, `crates/rendering-core/*`).
Requested by the Commander (issue #2 comment 5646362851) as one reviewable
producer/consumer schema + acceptance plan, to be co-signed by the consumer
side (mac-helper-display / Mac shell) before any wire activation.**

Base contracts, all unchanged by this proposal: runtime-v1, its layout
capabilities, `docs/contracts/runtime-v1-display-list-v2.md` (the full sibling
line and its decline fallback), `crates/preview-controller/docs/display-forwarding.md`,
`crates/document-runtime/docs/display-candidates.md`, `producer-size-contract.md`.

## 0. Ten-line summary (for relay)

1. New sibling opt-in layout capability `display-list-v2-delta`; only meaningful next to `display-list-v2`; unknown to old producers, never sent by old consumers; declined → today's full `display_list` line or today's `display_list_declined` fallback, byte-for-byte unchanged.
2. When accepted, the ONE sibling line after `compile_result` is `type: "display_list_delta"` (protocol_version 2) instead of `display_list`; the echoed capability list says which of the two the consumer must expect — a mismatch is a protocol violation.
3. A delta names its base exactly: the producer's last emitted sibling on this stream (`request_id`, `project_id`, `revision`, `page_count`, `list_digest`); both sides retain exactly one snapshot; any reply without a sibling (not requested, declined, `failed`), a restart or a session change clears both.
4. A delta carries the complete header (`documents`, `fonts` = the full resource closure, `diagnostics`, `required_features`), the full ordered `page_count`, a digest for EVERY page of the new list, the changed pages in full (exact page objects of the full reply), the explicit `removed_pages`, and per-document source relocations (`edit_start`, `edit_end`, `delta`) so unchanged pages' source spans are moved exactly as the producer's own block cache moves them.
5. Digests (`dl2-canon-1`) are SHA-256 over a specified canonical binary encoding of the semantic model (glyph runs, rules, clusters, hit rects, carets, source spans, paint, fonts, documents, diagnostics) — implementable identically in Rust, Swift and the 60-line Python reference in Appendix A, with test vectors.
6. Reconstruction = changed pages as sent + relocated base pages, then the normal full validation. Invariant: `to_json(reconstructed)` is byte-identical to a fresh full compile's `display_list` line (same id), including unchanged pages, resources, source spans and diagnostics — proven with the existing `tests/incremental.rs` gate shape (200 edits × 27 pages; 30 × 107).
7. Refusals are typed: the producer never emits a delta without a snapshot of its own (no snapshot → the unchanged full line, `status` stays `ok`, no diagnostic); the consumer verifies the delta's `base` against the snapshot it holds and refuses `delta_base_mismatch`, `delta_page_count`, `delta_relocation_invalid`, `delta_digest_mismatch`, `delta_list_digest_mismatch`, `delta_oversize`, `delta_unsolicited` and resyncs by requesting without `-delta` (→ full).
8. Bounded state: one retained snapshot per side, capped (`MAX_SNAPSHOT_PAGES` 1 024, retained model of a line that fit the line limit, previous document texts ≤ 8 MiB each); over the cap → no snapshot → full replies only.
9. Visibility page filtering is NOT this proposal: a filtered view is an incomplete view, never a complete compile, and never authorizes source actions outside its validated coverage; a reconstructed delta result IS a complete compile because it is verified equal to one.
10. Acceptance is three separate gates — producer (cargo, byte identity + digests + refusals), consumer (Swift, reconstruction equality + typed refusals + V2Parity 0 px), transport (size/latency measured on the direct route and, after an FT-049 runtime change to accept the new sibling type, the helper route) — nothing about size or latency is claimed from this schema.

## 1. Why (measured, not promised)

`crates/render-pipeline/docs/oracle-evidence.md` at 9aaec57a: with all three
caches a one-word edit on the 27-page document costs 27.1 ms in-process, 33.6 ms
wall over the protocol; of that, ~12 ms is producing the reply lines and ~6 ms
is request parse plus moving ~4.3 MB of reply through the pipe. The consumer
then decodes and validates the whole frame again. The runtime note
`producer-size-contract.md` shows the other end of the scale: a 26-page fixture
whose v2 line is 25 120 854 bytes is declined at 16 MiB and would exceed the
runtime's 8 MiB framed default anyway. A delta reply that carries only what
changed is the remaining lever the producer lane identified; the Commander
ruled that it must be bounded, opt-in, digest-verified, and must never replace
or truncate the full reply. This document is that ruling turned into a schema.

Explicitly out of scope: the runtime-v1 `compile_result` (the product preview)
stays a full reply; nothing here changes its bytes or size. Page filtering by
visibility (§9) is a different feature and is not proposed.

## 2. Identity (immutable, all present on every delta)

| identity | where | meaning |
|---|---|---|
| document/source | `documents[]` = `{path, revision, sha256, byte_length}` of EVERY request document (complete set, as in the full line) | raw UTF-8 SHA-256 and length of the request text this compile ran on; `revision` is the compile revision (runtime-v1), not an editor document version |
| compile | envelope `id` = the compile request id; `project_id`, `revision` = the `compile_result`'s | the same rule 1 as the full sibling |
| session | not on the wire (the full line has none and stays unchanged); bound by the consumer to the transport it owns: the worker process it spawned (direct route) or the helper `session_id` (helper route) | a restarted producer has no snapshot and answers full; a consumer drops its base on any restart/reattach/session change |
| base snapshot | `base = {request_id, project_id, revision, page_count, list_digest}` | exactly the producer's last emitted sibling on this stream (full `display_list` or the full list a previous delta reconstructed); verified field-by-field by the consumer against the snapshot it holds |
| new snapshot | `page_count`, `page_digests[1..N]`, `list_digest` | what the reconstructed list must hash to; becomes the next base on both sides |

`list_digest` binds the header (including `project_id`/`revision`, documents,
fonts, diagnostics, features) and every page digest in order, so two snapshots
with the same pages but different diagnostics or revision are different bases.

## 3. Negotiation

- Request: `payload.layout_capabilities` gains the string `display-list-v2-delta`.
  No new request field. Old producers ignore it (unknown names are never
  accepted). A request listing `-delta` without `display-list-v2` is legal but
  `-delta` is never accepted for it.
- Consumer rule: request `-delta` only while (a) the v2 pane is consumable,
  (b) the consumer holds a base, and (c) it has applied every sibling the
  producer emitted since that base (§6). Otherwise request `display-list-v2` alone.
- Producer decision per request, in order: capability requested and
  `display-list-v2` accepted and result non-`failed` → has a snapshot →
  document set (paths) identical → relocation computable (§5.3) → delta
  serialised and ≤ line limit → delta smaller than the full line would be
  (policy; the crate constant, proposed 3/4) → **accept**: echo both
  `display-list-v2` and `display-list-v2-delta`, emit ONE `display_list_delta`
  line. Any step fails → **do not accept**: echo omits `-delta`, and the reply
  is exactly today's: the full `display_list` line, or the existing
  `display_list_declined` warning + `recovered` when that would be oversize.
- Acceptance is bound to the applied result (layout-capabilities contract);
  a stale reply never changes the consumer's mode.
- Ordering on the stream is unchanged: `compile_result(id)` then the sibling
  `(id)`, contiguous, before any reply to a later request. Exactly one sibling
  per accepted non-failed result, of the type the echo announced.

The producer emits no diagnostic when it simply has no matching snapshot
(first request, after a restart, after a cleared snapshot) — that is the
normal path and the full line is the complete answer, so `status` stays `ok`.
The one anomalous case, a consumer that requested `-delta` while the
producer's snapshot exists for a different `request_id` chain than the consumer
can hold (only possible if a sibling was dropped in transit and the consumer
violated §6), cannot be detected by the producer (it never sees the consumer's
base); it is detected by the consumer as `delta_base_mismatch` (§7).

## 4. Wire shape of `display_list_delta`

```
{"protocol_version":2,"id":"<compile request id>","type":"display_list_delta","payload":{
  "render_format":"display-list-v2",            -- the format of the RECONSTRUCTED list
  "coordinate_unit":"bp_2pow20","color_space":"srgb","text_extraction":"cluster-actualtext",
  "project_id":"…","revision":N,                -- this compile (= compile_result)
  "required_features":[…],                      -- of the reconstructed list
  "digest_scheme":"dl2-canon-1",                -- §5.1; unknown scheme → refuse
  "base":{"request_id":"…","project_id":"…","revision":N-1,"page_count":B,"list_digest":"<hex64>"},
  "documents":[…],                              -- complete, as in the full line
  "fonts":[…],                                  -- complete resource closure of the reconstructed list
  "diagnostics":[…],                            -- complete, as in the full line
  "relocations":[{"path":"main.tex","edit_start":a,"edit_end":b,"delta":d}, …],   -- ≤ 1 per document
  "page_count":N,                               -- full ordered page count of the reconstructed list
  "page_digests":["<hex64>", … N entries],      -- page 1..N, ALL pages
  "changed_pages":[<page>, …],                  -- full page objects, ascending number, exactly as the full line would carry them
  "removed_pages":[B'+1, …, B],                 -- explicit; must equal exactly the base numbers > N
  "list_digest":"<hex64>"                       -- of the reconstructed list
}}
```

Rules:

1. `changed_pages[].number` are distinct, ascending, within `1..N`. Unchanged
   pages are `{1..N} \ changed`, and each must satisfy `number ≤ base.page_count`.
   Page numbers are contiguous `1..N` in every list (rendering-core rule), so
   `removed_pages` is redundant with `base.page_count` and `page_count` on
   purpose: the consumer verifies it equals `[N+1 .. base.page_count]` (empty
   when `N ≥ base.page_count`) and refuses otherwise.
2. `page_digests[i-1]` is the `dl2-canon-1` page digest of page `i` of the
   reconstructed list (changed and unchanged alike).
3. `fonts` is the complete closure: every `font_id` referenced by any page of
   the reconstructed list, unchanged pages included, is declared, and nothing
   undeclared is referenced (checked by the unchanged full validator on the
   reconstructed list). Fonts are small (≤ 256 × ~400 B) and are never
   delta-encoded.
4. `documents` and `diagnostics` are always complete, never delta-encoded
   (diagnostics carry relocated source spans exactly as the full line would).
5. `relocations` has at most one entry per declared document path and only
   for paths in `documents`; `0 ≤ edit_start ≤ edit_end ≤ base byte_length of
   that document`, `delta = new byte_length − old byte_length` of that document;
   a document without an entry has the same `sha256`/`byte_length` in the base.
6. Bounds (mirroring `RenderingV2.Bounds` / rendering-core): `page_count`
   0..10 000; `changed_pages` ≤ `page_count`; `relocations` ≤ `documents`
   (1..4 096); every hex digest exactly 64 lowercase hex; the whole line within
   the producer's line limit (16 MiB, `JSONLines.maxLineBytes`) — the
   consumer's framing limit is separate and is not negotiated here (see
   `producer-size-contract.md`).

## 5. Semantics

### 5.1 Digests — `dl2-canon-1`

SHA-256 over a canonical binary encoding of the semantic model, NOT over JSON
bytes (the consumer does not have the producer's JSON writer and must be able
to verify a page it relocated itself). Encoding primitives: integers as 8-byte
two's-complement little-endian (`i64`); counts as `i64`; strings as `i64` byte
length then UTF-8 bytes; paint components as IEEE-754 binary64 little-endian
bit patterns. Domain-separated prefixes:

- `page_digest(page) = SHA256("flashtex:dl2:page:1\0" ‖ number ‖ width ‖ height ‖ item_count ‖ items…)`
  - glyph run: `0x01 ‖ font_id ‖ font_size ‖ text ‖ glyph_count ‖ (gid, origin_x, baseline_y, advance_x, advance_y, cluster)… ‖ cluster_count ‖ per cluster (text_start_byte, text_end_byte, hit_rect_count, (x, top, width, height)…, caret_count, (text_byte, x, top, height)…, provenance) ‖ paint(r, g, b, a)`
  - rule: `0x02 ‖ x ‖ top ‖ width ‖ height ‖ paint ‖ provenance`
  - provenance: sources → `0x10 ‖ count ‖ (path, start_byte, end_byte)…`; synthetic → `0x11 ‖ reason`
- `header_digest(list) = SHA256("flashtex:dl2:header:1\0" ‖ render_format ‖ coordinate_unit ‖ color_space ‖ text_extraction ‖ project_id ‖ revision ‖ features(count ‖ str…) ‖ documents(count ‖ (path, revision, sha256, byte_length)…) ‖ fonts(count ‖ (font_id, sha256, byte_length, format, face_index, units_per_em, glyph_count, postscript_name)…) ‖ diagnostics(count ‖ (code, message, severity, sources(count ‖ (path, start_byte, end_byte)…))…))`
- `list_digest(list) = SHA256("flashtex:dl2:list:1\0" ‖ header_digest (32 raw bytes) ‖ page_count ‖ page_digest[1] ‖ … ‖ page_digest[N] (32 raw bytes each))`

Every field of the wire model is covered (glyph runs, rules, clusters, hit
rects, carets, source spans and synthetic reasons, paint, fonts, documents,
diagnostics, features, identity). The reference implementation is Appendix A;
its test vectors are the worked example's digests (Appendix B). Both producer
(Rust) and consumer (Swift) implementations must reproduce those vectors; a
scheme change is a new name (`dl2-canon-2`), never a silent change.

### 5.2 Relocation (what "unchanged page" means)

The producer's block cache (`src/incremental.rs`) reuses a block after an edit
by moving every source offset by the block's byte delta (`relocate_block`,
`place_item`). A page after the edit is therefore typically identical to the
base page except that every source span moved by the same integer. The delta
encodes exactly that move per document as `{edit_start: a, edit_end: b,
delta: d}` in the BASE document's byte coordinates (`[a, b)` is the replaced
region of the old text; the new region is `[a, b + d)`); the producer computes
it as the common-prefix/common-suffix diff of the previous and new request
text of that document (`a` = common prefix length; `b` = old length − common
suffix length, with `a + suffix ≤ min(old, new)`; `d` = new − old).

Applying a relocation to one `SourceRange {path, start_byte, end_byte}` whose
`path` has an entry:

```
if end_byte <= a:        unchanged
elif start_byte >= b:    start_byte += d; end_byte += d
else:                    INVALID for an unchanged page (the span intersects the edited region)
```

Applied to every source span of the page: cluster provenance (`sources`,
one or several ranges), rule provenance, and — for the header — nothing
(diagnostics are sent complete). Synthetic provenance and spans in documents
without an entry are untouched. `a == b` is a pure insertion; `d < 0` a net
deletion. The producer classifies a page as unchanged only when
`relocate(base_page) == new_page` as a model (Rust `PartialEq` on `Page`) and
the relocation is valid for every span on it; otherwise the page is sent in
`changed_pages`. Because `to_json` is a pure function of the model, model
equality is byte equality of the page's JSON.

### 5.3 When the producer must send full instead

Any of: no snapshot; `project_id` differs; the set of document paths differs;
a document's previous text is not retained (over the retention cap); the delta
line would exceed the line limit; the delta is not smaller than the full line
by the policy factor. The multi-edit case (two distant edits, a paste) is
handled by the same rule — `[a, b)` simply spans both edits and every page
with a span inside it is a changed page; the size policy then decides.

### 5.4 Reconstruction (consumer)

```
verify base == held snapshot (all five fields)      else delta_base_mismatch
verify digest_scheme known                           else delta_unsupported_scheme
verify page_count, changed numbers, removed_pages    else delta_page_count / delta_removed_pages
for n in 1..=page_count:
    page = changed[n] if present else relocate(base.pages[n], relocations)   (invalid span → delta_relocation_invalid)
    verify page_digest(page) == page_digests[n-1]    else delta_digest_mismatch(n)
list = header fields from the delta + pages
verify list_digest(list) == delta.list_digest        else delta_list_digest_mismatch
run the UNCHANGED full validation on `list` (RenderingV2.validate / rendering-core validate_display)
```

Only then does `list` replace the base; painting uses the existing v2 pipeline
(font resolution by content hash, page preparation off-main, identity recheck
before paint) exactly as for a full frame. Nothing is rendered partially: any
failure keeps the previous verified frame and the base is dropped (§7).

## 6. Retained state, lifecycle, bounds

**Producer (one snapshot per stream):** the last emitted sibling's `DisplayList`
model, its `page_digests`/`list_digest`, its `request_id`, and the request
document texts it was compiled from (needed for the relocation diff).
Replaced by every emitted sibling (full or delta). Cleared by: a reply with no
sibling (capability not requested, `display-list-v2` declined, `failed`),
`project_id` change, process exit. Caps: retained only if `page_count ≤
MAX_SNAPSHOT_PAGES` (proposed 1 024), the emitted line fit the line limit
(always true when emitted), and total retained document text ≤ 32 MiB (each
≤ 8 MiB, the rendering-core document bound); beyond a cap → no snapshot →
full replies only. The block caches in `incremental.rs` are unrelated and
keep their own bound (`MAX_BLOCKS`). Memory of the snapshot is the model of a
line that fit in 16 MiB; the actual RSS delta is a measurement item (§10.3),
not a claim.

**Consumer (one base per transport):** the verified list (it already retains
the applied v2 frame), its digests, `request_id`, and the transport identity
(worker launch generation / helper `session_id`). Replaced by every verified
sibling. Dropped by: a `compile_result` whose accepted capabilities lack
`display-list-v2` (declined, not requested, `failed`), any refusal in §7, v2
pane hidden (the consumer then stops requesting both capabilities), worker
restart/relaunch, helper restart/reattach/session change, project change. The
same `MAX_SNAPSHOT_PAGES` cap applies.

**Ordering under pipelining:** the stream is FIFO. A consumer with one request
in flight sends N+1 holding base N−1; the producer answers N+1 against N (its
snapshot after emitting N's sibling); the consumer receives sibling N first,
applies it (base N), then applies N+1. So one snapshot per side suffices,
provided the consumer applies EVERY sibling in order, including ones it will
not paint because a newer result is already applied ("stale" siblings are
applied to the chain off-main but never published — this splits the existing
"stale sibling is dropped" rule into "not painted" vs "not applied").

**Reset/cancel/restart:** runtime-v1 has no cancel; superseded requests are
still answered in order and are part of the chain. A consumer that cannot
apply a sibling (helper dropped it as oversize/busy — allowed by
`display-forwarding.md`; validation failure; decode failure) drops its base and
sends its next request without `-delta` (full resync, §8). A restarted
producer has no snapshot and answers full; a restarted helper session requires
the existing fresh opt-in (`configure_display_candidates`), which also drops
the base.

## 7. Typed refusals (consumer) and the wrong-base rule

| code | when | action |
|---|---|---|
| `delta_unsolicited` | `display_list_delta` line while `-delta` was not accepted for that id, or `display_list` while it was | protocol violation, as today for unexpected v2 lines |
| `delta_unsupported_scheme` | `digest_scheme` ≠ `dl2-canon-1` | refuse, resync |
| `delta_base_mismatch` | any of the five `base` fields ≠ held base, or no base held | refuse, resync; counted separately as it indicates a chain break |
| `delta_page_count` / `delta_removed_pages` | bounds, ordering, or `removed_pages` ≠ `[N+1..B]` | refuse, resync |
| `delta_relocation_invalid` | a relocation names an undeclared path, violates §4 rule 5, or an unchanged page's span intersects the edited region | refuse, resync |
| `delta_digest_mismatch(n)` | a page digest differs after reconstruction | refuse, resync; evidence-worthy: reconstruction or producer classification bug |
| `delta_list_digest_mismatch` | header/list digest differs | refuse, resync |
| `delta_oversize` | line over the consumer's framing budget | dropped before parse (existing behaviour), resync |
| existing full-validation errors | the reconstructed list fails `RenderingV2.validate` | refuse, resync (same codes as a full frame) |

"Refuse" = keep the previous verified frame on screen (labelled stale as
today), publish nothing, log the code, drop the base. "Resync" = the next
compile request omits `display-list-v2-delta`; its reply is the unchanged full
sibling (or the unchanged decline), which re-establishes the base.

## 8. Full resync path

Always available and always the unchanged contract: request `display-list-v2`
without `-delta` → full `display_list` line (or `display_list_declined`).
Used on: no base, any refusal, any dropped sibling, any restart. A document
whose full line is declined (oversize) never obtains a base and therefore never
gets a delta — the delta does not raise or bypass the full-reply bound.

## 9. Not a page filter, not a partial compile

A view that requests or paints only visible pages is a DIFFERENT, incomplete
view. It must never be labelled a complete compile, must never be used as a
base, and must never authorize source actions (navigation, edits, exports)
outside the pages it validated. This proposal is the opposite: a delta is
accepted only when the reconstructed list is verified equal (digests, then the
full validator, and in the producer gate byte-identical) to an unchanged fresh
full compile, so the reconstructed list carries exactly the authority of a full
`display_list` — and no more: the existing consumer rule stands that a v2
frame never authorizes source actions beyond the v2 pane's own navigation gate
(buffer-hash equality plus the stale refusal). Candidate flags on the helper
route (`untrusted:true`, `source_actions_enabled:false`) stay as they are.

## 10. Acceptance plan (three separate gates; none claims the others)

### 10.1 Producer gate (`crates/render-pipeline`, cargo tests, no wire activation until reviewed)

- P1 negotiation: not requested → never emitted; `-delta` without
  `display-list-v2` → never accepted; first request of a process → full line,
  echo without `-delta`; second request → delta, echo with both; after a
  `failed` / declined / not-requested reply → snapshot cleared → full again.
  Golden-byte check: for requests that do not list `-delta`, `compile_result`
  and `display_list` bytes are unchanged (existing `golden_v1.rs` fixtures +
  `v2_and_math.rs`).
- P2 byte identity (the Commander's invariant, `tests/incremental.rs` shape):
  for each edit `i` of the existing 200-edit script over the 27-page document
  (and the ignored 30 × 107 run): fresh full line `F_i` (no cache, no delta);
  delta-mode worker with the caches → sibling `S_i` (delta or full); reference
  `apply_delta` in the crate reconstructs `R_i`; assert `write(R_i.to_json(id))
  == F_i` bytes, every `page_digests` entry equals `page_digest` of `R_i`'s
  page, `list_digest` matches, and `compile_result` bytes equal the non-delta
  run's. Record per edit: pages, changed-page count, delta bytes vs full bytes
  (evidence, not a claim).
- P3 refusal/wrong base: reference consumer holds base N−1 while the producer
  emits against N → `delta_base_mismatch`; a tampered page digest → mismatch;
  removed pages after the paragraph-deletion edits → exact `removed_pages`;
  a document-set change → full; a paste spanning pages → changed pages or full
  by policy.
- P4 size: a delta over the line limit → full; a full over the limit → the
  existing decline bytes unchanged; policy factor honoured.
- P5 vectors: Rust `dl2-canon-1` reproduces Appendix B's digests from the
  Appendix B JSON.

### 10.2 Consumer gate (`apps/mac`, Swift tests; fake worker script + real `flashtex-render`)

- C1 `DisplayListDelta.apply(base:delta:)` on the Appendix B vectors → the
  reconstructed `RenderingV2.DisplayList` is `Equatable`-equal to
  `RenderingV2.decode(fresh full)`; digests reproduce the vectors.
- C2 every §7 refusal with a fake worker (unsolicited type, wrong base, bad
  digest, bad relocation, removed-page mismatch, unknown scheme); previous
  frame stays; base dropped; next request omits `-delta`; the following full
  re-establishes the base.
- C3 chain under pipelining: sibling N applied but not painted, N+1 applied and
  painted; base cleared on decline/failed/restart/session change/pane hidden.
- C4 real producer: `flashtex-render` in delta mode over an edit script; every
  reconstructed frame passes the unchanged validator, resolves fonts by content
  hash and reaches `V2Parity` with 0 differing pixels at 1 and 2 px/pt (the
  existing parity gate) — parity is measured, never inferred from digests.
- C5 helper route: blocked until the runtime accepts sibling
  `type: "display_list_delta"` (`crates/document-runtime/src/display_candidate.rs`,
  `raw_display.rs` currently require `display_list`; FT-049 owner) and the
  helper forwards it inside the same `display_candidate` update; the
  `DisplayCandidateGate` identity rules apply unchanged. Until then the
  consumer gate runs on the direct route only, and direct-route results are
  not evidence for the helper route.

### 10.3 Transport gate (measured separately; never claimed from the schema)

- Bytes per edit: delta line vs full line over the 200-edit script (from P2).
- Producer cost: in-process stage timings (`examples/stages.rs`) with the
  digest/classification pass added; worker CPU per request; snapshot RSS.
- Consumer cost: decode+apply+verify of a delta vs decode+validate of a full
  frame, off-main, on the same frames.
- Wall: keystroke→paint through the v2 pane (`TypingBench`) with and without
  `-delta`, load-aware (`uptime` recorded; skipped above 1-min load 20).
- Report as ranges with load; the 33.6 ms / 4.3 MB baseline in
  `oracle-evidence.md` is the comparison point.

## 11. What this proposal does not do

No change to `compile_result`; no change to the full `display_list` bytes; no
change to the decline fallback; no new request field; no change to the
runtime-v1 stream framing; no helper/runtime changes (C5 names the owner); no
size or latency claims. Adoption order after co-signature: producer behind a
build flag with P1–P5 green → Commander records the contract → runtime/helper
change → consumer gate → transport measurements → then, and only then, a
decision to request `-delta` by default in the v2 pane.

## Appendix A — `dl2-canon-1` reference and example generator (Python, stdlib only)

Running `python3 dl2_delta_example.py` prints the digests below;
`… base|delta|fresh` prints the envelopes. This is the normative description of
the digest scheme and of the consumer's reconstruction algorithm; the Rust and
Swift implementations must agree with it on the vectors.

```python
#!/usr/bin/env python3
import hashlib, json, struct, sys

# ---- canonical encoding (dl2-canon-1) -------------------------------------
def i64(n):  return struct.pack('<q', int(n))
def f64(x):  return struct.pack('<d', float(x))
def s(t):
    b = t.encode('utf-8'); return i64(len(b)) + b
def ranges(rs):
    out = i64(len(rs))
    for r in rs: out += s(r['path']) + i64(r['start_byte']) + i64(r['end_byte'])
    return out
def provenance(o):
    if 'sources' in o: return b'\x10' + ranges(o['sources'])
    return b'\x11' + s(o['synthetic_reason'])
def paint(p): return f64(p['r']) + f64(p['g']) + f64(p['b']) + f64(p['a'])
def item(it):
    if it['kind'] == 'glyph_run':
        out = b'\x01' + s(it['font_id']) + i64(it['font_size']) + s(it['text'])
        out += i64(len(it['glyphs']))
        for g in it['glyphs']:
            out += i64(g['gid']) + i64(g['origin_x']) + i64(g['baseline_y']) + i64(g['advance_x']) + i64(g['advance_y']) + i64(g['cluster'])
        out += i64(len(it['clusters']))
        for c in it['clusters']:
            out += i64(c['text_start_byte']) + i64(c['text_end_byte'])
            out += i64(len(c['hit_rects']))
            for r in c['hit_rects']: out += i64(r['x']) + i64(r['top']) + i64(r['width']) + i64(r['height'])
            out += i64(len(c['carets']))
            for k in c['carets']: out += i64(k['text_byte']) + i64(k['x']) + i64(k['top']) + i64(k['height'])
            out += provenance(c)
        return out + paint(it['paint'])
    if it['kind'] == 'rule':
        return b'\x02' + i64(it['x']) + i64(it['top']) + i64(it['width']) + i64(it['height']) + paint(it['paint']) + provenance(it)
    raise ValueError(it['kind'])
def page_digest(p):
    h = hashlib.sha256(b'flashtex:dl2:page:1\0')
    h.update(i64(p['number']) + i64(p['width']) + i64(p['height']) + i64(len(p['items'])))
    for it in p['items']: h.update(item(it))
    return h.hexdigest()
def header_digest(pl):
    h = hashlib.sha256(b'flashtex:dl2:header:1\0')
    for k in ('render_format', 'coordinate_unit', 'color_space', 'text_extraction', 'project_id'): h.update(s(pl[k]))
    h.update(i64(pl['revision']))
    h.update(i64(len(pl['required_features'])))
    for f in pl['required_features']: h.update(s(f))
    h.update(i64(len(pl['documents'])))
    for d in pl['documents']: h.update(s(d['path']) + i64(d['revision']) + s(d['sha256']) + i64(d['byte_length']))
    h.update(i64(len(pl['fonts'])))
    for f in pl['fonts']:
        h.update(s(f['font_id']) + s(f['sha256']) + i64(f['byte_length']) + s(f['format']) + i64(f['face_index']) + i64(f['units_per_em']) + i64(f['glyph_count']) + s(f['postscript_name']))
    h.update(i64(len(pl['diagnostics'])))
    for d in pl['diagnostics']: h.update(s(d['code']) + s(d['message']) + s(d['severity']) + ranges(d['sources']))
    return h.hexdigest()
def list_digest(pl, page_digests):
    h = hashlib.sha256(b'flashtex:dl2:list:1\0')
    h.update(bytes.fromhex(header_digest(pl)) + i64(len(page_digests)))
    for d in page_digests: h.update(bytes.fromhex(d))
    return h.hexdigest()

# ---- relocation + reconstruction (consumer algorithm) ---------------------
def relocate_range(r, reloc):
    a, b, d = reloc['edit_start'], reloc['edit_end'], reloc['delta']
    if r['end_byte'] <= a: return dict(r)
    if r['start_byte'] >= b: return dict(r, start_byte=r['start_byte'] + d, end_byte=r['end_byte'] + d)
    raise ValueError(f"span {r} intersects edited region [{a},{b}) of {reloc['path']}")
def relocate_prov(o, by_path):
    if 'sources' not in o: return o
    return dict(o, sources=[relocate_range(r, by_path[r['path']]) if r['path'] in by_path else dict(r) for r in o['sources']])
def relocate_page(p, relocs):
    by_path = {r['path']: r for r in relocs}
    items = []
    for it in p['items']:
        it2 = dict(it)
        if it['kind'] == 'glyph_run': it2['clusters'] = [relocate_prov(c, by_path) for c in it['clusters']]
        else: it2 = relocate_prov(it2, by_path)
        items.append(it2)
    return dict(p, items=items)
def apply_delta(base_env, delta_env):
    base, d = base_env['payload'], delta_env['payload']
    assert d['base']['request_id'] == base_env['id'], 'delta_base_mismatch'
    assert (d['base']['project_id'], d['base']['revision'], d['base']['page_count']) == (base['project_id'], base['revision'], len(base['pages'])), 'delta_base_mismatch'
    base_pd = [page_digest(p) for p in base['pages']]
    assert d['base']['list_digest'] == list_digest(base, base_pd), 'delta_base_mismatch'
    n = d['page_count']; assert len(d['page_digests']) == n, 'delta_page_count'
    changed = {p['number']: p for p in d['changed_pages']}
    assert sorted(changed) == [p['number'] for p in d['changed_pages']], 'delta_pages_unordered'
    assert d['removed_pages'] == list(range(n + 1, len(base['pages']) + 1)), 'delta_removed_pages'
    pages = []
    for num in range(1, n + 1):
        if num in changed: p = changed[num]
        else:
            assert num <= len(base['pages']), 'delta_page_count'
            p = relocate_page(base['pages'][num - 1], d['relocations'])
        assert page_digest(p) == d['page_digests'][num - 1], f'delta_digest_mismatch page {num}'
        pages.append(p)
    full = {k: d[k] for k in ('render_format', 'coordinate_unit', 'color_space', 'text_extraction', 'project_id', 'revision', 'required_features', 'documents', 'fonts')}
    full['pages'] = pages; full['diagnostics'] = d['diagnostics']
    assert list_digest(full, d['page_digests']) == d['list_digest'], 'delta_list_digest_mismatch'
    return {'protocol_version': 2, 'id': delta_env['id'], 'type': 'display_list', 'payload': full}

# ---- worked example ---------------------------------------------------------
T = 1 << 20
FONT = 'c1f0e5d6a7b8c9d0e1f2a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f8'  # illustrative
def rect(x, w): return {'x': x, 'top': 78643200, 'width': w, 'height': 12582912}
def caret(tb, x): return {'text_byte': tb, 'x': x, 'top': 78643200, 'height': 12582912}
def run(text, glyphs, src0):
    """glyphs: [(gid, advance)] laid out from x=72bp; one cluster per glyph; one source byte each from src0."""
    x = 72 * T; gs, cs = [], []
    for i, (gid, adv) in enumerate(glyphs):
        gs.append({'gid': gid, 'origin_x': x, 'baseline_y': 84 * T, 'advance_x': adv, 'advance_y': 0, 'cluster': i})
        carets = [caret(i, x)] + ([caret(i + 1, x + adv)] if i == len(glyphs) - 1 else [])
        cs.append({'text_start_byte': i, 'text_end_byte': i + 1, 'hit_rects': [rect(x, adv)], 'carets': carets,
                   'sources': [{'path': 'main.tex', 'start_byte': src0 + i, 'end_byte': src0 + i + 1}]})
        x += adv
    return {'kind': 'glyph_run', 'font_id': FONT, 'font_size': 12 * T, 'text': text, 'glyphs': gs, 'clusters': cs,
            'paint': {'r': 0, 'g': 0, 'b': 0, 'a': 1}}
def page(n, items): return {'number': n, 'width': 612 * T, 'height': 792 * T, 'items': items}
def header(rev, text):
    return {'render_format': 'display-list-v2', 'coordinate_unit': 'bp_2pow20', 'color_space': 'srgb', 'text_extraction': 'cluster-actualtext',
            'project_id': 'demo', 'revision': rev, 'required_features': ['glyph_run', 'rgba-srgb', 'cluster-actualtext'],
            'documents': [{'path': 'main.tex', 'revision': rev, 'sha256': hashlib.sha256(text.encode()).hexdigest(), 'byte_length': len(text.encode())}],
            'fonts': [{'font_id': FONT, 'sha256': FONT, 'byte_length': 154012, 'format': 'opentype-cff', 'face_index': 0, 'units_per_em': 1000, 'glyph_count': 821, 'postscript_name': 'LMRoman12-Regular'}],
            'diagnostics': []}

T0 = "\\begin{document}\nHi\n\\newpage\nBye\n\\end{document}\n"
T1 = "\\begin{document}\nHio\n\\newpage\nBye\n\\end{document}\n"
assert T0.index('Hi') == 17 and T0.index('Bye') == 29 and T1.index('Bye') == 30
H, I, O = (43, 9437184), (76, 3495390), (82, 6291456)
B, Y, E = (37, 8912896), (92, 6640435), (72, 5767168)

base = header(7, T0); base['pages'] = [page(1, [run('Hi', [H, I], 17)]), page(2, [run('Bye', [B, Y, E], 29)])]
base_env = {'protocol_version': 2, 'id': 'mac-42', 'type': 'display_list', 'payload': base}
fresh = header(8, T1); fresh['pages'] = [page(1, [run('Hio', [H, I, O], 17)]), page(2, [run('Bye', [B, Y, E], 30)])]
fresh_env = {'protocol_version': 2, 'id': 'mac-43', 'type': 'display_list', 'payload': fresh}

base_pd = [page_digest(p) for p in base['pages']]
fresh_pd = [page_digest(p) for p in fresh['pages']]
delta = {k: fresh[k] for k in ('render_format', 'coordinate_unit', 'color_space', 'text_extraction', 'project_id', 'revision', 'required_features')}
delta['digest_scheme'] = 'dl2-canon-1'
delta['base'] = {'request_id': 'mac-42', 'project_id': 'demo', 'revision': 7, 'page_count': 2, 'list_digest': list_digest(base, base_pd)}
delta['documents'] = fresh['documents']; delta['fonts'] = fresh['fonts']; delta['diagnostics'] = fresh['diagnostics']
delta['relocations'] = [{'path': 'main.tex', 'edit_start': 19, 'edit_end': 19, 'delta': 1}]
delta['page_count'] = 2
delta['page_digests'] = fresh_pd
delta['changed_pages'] = [fresh['pages'][0]]
delta['removed_pages'] = []
delta['list_digest'] = list_digest(fresh, fresh_pd)
delta_env = {'protocol_version': 2, 'id': 'mac-43', 'type': 'display_list_delta', 'payload': delta}

recon = apply_delta(base_env, delta_env)
assert recon == fresh_env, 'reconstruction differs'
assert json.dumps(recon, sort_keys=True, separators=(',', ':')) == json.dumps(fresh_env, sort_keys=True, separators=(',', ':'))

if __name__ == '__main__':
    what = sys.argv[1] if len(sys.argv) > 1 else 'digests'
    if what == 'base': print(json.dumps(base_env, indent=1))
    elif what == 'delta': print(json.dumps(delta_env, indent=1))
    elif what == 'fresh': print(json.dumps(fresh_env, indent=1))
    else:
        print('T0 sha256', hashlib.sha256(T0.encode()).hexdigest(), len(T0.encode()))
        print('T1 sha256', hashlib.sha256(T1.encode()).hexdigest(), len(T1.encode()))
        print('base page digests', base_pd)
        print('base header', header_digest(base)); print('base list', delta['base']['list_digest'])
        print('fresh page digests', fresh_pd)
        print('fresh header', header_digest(fresh)); print('fresh list', delta['list_digest'])
        print('base line bytes', len(json.dumps(base_env, separators=(',', ':'))), 'delta line bytes', len(json.dumps(delta_env, separators=(',', ':'))), 'fresh line bytes', len(json.dumps(fresh_env, separators=(',', ':'))))
        print('reconstruction == fresh: True')
```

Output on 2026-09-12 (Python 3, mac-m1max-a):

```
T0 sha256 580f3c9f730136acf7979d99d8702b41220588ad2cf482a773b7a2b277db29f2 48
T1 sha256 1e2dc5fa4cfdf13ffde12d7e17a142ad0b7389be3aabe5ce54535f221d4b1eee 49
base page digests ['154625d5e6e3596b9af130702f332a98141e08984216fbe68947183ac0f9aa77', '1748b1d3e714ec6222ee2fdf8d06f86943e0bab41806c3a0715435ef29a7a061']
base header 1ce217312d6d30d9809528a6d656025f4711b1c037cd7b75b9c46a74ff600618
base list 68db4fe3ae528414058efecd7d5870c689d598f415a4c6045a86173d7514eec1
fresh page digests ['2a2f15ca94dd927ace6c633f69cf5db765ccdf8622df40ca107b93d850edee94', '4d71e83b0f5b28c5db04e4232d75fde0deb61912acb6e4f5b37c0969ecf7518e']
fresh header 7411c7fde8668f1bf2e8cf39a762d2e4b7eac6c4db3967fd8cfb4f2096503ab8
fresh list 6e93b60654755c252c79580be04e5d964d7b9fc4aee167d6124afc766a6c4e1e
base line bytes 3145 delta line bytes 2676 fresh line bytes 3498
reconstruction == fresh: True
```

(The byte counts are Python's compact `json.dumps` of the example, not the
producer writer's; the example is too small for the delta to be meaningfully
smaller — it exists for the digests and the algorithm, not for size.)

## Appendix B — worked example: base full list → one-word edit → delta → reconstruction

Geometry is illustrative (a 12 pt run at the 1 in margin, baseline at 84 bp),
not pipeline output; the font id/sha256 `c1f0e5d6…` is the 64-hex constant
`FONT` of Appendix A, abbreviated here for width only (digests were computed
over the full value). Source: `T0` = `\begin{document}\nHi\n\newpage\nBye\n\end{document}\n`
(48 bytes; `Hi` at bytes 17–19, `Bye` at 29–32). The edit inserts `o` at byte
19 (`T1`, 49 bytes; `Bye` now at 30–33): relocation `{edit_start: 19,
edit_end: 19, delta: 1}`.

### B.1 Base: the full `display_list` sibling of request `mac-42` (revision 7)

```json
{"protocol_version":2,"id":"mac-42","type":"display_list","payload":{
  "render_format":"display-list-v2",
  "coordinate_unit":"bp_2pow20",
  "color_space":"srgb",
  "text_extraction":"cluster-actualtext",
  "project_id":"demo",
  "revision":7,
  "required_features":["glyph_run","rgba-srgb","cluster-actualtext"],
  "documents":[{"path":"main.tex","revision":7,"sha256":"580f3c9f730136acf7979d99d8702b41220588ad2cf482a773b7a2b277db29f2","byte_length":48}],
  "fonts":[{"font_id":"c1f0e5d6…","sha256":"c1f0e5d6…","byte_length":154012,"format":"opentype-cff","face_index":0,"units_per_em":1000,"glyph_count":821,"postscript_name":"LMRoman12-Regular"}],
  "pages":[
    {"number":1,"width":641728512,"height":830472192,"items":[
     {"kind":"glyph_run","font_id":"c1f0e5d6…","font_size":12582912,"text":"Hi",
      "glyphs":[
       {"gid":43,"origin_x":75497472,"baseline_y":88080384,"advance_x":9437184,"advance_y":0,"cluster":0},
       {"gid":76,"origin_x":84934656,"baseline_y":88080384,"advance_x":3495390,"advance_y":0,"cluster":1}
      ],
      "clusters":[
       {"text_start_byte":0,"text_end_byte":1,"hit_rects":[{"x":75497472,"top":78643200,"width":9437184,"height":12582912}],"carets":[{"text_byte":0,"x":75497472,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":17,"end_byte":18}]},
       {"text_start_byte":1,"text_end_byte":2,"hit_rects":[{"x":84934656,"top":78643200,"width":3495390,"height":12582912}],"carets":[{"text_byte":1,"x":84934656,"top":78643200,"height":12582912},{"text_byte":2,"x":88430046,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":18,"end_byte":19}]}
      ],
      "paint":{"r":0,"g":0,"b":0,"a":1}}
    ]},
    {"number":2,"width":641728512,"height":830472192,"items":[
     {"kind":"glyph_run","font_id":"c1f0e5d6…","font_size":12582912,"text":"Bye",
      "glyphs":[
       {"gid":37,"origin_x":75497472,"baseline_y":88080384,"advance_x":8912896,"advance_y":0,"cluster":0},
       {"gid":92,"origin_x":84410368,"baseline_y":88080384,"advance_x":6640435,"advance_y":0,"cluster":1},
       {"gid":72,"origin_x":91050803,"baseline_y":88080384,"advance_x":5767168,"advance_y":0,"cluster":2}
      ],
      "clusters":[
       {"text_start_byte":0,"text_end_byte":1,"hit_rects":[{"x":75497472,"top":78643200,"width":8912896,"height":12582912}],"carets":[{"text_byte":0,"x":75497472,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":29,"end_byte":30}]},
       {"text_start_byte":1,"text_end_byte":2,"hit_rects":[{"x":84410368,"top":78643200,"width":6640435,"height":12582912}],"carets":[{"text_byte":1,"x":84410368,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":30,"end_byte":31}]},
       {"text_start_byte":2,"text_end_byte":3,"hit_rects":[{"x":91050803,"top":78643200,"width":5767168,"height":12582912}],"carets":[{"text_byte":2,"x":91050803,"top":78643200,"height":12582912},{"text_byte":3,"x":96817971,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":31,"end_byte":32}]}
      ],
      "paint":{"r":0,"g":0,"b":0,"a":1}}
    ]}
  ],
  "diagnostics":[]
}}
```

Base snapshot (both sides): `page_digests = [154625d5…aa77, 1748b1d3…a061]`,
`list_digest = 68db4fe3ae528414058efecd7d5870c689d598f415a4c6045a86173d7514eec1`.

### B.2 The edit and the request

`Hi` → `Hio`; request `mac-43`, revision 8, `layout_capabilities:
["rules-v1","font-hints-v1","display-list-v2","display-list-v2-delta"]`.
The producer's snapshot is `mac-42`; the document set is the same; the diff
gives `a = 19, b = 19, d = +1`. Page 1 retypesets (changed); page 2's only
block is a cache hit relocated by +1 and `relocate(base page 2) == new page 2`
(unchanged). The `compile_result` for `mac-43` echoes both capabilities.

### B.3 The delta sibling of `mac-43`

```json
{"protocol_version":2,"id":"mac-43","type":"display_list_delta","payload":{
  "render_format":"display-list-v2",
  "coordinate_unit":"bp_2pow20",
  "color_space":"srgb",
  "text_extraction":"cluster-actualtext",
  "project_id":"demo",
  "revision":8,
  "required_features":["glyph_run","rgba-srgb","cluster-actualtext"],
  "digest_scheme":"dl2-canon-1",
  "base":{"request_id":"mac-42","project_id":"demo","revision":7,"page_count":2,"list_digest":"68db4fe3ae528414058efecd7d5870c689d598f415a4c6045a86173d7514eec1"},
  "documents":[{"path":"main.tex","revision":8,"sha256":"1e2dc5fa4cfdf13ffde12d7e17a142ad0b7389be3aabe5ce54535f221d4b1eee","byte_length":49}],
  "fonts":[{"font_id":"c1f0e5d6…","sha256":"c1f0e5d6…","byte_length":154012,"format":"opentype-cff","face_index":0,"units_per_em":1000,"glyph_count":821,"postscript_name":"LMRoman12-Regular"}],
  "diagnostics":[],
  "relocations":[{"path":"main.tex","edit_start":19,"edit_end":19,"delta":1}],
  "page_count":2,
  "page_digests":[
    "2a2f15ca94dd927ace6c633f69cf5db765ccdf8622df40ca107b93d850edee94",
    "4d71e83b0f5b28c5db04e4232d75fde0deb61912acb6e4f5b37c0969ecf7518e"
  ],
  "changed_pages":[
    {"number":1,"width":641728512,"height":830472192,"items":[
     {"kind":"glyph_run","font_id":"c1f0e5d6…","font_size":12582912,"text":"Hio",
      "glyphs":[
       {"gid":43,"origin_x":75497472,"baseline_y":88080384,"advance_x":9437184,"advance_y":0,"cluster":0},
       {"gid":76,"origin_x":84934656,"baseline_y":88080384,"advance_x":3495390,"advance_y":0,"cluster":1},
       {"gid":82,"origin_x":88430046,"baseline_y":88080384,"advance_x":6291456,"advance_y":0,"cluster":2}
      ],
      "clusters":[
       {"text_start_byte":0,"text_end_byte":1,"hit_rects":[{"x":75497472,"top":78643200,"width":9437184,"height":12582912}],"carets":[{"text_byte":0,"x":75497472,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":17,"end_byte":18}]},
       {"text_start_byte":1,"text_end_byte":2,"hit_rects":[{"x":84934656,"top":78643200,"width":3495390,"height":12582912}],"carets":[{"text_byte":1,"x":84934656,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":18,"end_byte":19}]},
       {"text_start_byte":2,"text_end_byte":3,"hit_rects":[{"x":88430046,"top":78643200,"width":6291456,"height":12582912}],"carets":[{"text_byte":2,"x":88430046,"top":78643200,"height":12582912},{"text_byte":3,"x":94721502,"top":78643200,"height":12582912}],"sources":[{"path":"main.tex","start_byte":19,"end_byte":20}]}
      ],
      "paint":{"r":0,"g":0,"b":0,"a":1}}
    ]}
  ],
  "removed_pages":[],
  "list_digest":"6e93b60654755c252c79580be04e5d964d7b9fc4aee167d6124afc766a6c4e1e"
}}
```

### B.4 Reconstruction on the consumer

1. `base` equals the held snapshot: `mac-42` / `demo` / 7 / 2 pages /
   `68db4fe3…eec1`. `digest_scheme` known. `page_count` 2, `changed_pages`
   = {1}, `removed_pages` = [] = `[3..2]`. OK.
2. Page 1 := the changed page as sent; `page_digest` = `2a2f15ca…ee94` = `page_digests[0]`. OK.
3. Page 2 := relocate(base page 2, `{19, 19, +1}`): every span has
   `start_byte ≥ 19`, so `29–30 → 30–31`, `30–31 → 31–32`, `31–32 → 32–33`;
   glyphs, hit rects, carets, paint untouched. `page_digest` =
   `4d71e83b…518e` = `page_digests[1]`. OK.
4. Header from the delta (`documents` with the new sha256/49 bytes, `fonts`,
   `diagnostics`, features, `demo`/8) + pages → `list_digest` =
   `6e93b606…4e1e` = the delta's. OK.
5. The full validator runs on the result; it is then the new base
   (`mac-43` / `demo` / 8 / 2 / `6e93b606…4e1e`).

The reconstructed envelope (with `type` `display_list` and id `mac-43`) equals
the fresh full list for `T1`, field for field (Appendix A asserts it); in the
producer gate the same equality is asserted on the producer's own JSON bytes.
