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

Startup in either store or rooted-project mode may supply explicit
`bibliography_paths`; paths identify non-entry sources, with no duplicates.
Rooted mode imports declared files through the existing bounded project capability,
preferring retained durable ledgers over disk. Declarations must be supplied again
on reopen, including dynamically attached bibliography sources. No filename
extension inference. The application owns persistence of its startup configuration.

`open_document` accepts optional `document_kind`: `latex` (default) or
`bibliography`. Source edits, undo/redo, compiler restart, and membership changes
preserve surviving indexed kinds. Reattaching a detached bibliography source must
explicitly select `bibliography`; the generic default remains LaTeX. Membership
generation guards reject proposals predating detach/reattach even when revisions
are unchanged. No live native application adoption has been verified yet.

The proposal JSON limit bounds encoded output, not RSS; parsing the bounded export
into the helper envelope temporarily holds both forms. Complete outer frames also
pass the existing output check. No truncation is used to satisfy a proposal limit.

Checkpoint verification: actual helper process test covers two source documents,
Unicode byte offsets, citation comment/verbatim exclusion, read-only proposals,
small serialization budget refusal, and stale plans after another document changes.

Rooted helper lifecycle coverage includes explicit non-.bib source import, typed
detach/reattach, stale membership refusal, edit/undo/redo, and process reopen from
durable source after the disk file is deleted. Invalid declarations and extension
non-inference are checked independently.
