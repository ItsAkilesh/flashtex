# Source-bound AI explanation context

This Rust crate prepares bounded diagnostic context and validates explanation
proposals. It performs no model calls, credential access, network IO or source
mutation. The default provider intent is Grok; model selection and transport are
owned separately by the native integration.

Capture `CompileBinding::capture(request_id, project_id, compile_revision, sources)`
when submitting the compiler request. Build Context only from the matching reply
and unchanged authoritative Documents. Request identity, project, revision and
all source hashes must match. Caller capture timing matters: making a new binding
from current text after receiving an old reply cannot establish provenance.

The payload separates system instructions, explicit user instructions, compiler
status/recovery information, diagnostics and source snippets. Source content is
untrusted data, not agent instructions. A context ID hashes the binding and exact
payload; provider responses must echo it. At most16diagnostics and8explicit related
files are included, each snippet at most2048UTF-8 bytes. Snippet offsets refer to
the original file. Diagnostic messages are clipped to2048bytes; omitted diagnostic
count is explicit. The user instruction limit is8192bytes; serialized payload and
response limits are64KiB. Exceeding aggregate bounds fails explicitly.

`validate_response(bytes, current_sources)` requires a nonempty explanation and
at most8proposed edits. Each edit must name valid UTF-8 offsets inside a supplied
snippet and exact removed text. Replacements are capped8KiB. Overlapping edits
or edits sharing a boundary are refused. All source identities are rechecked
before returning the proposal. Validation is not approval: show the proposal,
obtain explicit user approval, then use the editor's durable review/ledger path.
Never apply the returned edit objects directly or automatically.

The prompt is not itself a provider-specific HTTP request. Response JSON fields:
`context_id`, `explanation`, `edits:[{location:{path,start_byte,end_byte},removed_text,replacement}]`.
A model can still give an incorrect explanation; validation establishes source
binding and bounded edit shape, not correctness of advice. Live Grok, native UI,
request cancellation and provider error handling remain separate integration work.

Validation: `cargo test --manifest-path crates/assistant-context/Cargo.toml --offline`.
Set `FLASHTEX_TEST_COMPILER` to the original compiler binary and add
`-- --include-ignored` to exercise actual compiler diagnostics as well.

Use `context.payload()` to obtain a read-only payload reference. The bound payload
cannot be mutated through Context after its context ID is computed. A cloned
payload is presentation data, not a new validated context. Each clipped diagnostic
sets `message_truncated:true`, so clients and the model can distinguish incomplete
message text from a complete compiler diagnostic.

`ExplanationFlight` binds one Context to a local deadline (at most120seconds).
It accepts at most one validated response. Cancellation, expiry and validation
failure are terminal; stale or duplicate responses cannot revive the request.
The native caller must cancel the actual provider task separately and still
recheck source when approving any returned proposal. Polling `state()` observes
expiry without an inference call; this library does not run a timer or network job.

`build_selected(..., selected_indices)` selects up to16unique diagnostics by their
indices in the bound compiler result. Each selected context includes its original
`diagnostic_index`; the context hash therefore binds the exact selection. Invalid
or duplicate indices are refused. `build` retains its default first16behavior.

`restrict_edits(destinations,current_sources)` consumes Context and returns a new
context ID bound to up to8explicit UTF-8 destination ranges. Proposals must stay
inside those ranges as well as supplied snippets. A zero-width destination permits
insertion only at that offset; an empty destination list means explanation-only.

The `flashtex-assistant-context` binary accepts one JSON request on stdin (16MiB
maximum) and emits one JSON reply (128KiB maximum) before exiting. Run it off the
native UI thread. Fields: `operation` (`prepare` or `validate`), `binding`, `sources`,
`compiler_result`, `user_instruction`; optional `related_paths`,
`selected_diagnostics`, `destinations`. `validate` additionally requires `response`
and `current_sources`. It reconstructs/checks the same bound context, returns a
`validated_proposal` with `applied:false`, and never writes source. `prepare`
returns `prepared_context`. Errors return `type:error` and nonzero exit status.
Provider transport and actual user approval remain separate native operations.

Source binding rejects more than32MiB of aggregate source before hashing, more
than8MiB in one document, or paths longer than1024bytes. This bounds work when
called directly as a library as well as through the16MiB JSON helper. The limit
is explicit; this crate does not silently omit source identities to make a large
project fit. Malformed/oversized helper requests fail without mutating source;
a subsequent clean process can prepare a fresh context normally.

`ExplanationRegistry` manages up to32 pending requests and256 terminal IDs (caller
chooses lower limits). Supply a fresh session identifier for each registry lifetime;
monotonic request IDs are never recycled within that lifetime, even after eviction.
Route responses with both request ID and immutable context ID. Wrong routing cannot
consume another pending request. Invalid correctly routed responses terminate it.
Cancellation, timeout and successful completion reject all later callbacks.
Terminal retention stores only IDs/states, never source text or completed proposals.
The caller receives a proposal once and must separately obtain approval and perform
ledger validation before editing. There is intentionally no automatic replay/apply.
Call `revoke_stale(project_id, complete_current_snapshot)` on source changes and
cancel the returned provider task IDs; call `sweep` from the native timer to collect
expired IDs. Registry lifecycle actions do not themselves stop network requests.
This is a Rust integration API; the one-request JSON helper remains unchanged.

