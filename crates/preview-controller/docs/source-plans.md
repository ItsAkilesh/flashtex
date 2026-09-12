# Native source proposals (FT-048 revision 5, in progress)

The helper exposes `plan_literal_replacement`, `plan_citation_rename`, and
`plan_citation_rename_at`. Each requires the complete `source_versions` map and
`membership_generation` returned by `snapshot`, plus `max_bytes` in 1..8388608.
Responses contain `plan`, the existing versioned project-index export with decimal
string offsets/revisions, `proposal_only` and `requires_user_approval`. These
commands never apply edits. Existing explicit `apply_group` remains separate;
no project-wide atomic application is promised.

Literal requests also require `literal`, `replacement`, `max_matches` (1..1000),
`max_work` (1..1000000), and optional `documents`. Incomplete searches are refused.
Citation requests require `old_name`/`new_name`; the `_at` variant instead takes
`path`, `start_byte`, `end_byte` identifying the exact lexical key span.

Store-based startup may supply explicit `bibliography_paths`; paths must identify
attached non-entry stores, with no duplicates. No filename-extension inference.
Declarations must be supplied again on reopen. Source edits retain their indexed
kind, and ordinary membership changes preserve surviving kinds. A detached source
reattached through the current generic endpoint is LaTeX; an explicit typed attach
path and project-root bibliography imports remain required work tracked in GH #30.
Project-root mode rejects nonempty bibliography declarations until that support is
implemented. Native citation adoption must wait for these lifecycle gates.

The proposal JSON limit bounds encoded output, not RSS; parsing the bounded export
into the helper envelope temporarily holds both forms. Complete outer frames also
pass the existing output check. No truncation is used to satisfy a proposal limit.

Checkpoint verification: actual helper process test covers two source documents,
Unicode byte offsets, citation comment/verbatim exclusion, read-only proposals,
small serialization budget refusal, and stale plans after another document changes.
