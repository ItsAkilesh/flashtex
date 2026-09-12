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