Run `flashtex-assistant-context --session FRESH_SESSION_ID` for a persistent JSONL
helper (8 pending requests,64 terminal IDs). Each command is
`{"id":"caller-command-id","action":{"operation":"...",...}}`; each reply
contains that ID plus `result` or `error`. Invalid JSON returns a null ID and does
not terminate the session. Oversized16MiB input frames terminate it. Replies are
bounded128KiB and flushed after every command. EOF ends the process.

Actions:
- `submit`: `input` is the existing prepare request, plus `timeout_ms`1..120000.
  Returns `request_id` and immutable context `payload`.
- `receive`: `request_id`, `context_id`, `response`, `current_sources`; returns a
  validated proposal with `applied:false`, never edits documents.
- `cancel`: `request_id`; returns whether a pending request changed.
- `revoke_stale`: `project_id`, complete `current_sources`; returns revoked IDs.
- `status`: `request_id`; returns state or null if unknown/evicted.
- `sweep`: returns expired IDs for cancelling provider tasks.

Commands are processed sequentially while multiple provider requests can be pending
in the native caller. No provider is called here. Use a fresh session ID on restart,
read stdout continuously off the UI thread, and supervise process/IO deadlines in
the native host. The helper uses blocking pipes and does not enforce an idle or
blocked-output deadline itself. Responses are consumed once, including on a lost
pipe reply; never replay an edit automatically after helper restart.

On Unix (macOS/Linux), `SessionClient` supervises the helper using nonblocking
stdin/stdout. `call(action, timeout)` bounds both a full input pipe and a stalled
reply, checks command correlation and reply shape, and kills/reaps the direct
helper on transport/protocol failure. It never retries. stderr goes to the null
device, and retained frames are bounded. Drop also closes pipes and reaps the
helper. No IO reader threads can remain blocked on descendant-held descriptors.
The client does not kill arbitrary descendants; only launch the trusted helper,
which does not spawn any. Run calls on a background worker; this API still blocks
that worker while awaiting a reply. Use a new session ID for every explicit restart.
OS process creation/reaping and JSON serialization are not real-time operations;
the deadline specifically bounds pipe exchange rather than guaranteeing a hard
wall-clock deadline under all operating-system conditions.

`cargo run --release --example client_latency -- PATH_TO_HELPER` measures five
local submit/cancel roundtrips at each source size (5KB,50KB,500KB,1MB). It includes
JSON serialization, pipe exchange and source/context validation, excludes provider
calls, and does not measure editor-to-painted-preview latency. On this shared Linux
host a baseline run measured medians1.06/1.10/12.99/26.22ms. An experimental loop
that slept only when IO made no progress measured2.66/9.40/90.78/149.42ms, with
substantial variation on repeat. The experiment was reverted: these observations
do not establish a speedup or a stable regression cause under concurrent builds.
Use controlled repeated comparisons before changing scheduling behavior.

Optional feature `grok` exposes a Rust-only `grok::GrokClient`. The caller supplies
an authorized key, explicit model ID and timeout; construction reads no environment
or keychain and performs no calls. Only an explicit `request(context,current)`
sends HTTPS to `https://api.x.ai/v1/responses`. Redirects, automatic retries and
inherited proxies are disabled. Requests set `store:false`, nonstreaming structured
JSON output and8192 max output tokens, with256KiB request/512KiB response limits.
No tools or billing fallback are configured. Errors never include provider bodies,
request source, or the credential. Run blocking HTTP off the UI thread.

The returned `ProviderReply` remains untrusted: validate it with a fresh source
snapshot after the request, or pass its bytes to the owning registry's `receive`
method to enforce cancellation/expiry too. No edit is applied. Blocking transport
cannot be interrupted through registry cancellation; the response is rejected and
the configured HTTP timeout bounds the outstanding call. API availability, selected
model access, actual provider schema behavior and live latency remain unverified.
The default JSON helper does not accept credentials or initiate network requests.

