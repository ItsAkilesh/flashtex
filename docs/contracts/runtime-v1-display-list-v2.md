# Runtime-v1 negotiated capability `display-list-v2`: live rendering-v2 frames

Status: **proposed by the Mac preview owner (mac-preview-v2) and ACKed/implemented by the
producer lane (mac-render-pipeline) at `agent/mac-render-pipeline/unified` 4888a67
("display-list-v2 sibling line"); consumed by the Mac shell on `agent/mac-preview-v2/live`.
Additive to runtime-v1; becomes authoritative when the Commander records it on main.**
Producer-side details that go beyond this text (4888a67): a declined request also sets
`status: recovered` and the warning diagnostic's code is `display_list_declined`; the
producer's line limit is 16 MiB (`JSONLines.maxLineBytes`). Base contracts stay
unchanged: [runtime-v1](runtime-v1.md), [layout capabilities](runtime-v1-layout-capabilities.md),
[rendering-v2 proposal](rendering-v2-proposal.md), `protocol/rendering-v2.schema.json`.
Owner: mac-preview-v2 (consumer side). Date: 2026-09-12. Supersedes nothing.

## Why a layout capability, not a new `render_format` request field

`payload.layout_capabilities` is runtime-v1's only authoritative per-request negotiation:
duplicate-free strings, the producer echoes the accepted subset, unknown names are never
accepted, nothing unrequested is ever emitted, and acceptance is bound to the applied result
(stale replies never change the renderer mode). `display-list-v2` fits that mechanism
exactly and needs no new top-level field or new validation on either side:

- Existing producers (`flashtex-compiler` on main, `flashtex-render`) already ignore the
  unknown string, so an old producer answers a new consumer with plain runtime-v1.
- Existing consumers never send it, so an old consumer is never sent a v2 line.
- The Mac shell's `LayoutNegotiation` (request/accept binding, violation checks, capability
  switch mid-flight) applies unchanged. A top-level `render_format` would duplicate that
  machinery and would not compose with `rules-v1`/`font-hints-v1` per request.

The rendering-v2 proposal's separate `render_capabilities`/`render_format_selected` handshake
(crates/rendering-core `wire.rs`) remains the design for a dedicated v2 stream; this document
carries the *same* `display_list` envelope over the existing v1 stream, per request.

## Request

```json
{"protocol_version":1,"id":"mac-42","type":"compile","payload":{
  "project_id":"demo","revision":7,"entry_path":"main.tex",
  "documents":[{"path":"main.tex","text":"\\documentclass{article}\n\\begin{document}\nHi\n\\end{document}\n"}],
  "layout_capabilities":["rules-v1","font-hints-v1","display-list-v2"]}}
```

The consumer requests `display-list-v2` only while it can consume it (the Mac shell: while
the v2 pane is visible). Everything else in the request is base runtime-v1.

## Reply: the unchanged `compile_result` line, then ONE sibling `display_list` line

When the producer accepts `display-list-v2` it echoes it in `payload.layout_capabilities`
of the `compile_result` exactly as for the other capabilities, and — only for `status`
`ok` or `recovered` — writes one additional JSON line **immediately after** that
`compile_result` line and **before** any reply to a later request:

```json
{"protocol_version":1,"id":"mac-42","type":"compile_result","payload":{
  "project_id":"demo","revision":7,"status":"ok","pages":[…runtime-v1 pages…],
  "diagnostics":[],"pdf_path":null,
  "layout_capabilities":["rules-v1","font-hints-v1","display-list-v2"]}}
{"protocol_version":2,"id":"mac-42","type":"display_list","payload":{
  "render_format":"display-list-v2","coordinate_unit":"bp_2pow20","color_space":"srgb",
  "text_extraction":"cluster-actualtext","project_id":"demo","revision":7,
  "required_features":["glyph_run","rgba-srgb","cluster-actualtext"],
  "documents":[{"path":"main.tex","revision":7,"sha256":"<sha256 of the request text>","byte_length":58}],
  "fonts":[…],"pages":[…],"diagnostics":[]}}
```

The second line is byte-for-byte what `flashtex-render --v2 out.json` writes today for that
request (the `display_list` envelope of `protocol/rendering-v2.schema.json`, with the
documented `opentype-cff` / SHA-256(bytes‖face0) deviations), so `crates/rendering-core`
`validate_display` and the Mac `RenderingV2.decode` consume it unchanged. Rules:

1. `id` equals the compile request id. `payload.project_id` and `payload.revision` equal the
   `compile_result`'s. `documents[].revision` equals the request revision and
   `documents[].sha256`/`byte_length` are those of the request's document text.
2. Exactly one `display_list` line per accepted, non-`failed` `compile_result`; none for
   `status: failed` and none when the capability was not accepted for that request.
3. Ordering on the stream: `compile_result(id)` then `display_list(id)`, contiguous.
4. Line limits are per line: the v1 result keeps its own budget and paints first. A producer
   whose envelope would exceed its line limit (flashtex-compiler protocol: 8 MiB) **declines
   the capability for that request**: it omits `display-list-v2` from the echoed list, sends
   no `display_list` line, and adds a `warning` diagnostic whose message starts with
   `display-list-v2 declined:`. Declining per request is already legal under the layout
   capabilities contract (acceptance is a subset, per request).
5. The producer never sends a `display_list` line for a request that did not ask, and never
   sends any other `protocol_version: 2` message on the v1 stream.

## Consumer action (Mac shell, this lane)

- Only a consumer that requested `display-list-v2` accepts `protocol_version: 2` +
  `type: display_list` lines on the v1 stream; any other v2 line, or a v2 line while the
  capability is not requested, is a protocol violation (as today).
- A `display_list` line is applied only if its `id` is the id of the *currently applied*
  `compile_result` and that result's accepted capabilities include `display-list-v2`, and its
  `project_id`/`revision` match. Otherwise it is dropped and logged as stale/unsolicited —
  the same suppression as the compile results themselves; a stale v2 frame never replaces a
  newer one. Decoding, validation, font resolution and page preparation run off the main
  thread (`V2Loader`), with the v2 pane keeping the previous verified frame, labelled stale,
  until the new one is verified.
- The v1 pages of the same `compile_result` are the fallback: they are applied first and
  stay the product preview; the v2 pane is experimental and opt-in.
- The preview-controller (helper) route is not covered by this revision: the helper would
  have to forward the sibling line inside its `update` framing; that is a follow-up for the
  preview-controller owner.

## Fixture and gates

- Producer gate: `cargo test` covering (a) not requested → no line; (b) requested + `ok` →
  echoed + one line with matching id/project/revision/document digest; (c) `failed` → no
  line; (d) oversize → declined with the warning.
- Consumer gate (this lane, apps/mac): fake worker emitting the sibling line; stale id
  dropped; unsolicited line rejected; live frame → parity gate (`V2Parity`) 0 differing
  pixels at 1 and 2 px/pt; keystroke→paint through the v2 pane via `TypingBench`.
- Evidence for both lives under `docs/evidence/` with exact binary SHAs: consumer side
  `docs/evidence/mac-preview-v2-live-2026-09-12.md` (keystroke→paint through the v2 pane with
  4888a67, 0-pixel export parity on six consecutive live frames).
