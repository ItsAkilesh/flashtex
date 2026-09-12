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