Contract references checked September12,2026:
[xAI Responses](https://docs.x.ai/developers/rest-api-reference/inference/responses),
[structured outputs](https://docs.x.ai/developers/model-capabilities/text/structured-outputs).
Local HTTP fixtures use dummy credentials and are not evidence of live Grok success.

Additional local HTTP coverage exercises delayed responses, chunked bodies without
Content-Length, oversized chunked output, and a successful reply arriving after
registry cancellation. These validate timeout/size/lifecycle behavior without
calling xAI; they do not prove server-side cancellation or live model accuracy.

For registry-owned requests, use `registry.lease(id,current)` followed by
`GrokClient::request_lease(lease,current)` on a background worker. A lease shares
the immutable Context with the registry and is issued at most once per request.
It cannot be cloned; cancellation, expiry or registry destruction invalidates
preflight dispatch. The resulting `RoutedReply::receive(registry,fresh_sources)`
keeps exact request/context correlation and terminal response ownership inside
the registry. Call `registry.fail(id)` on a terminal provider error. No retry is
implied by failure. Cancellation can race network dispatch; late successful HTTP
results still cannot revive the registry flight. Held leases retain their bounded
context until dropped, so the host must bound queued/in-flight provider workers.

`provider_queue::ProviderQueue` (feature `grok`) reuses conversion-jobs' bounded
scheduler:1..8workers,1..32queued and1..32retained jobs. It owns the explanation
registry and maps its exact request IDs to SHA256 scheduler IDs (scheduler IDs
cannot contain colons). `submit` requires `UsageIntent { user_requested:true,
allocation }` and an explicitly configured `Arc<GrokClient>`. The intent is an
accounting label supplied after host authorization; it cannot verify a balance or
enforce a dollar cap. `usage` reports scheduler evidence, not provider token/dollar
usage. No automatic retry, purchase, credential discovery, or edit is performed.

Source validation occurs during admission; workers share only the bounded immutable
Context. The host must call `revoke_stale` on source changes and refresh/poll status
to cancel expired queued work. `receive` always validates fresh source before
returning a proposal. `cancel` does not release a running HTTP slot prematurely;
`retire` fails until that call actually returns. Ready results are consumed once,
and the record is released on consumption. Retire failed/cancelled records
explicitly to reclaim retention. Shutdown prevents new work but cannot forcibly
interrupt an already-running HTTP call; that call retains its configured timeout.
Native UI wiring remains pending, and no live provider call
has been used in its tests.

`ProviderQueue::snapshot()` exports bounded JSON-serializable job identities,
allocation labels and lowercase states plus queued/executing/retained counts.
`scheduler_tasks_started` counts scheduler starts, including preflight rejection;
it is not a count of HTTP requests or billable calls. Provider billing remains
explicitly unknown. Snapshots exclude source text, proposals, credentials and
provider error bodies. Polling propagates expired requests to scheduler cancellation.

With `--features grok`, native hosts can explicitly launch
`--provider-session FRESH_SESSION_ID MODEL_ID` with `FLASHTEX_GROK_API_KEY` supplied
in the child environment from their credential adapter. Never place keys in argv,
JSON commands, logs, or project files. Default `--session` remains offline even if
that environment variable exists. Provider startup builds a2worker/8queued/16retained
queue and a90second HTTP client but performs no call until explicit admission.

Use the same JSONL envelope with
`action:{"operation":"provider","command":{...}}`. Provider commands:
- `admit`: existing `input` prepare object, `timeout_ms`, `user_requested:true`,
  and `allocation` audit label. Returns `provider_admitted` with request ID.
- `poll`: request ID and complete `current_sources`; returns current status or
  consumes a ready validated proposal (`applied:false`). Does not wait for HTTP.
- `cancel` / `retire`: request ID, with running-slot retention as documented above.
- `revoke_stale`: project ID and complete current sources.
- `snapshot`: bounded queue status, no source/key/provider bodies.

The native host must keep reading/writing off the UI thread, propagate source
changes, poll/retire jobs, supervise helper lifetime, and request user review before
applying proposals. Provider subprocess tests verify startup/admission gates and
zero scheduled tasks; local HTTP and scheduler tests verify the lower-level
workflow separately. A complete native→helper→liveGrok integration remains untested.

Unix native Rust consumers can use `SessionClient::spawn_provider(executable,
session,model,key)` to launch that explicit mode under the same pipe deadlines.
It validates inputs before spawning and passes the key only through the child
environment. Ordinary `SessionClient::spawn` removes that provider variable from
the child's inherited environment. Startup does not submit jobs; the first status
snapshot reports zero scheduler starts. This is not a keychain adapter: the native
host still owns secure credential retrieval and user authorization.

`ProposalReview::prepare(request_id,context,response,current_sources)` revalidates
provider JSON and exposes an immutable proposal plus a review digest covering
request identity, full source binding and exact proposal. Explain-only results
have no edit handoff. Multi-document edits are refused because the existing grouped
ledger API is single-document; no partial plan is emitted.

After displaying the proposal, the host passes explicit approval and that exact
digest to `approve`, with a fresh complete source snapshot. It returns an
`ApprovedGroup` envelope identifying project/path/request/review and the existing
`history::GroupedEdit` payload. Check the selected ledger's identity with
`check_target` before passing its group to `Store::apply_group`; the ledger enforces
revision/hash/removed text. Preserve the unchanged approved group for idempotent
retry after an uncertain response. Do not rebuild against post-edit source or issue
a fresh command ID. Tests use the real Store for apply/retry/undo/reopen and prove
that retry after undo does not reapply the edit. The adapter never writes a ledger
or applies provider output on its own. User interaction and correct ledger routing
remain host responsibilities; an approval flag is not independent proof of consent.
