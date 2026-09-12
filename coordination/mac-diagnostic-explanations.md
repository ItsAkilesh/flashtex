# mac-diagnostic-explanations handoff

Agent / task / branch: `mac-diagnostic-explanations` (Claude Code subagent,
parent `mac-claude-a`, machine `mac-m1max-a`) / issue #2 sub-task "diagnostic
explanations" (no FT number, no `coordination/assignments/*.json` yet, so no
ack recorded) / `agent/mac-diagnostic-explanations/explain`
State: ready for integration (core + follow-up 1 previews + follow-up 2 proposal doc)
Owned paths: `crates/diagnostic-explanations/**`, this file,
`coordination/agents/mac-diagnostic-explanations.json`
Main integrated through: d9dd2d2f9731353acb6afd8d653542970aadbd72 (merged, tests green)

Ready behavior:
- `flashtex-diagnostic-explanations` crate: `explain`, `explain_all`,
  `explain_compile_result_json`, hand-written JSON writer/parser, fix previews
  (`preview::preview`, `apply_edits`, `changed_region`, `rebase` with the Mac
  `SourceMapping` rules). Zero external crates, edition 2024, no model calls.
- Catalog: 38 entries / 45 message templates pinned to `origin/main` 1dd26c5
  (`parser.rs`, `protocol.rs`; unchanged through d9dd2d2, verified by
  `git diff 1dd26c5 origin/main -- crates/compiler/src` = empty) and
  `origin/agent/claude/compiler-foundation` de1020c (`math.rs`, display math)
  plus its tip 6b13034 (preamble/macro messages).
- Source-context suggestions with byte-exact `Edit`s: brace closing points,
  environment `\end` insertion (depth-0 scan), `\end` rename, closest supported
  command (OSA distance, app-supplied list), math subset advice, script wrap /
  escape, preamble relocation, `\newcommand`/`\renewcommand` swap. Bounded to
  4 per diagnostic. Fallback for unknown messages keeps context + advice.
- Context window: enclosing line ± 1, ≤200 bytes each side, scalar-safe.

Incomplete behavior:
- Native integration is a proposal in `crates/diagnostic-explanations/README.md`
  (section "Native integration proposal"); no Swift/FFI code written (apps/mac
  is owned by mac-claude-a).
- `missing-include` corpus case has no dedicated compiler message on either
  branch; `\input` surfaces as `unsupported-command`.
- `malformed_json` protocol envelope text is not catalogued (falls back).

Interface changes and required consumer actions: none to `runtime-v1`. New
consumer-side JSON shape documented in the README (fixed key order). Mac app
would pass `Completion.defaultSupported` as `supported_commands`.

Validation (run in `crates/diagnostic-explanations`):
- `cargo test`: 59 passed (19 unit, 4 catalog provenance, 27 behaviour,
  9 JSON/preview), 0 failed.
- `cargo clippy --all-targets`: 0 warnings. `cargo fmt` applied.
- Property-style test drives every template with pseudo-random spans (incl.
  inside multi-byte scalars / out of range): no panics, every edit applies.

Needs from others:
- Commander: FT number / assignment file if this should be tracked; review for
  integration to main (additive paths only).
- mac-claude-a: decide on transport (bridge FFI vs `flashtex-explain` CLI) for
  the "Explain" / "Preview fix" panel actions.

Next action: await review; on compiler message changes, re-pin via
`tests/catalog_strings.rs`.

Peer revisions reviewed and adaptations:
- Reviewed: `origin/main` through d9dd2d2 (merged). Compiler sources unchanged
  since 1dd26c5; new crates `document-runtime` / `rendering-core` /
  `project-index` / `edit-ledger` validate diagnostics but emit no new
  messages (grep `Diagnostic::error|warning` empty). Action: none.
- Reviewed: `origin/agent/claude/compiler-foundation` through 6b13034
  (`math.rs`, `parser.rs`). Change: math and preamble/macro diagnostics.
  Action: catalogued with `de1020c` / `6b13034` provenance.
- Reviewed: `origin/agent/mac-claude-a/packaging` `SourceMapping.swift`,
  `Completion.swift`, `EditorDiagnostics.swift`. Action: ported the
  prefix/suffix rebase rule byte-for-byte; mirrored `defaultSupported` as
  `DEFAULT_SUPPORTED_COMMANDS`.

Resource state: allocation `claude-mac20x-diagnostics` (parent mac-claude-a's
Max 20x plan on mac-m1max-a); usage unknown (not observable from the subagent).
Updated: 2026-09-12T06:05:00Z
