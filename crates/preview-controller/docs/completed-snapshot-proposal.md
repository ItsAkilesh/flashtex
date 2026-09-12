# Completed snapshots during continuous typing — integration proposal

Status: proposed, not implemented or enabled. Existing strict current-preview behavior remains the default. Evidence: `benchmarks/helper-burst/50k.json` has20 durable edits at30ms intervals but only1 preview,11 stale completions and8 superseded requests. The final exact preview arrived91ms after the final send. Publishing already-completed older snapshots could show progress during this burst; it cannot establish instantaneous current-document output or solve compiler throughput.

## Why this needs a runtime change

`document-runtime::Session::poll` validates every complete response against its original request, then drops the result when a newer revision is pending, emitting only `Event::Stale { id, revision }`. The controller cannot recover those bytes from that event. `Controller::submitted` also retains only the latest request binding. A controller-only switch that relabels an old result as current would be incorrect.

Keep `Event::Preview`, `Controller::is_current_preview`, and default helper `kind:preview` semantics unchanged. Add a separate runtime opt-in side channel for fully validated, uncancelled stale completions. A proposed `take_completed_snapshot()` moves out at most one retained result; it never reparses, clones or recompiles it. Disabling the feature, closing the project, cancelling its request, restarting the compiler or changing session identity clears retained snapshots. A higher compile revision replaces a lower retained snapshot. No partially framed or validation-failed response can enter this slot.

The result needs immutable origin metadata: project ID, session/incarnation, request ID, compile generation, and an opaque bounded source-binding token captured on submission. The controller must pass that token with the exact original `VersionSnapshot` rather than derive it from whichever document is current at completion time. Runtime should return the token unchanged; it must not depend on editor-specific revision types. Existing callers omit the optional token. Use an owned byte/string token with an explicit small limit, not arbitrary JSON or captured document text.

For a controller integration, retain only bounded metadata for potentially deliverable submissions and remove it on supersession/cancellation/completion. Do not introduce an unbounded request-ID map. Size this against actual runtime admission/event bounds; reject admission before a durable mutation if additional metadata capacity is required, or discard historical display eligibility without compromising the successful source save. Never reject a source save merely because an optional historical preview cannot be retained.

## Proposed helper negotiation and event

Native explicitly requests `completed-snapshots-v1`; absence preserves current wire behavior. The helper must report negotiated support, and must not emit the new event before acknowledgement. This is separate from layout capabilities (`rules-v1`, `font-hints-v1`). Old native clients receive no new message kinds.

A historical update would have this shape:

```json
{
  "protocol_version": 1,
  "session_id": "current-helper-incarnation",
  "id": null,
  "type": "update",
  "payload": {
    "kind": "completed_snapshot",
    "request_id": "preview-10",
    "compile_revision": 10,
    "source_versions": {"main.tex": 7},
    "current_compile_revision": 12,
    "is_current": false,
    "source_actions_enabled": false,
    "result": {"...": "the unchanged, fully validated original compile result"}
  }
}
```

The illustrative result placeholder above is not a valid compiler response. The implementation must forward the full original result through the owned-value wrapper path. Source versions must be the originating versions, not the latest ones. Session identity and project identity must be checked independently of numeric generation, which may restart in a new helper.

## Native behavior and authorization boundaries

The app may paint a monotonic sequence of completed snapshots while it visibly indicates that a newer edit is compiling. Recheck current native session/project and latest displayed generation immediately before painting, including after queued UI work. An older completed generation must never overwrite a newer displayed generation. Closing/reopening or switching documents invalidates queued snapshot deliveries.

A historical snapshot grants no source navigation, diagnostic jump, AI edit destination, source-map application or current-document export authority. Disable these actions for historical display; restore them only after the existing exact current-source checks succeed. Do not use an old source span with the current text merely because its byte offset remains in bounds. Export can either wait for the current build or offer an explicitly versioned historical artifact through a separately reviewed flow; no silent substitution.

Keep at most one historical pending frame at each integration boundary. Bounded memory includes runtime retained Value, helper serialization buffer, output queue, and native retained PDF/display-list caches. Existing output limits and stalled-reader termination remain in force. Do not allow a stream of optional historical updates to crowd out durable acknowledgements or final current previews; prioritize those and drop optional snapshots before queue admission when required. This likely requires explicit scheduling support beyond merely appending another FIFO event.

## Gates before enabling the feature

| Gate | Required evidence |
| --- | --- |
|Default compatibility| Existing callers receive identical event kinds, exact compiler bytes and current-source rejection behavior with feature off. |
|Validated origin| Slow A followed by B can deliver A only as historical, bound to A's immutable source metadata; malformed A produces no snapshot. |
|Cancellation/restart| Explicit cancellation, project close, capability reset and session restart prevent late delivery; same numeric generation in a new session cannot authorize an old frame. |
|Monotonic UI| Delayed historical UI callbacks cannot replace a newer displayed/current frame. |
|Source actions| Historical output cannot initiate navigation or an edit; old diagnostic spans remain inert even if valid offsets in current text. |
|Bounded delivery| Flood edits and stall native reads; acknowledgements retain their durable semantics, final current preview is eligible, optional snapshots cannot grow queues without bound. |
|Typing fidelity| Repeat5/50/500KB burst driver with exact final clean equality, durable reopen, actual frame timestamps and number of updates during typing. Report historical lag separately from current-preview latency. |
|Native acceptance| Mac owner runs real rendering/interaction tests and marks older snapshots visibly. Linux helper events alone cannot prove native safety or perceived responsiveness. |

Runtime owner must approve and implement the additive origin-token/retained-snapshot API; controller owner implements bounded metadata and helper negotiation; Mac parent owns visible labeling, UI ordering and source-action gating. Publish interface agreement before parallel edits. Do not enable production behavior until all three owners complete the gates. In parallel, continue compiler dependency/incremental/layout work: this feature complements exact low-latency compilation and does not replace it.
