# flashtex-diagnostic-explanations

Original, actionable explanations and previewable fix suggestions for FlashTeX
compiler diagnostics. Pure Rust, edition 2024, **no external crates, no model or
network calls**. Owner: `mac-diagnostic-explanations` (issue #2). Consumer: the
Mac diagnostics panel (proposal below). It does not modify the compiler.

```sh
cd crates/diagnostic-explanations
cargo test            # 59 tests
cargo clippy --all-targets
```

## What it does

Input: a runtime-v1 diagnostic `{severity, message, source, recovery}`
(`docs/contracts/runtime-v1.md`), the exact text of the document `source.path`
names (same revision), and the app's supported-command list.

Output: an `Explanation`:

| Field | Meaning |
|---|---|
| `catalog_id` | Stable id of the matched catalog entry, or `null` when nothing matched. |
| `title` | Short, specific headline (e.g. `\begin{itemize} is never closed`). |
| `category` | One of `unsupported-command`, `unmatched-brace`, `environment-mismatch`, `unsupported-environment`, `math-unsupported`, `math-syntax`, `missing-argument`, `empty-argument`, `preamble-unsupported`, `macro-definition`, `multi-file`, `path-rejected`, `protocol`, `unknown`. |
| `why` | Why the compiler complained, in the author's terms. |
| `what_happened` | What the compiler did instead, built from the diagnostic's `recovery` text. |
| `suggestions[]` | `{text, confidence: low\|medium\|high, edits: [{path, start_byte, end_byte, replacement}]}`. Edits are candidates for preview; **nothing is applied by this crate**. `edits` is empty for advice with no mechanical form (paths, protocol). |
| `context` | Bounded excerpt: the span's line(s) ± one line, never more than 200 bytes beyond either end of the span, window and span ends always on scalar boundaries, plus 1-based `line`/`column` and `span_in_bounds` (false when the span does not fit the supplied text). |

API (`src/lib.rs`):

```rust
explain(&Diagnostic, document_text: &str, supported_commands: &[&str]) -> Explanation
explain_all(&[Diagnostic], &[Document{path,text}], supported) -> Vec<Explanation>
explain_compile_result_json(compile_result_json, documents, supported) -> Result<Vec<Explanation>, JsonError>
Explanation::to_json() / Explanation::from_json()      // fixed key order, hand-written
json::explanations_to_json(&[Explanation])
preview::preview(text, &Suggestion, original_span) -> Result<Preview, PreviewError>
preview::apply_edits / changed_region / rebase          // same rules as the Mac's SourceMapping
DEFAULT_SUPPORTED_COMMANDS                              // mirrors Completion.defaultSupported
```

All offsets are zero-based, end-exclusive UTF-8 byte offsets into the exact
document text the compile ran on, the contract's currency. Results are
deterministic for identical inputs (tested).

## Catalog (38 entries, 45 message templates)

Matching is on the compiler's exact message text; `*` marks an interpolated
value that is captured. Provenance is `file:line@sha`:

- `1dd26c5` = `origin/main` 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863
- `de1020c` = `origin/agent/claude/compiler-foundation` (math, display math)
- `6b13034` = tip of that branch at cataloguing time (preamble and macro messages)

`tests/catalog_strings.rs` embeds one real message per template with the same
citation, and pins the counts above. When the compiler rewords a message the
test fails and the entry must be re-pinned; that coupling is intended.
Diagnostic wording is not prescribed by the corpus (`tests/tex-corpus/README.md`),
so this crate never asks the compiler to change a string.

| id | category | message template | source |
|---|---|---|---|
| `unclosed-group` | unmatched-brace | `unmatched '{' — group never closed` | `crates/compiler/src/parser.rs:57@1dd26c5` |
| `stray-close-brace` | unmatched-brace | `unmatched '}' — no group is open here` | `crates/compiler/src/parser.rs:124@1dd26c5` |
| `unterminated-environment` | environment-mismatch | `unterminated environment '*' — no matching \end` | `crates/compiler/src/parser.rs:64@1dd26c5` |
| `environment-mismatch` | environment-mismatch | `\end{*} does not match \begin{*}` | `crates/compiler/src/parser.rs:190@1dd26c5` |
| `stray-end` | environment-mismatch | `\end{*} with no matching \begin` | `crates/compiler/src/parser.rs:197@1dd26c5` |
| `unsupported-environment` | unsupported-environment | `environment '*' is not implemented; its body is typeset as plain text` | `crates/compiler/src/parser.rs:179@1dd26c5` |
| `math-mode-unavailable` | math-unsupported | `math mode is not implemented in this version` | `crates/compiler/src/parser.rs:133@1dd26c5` |
| `unsupported-command` | unsupported-command | `\* is not supported by this compiler version; unrestricted TeX math mode is not implemented` | `crates/compiler/src/parser.rs:235@de1020c` |
| `unsupported-command` | unsupported-command | `\* is not supported by this compiler version` | `crates/compiler/src/parser.rs:212@1dd26c5` |
| `missing-braced-argument` | missing-argument | `\* requires a braced argument` | `crates/compiler/src/parser.rs:234@1dd26c5` |
| `argument-unclosed` | unmatched-brace | `argument to \* is missing its closing brace` | `crates/compiler/src/parser.rs:280@1dd26c5` |
| `empty-documentclass` | empty-argument | `\documentclass was given an empty argument` | `crates/compiler/src/parser.rs:310@6b13034` |
| `empty-package-list` | empty-argument | `\usepackage was given an empty package list` | `crates/compiler/src/parser.rs:330@6b13034` |
| `empty-argument` | empty-argument | `\* was given an empty argument` | `crates/compiler/src/parser.rs:287@1dd26c5` |
| `path-rejected` | path-rejected | `rejected document path '*': paths must be project-relative with no parent traversal` | `crates/compiler/src/protocol.rs:244@1dd26c5` |
| `path-rejected` | path-rejected | `rejected entry_path '*': paths must be project-relative with no parent traversal` | `crates/compiler/src/protocol.rs:255@1dd26c5` |
| `no-documents` | protocol | `no documents supplied to compile` | `crates/compiler/src/protocol.rs:285@1dd26c5` |
| `multi-document` | multi-file | `* documents were supplied; this version compiles only the entry document` | `crates/compiler/src/protocol.rs:300@1dd26c5` |
| `protocol-error` | protocol | `line exceeds the *-byte limit` | `crates/compiler/src/protocol.rs:100@1dd26c5` |
| `protocol-error` | protocol | `protocol version * is not supported; this build speaks version *` | `crates/compiler/src/protocol.rs:124@1dd26c5` |
| `protocol-error` | protocol | `protocol_version is required` | `crates/compiler/src/protocol.rs:133@1dd26c5` |
| `protocol-error` | protocol | `compile requires a payload` | `crates/compiler/src/protocol.rs:144@1dd26c5` |
| `protocol-error` | protocol | `message type '*' is not supported` | `crates/compiler/src/protocol.rs:150@1dd26c5` |
| `protocol-error` | protocol | `type is required` | `crates/compiler/src/protocol.rs:152@1dd26c5` |
| `math-stray-close-brace` | unmatched-brace | `unmatched '}' in math mode` | `crates/compiler/src/math.rs:90@de1020c` |
| `duplicate-script` | math-syntax | `duplicate script on a math atom` | `crates/compiler/src/math.rs:110@de1020c` |
| `script-without-atom` | math-syntax | `script marker has no preceding math atom` | `crates/compiler/src/math.rs:117@de1020c` |
| `math-group-unclosed` | unmatched-brace | `math group is missing its closing brace` | `crates/compiler/src/math.rs:133@de1020c` |
| `script-missing-argument` | math-syntax | `math script is missing its argument` | `crates/compiler/src/math.rs:159@de1020c` |
| `nested-math-delimiter` | math-syntax | `unexpected math delimiter inside math mode` | `crates/compiler/src/math.rs:188@de1020c` |
| `unsupported-math-command` | math-unsupported | `\* is not supported in math mode` | `crates/compiler/src/math.rs:230@de1020c` |
| `missing-math-argument` | missing-argument | `\* requires a braced math argument` | `crates/compiler/src/math.rs:255@de1020c` |
| `stray-display-close` | math-syntax | `stray \] has no matching \[` | `crates/compiler/src/parser.rs:147@de1020c` |
| `script-outside-math` | math-syntax | `math script marker used outside math mode` | `crates/compiler/src/parser.rs:155@de1020c` |
| `display-math-unclosed` | math-syntax | `display math is missing its closing delimiter` | `crates/compiler/src/parser.rs:333@de1020c` |
| `inline-math-unclosed` | math-syntax | `inline math is missing its closing '$'` | `crates/compiler/src/parser.rs:335@de1020c` |
| `packages-not-implemented` | unsupported-command | `packages * are recognised but not implemented` | `crates/compiler/src/parser.rs:339@6b13034` |
| `macro-name-invalid` | macro-definition | `\* requires a single command name as its first argument` | `crates/compiler/src/parser.rs:365@6b13034` |
| `macro-argcount-invalid` | macro-definition | `\* argument count must be an integer from 0 to 9` | `crates/compiler/src/parser.rs:382@6b13034` |
| `newcommand-exists` | macro-definition | `\newcommand cannot redefine existing command \*` | `crates/compiler/src/parser.rs:402@6b13034` |
| `renewcommand-undefined` | macro-definition | `\renewcommand cannot redefine undefined command \*` | `crates/compiler/src/parser.rs:415@6b13034` |
| `macro-recursion` | macro-definition | `macro \* exceeded the expansion recursion limit of *` | `crates/compiler/src/parser.rs:436@6b13034` |
| `macro-argument-undeclared` | macro-definition | `macro replacement references #* but that argument is not declared` | `crates/compiler/src/parser.rs:492@6b13034` |
| `optional-argument-unclosed` | unmatched-brace | `optional argument is missing its closing ']'` | `crates/compiler/src/parser.rs:784@6b13034` |
| `preamble-unsupported` | preamble-unsupported | `\* is not supported in the document preamble` | `crates/compiler/src/parser.rs:891@6b13034` |

Notes on provenance:

- The `protocol-error` templates come from `error` envelopes, not `compile_result`
  diagnostics; the Mac may surface them through the same panel. The
  `malformed_json` envelope carries the JSON parser's own text and is not
  catalogued (it falls back).
- The two `unsupported-command` templates are the same diagnostic on `main` and
  on the foundation branch (the branch appended a math clause). Both share one
  explanation and suggestion set.
- `\documentclass was given an empty argument` is listed before the parametrised
  `\* was given an empty argument`; catalog order decides such overlaps and a
  unit test (`templates_do_not_shadow_each_other`) forbids an earlier generic
  entry from stealing a later specific one.
- Corpus alignment: `unknown-command`, `unclosed-group`, `extra-closing-group`
  cases in `tests/tex-corpus/manifest.json` are covered by `unsupported-command`,
  `unclosed-group`, `stray-close-brace`. `missing-include` (`\input`) has no
  compiler message yet on either branch; it currently surfaces as
  `unsupported-command` for `\input`.

## Fallback

A message outside the catalog still yields an explanation: category `unknown`,
title derived from the first clause of the message, `what_happened` from
`recovery` (or a severity-appropriate sentence when `recovery` is null), the
same bounded context, and one generic advice suggestion with no edits.

## What is heuristic

Everything under `suggestions` is a guess about intent; confidence encodes how
often the assumption holds. Titles, `why`, and `what_happened` are not heuristic
(they restate the compiler's own report).

| Kind | Heuristic | Assumption / limit |
|---|---|---|
| Unclosed `{` / unclosed argument | Insert `}` at end of the line (if content follows on it) or before the next paragraph break (blank line), else EOF; or remove the `{`. | Groups rarely span paragraphs (arguments cannot: `required_argument` stops at `ParBreak`). Comments and escaped braces are skipped during scans. |
| Stray `}` | Remove, or escape as `\}`. | No attempt to find "where the group should have opened". |
| Unterminated environment | After an unterminated `\begin{X}` every later environment is balanced (an unbalanced `\end` would have closed X), so a depth-0 scan finds sibling structure: insert `\end{X}` before the next depth-0 `\section`/`\subsection`/`\end{document}` (medium), after the opening paragraph, before the next depth-0 `\begin`, or at EOF (distinct positions only, first is medium, rest low). `document` closes at EOF with high confidence. | Sectioning commands beyond `section`/`subsection` are not recognised. |
| `\end{a}` vs `\begin{b}` | Rename to `\end{b}` (high when the names are within OSA distance 2, else medium); or insert `\end{b}` before it (low). | The span is the `\end` token; the argument is re-read from the text. |
| Unsupported command | Closest names from the app-supplied list by optimal-string-alignment distance (transposition = 1), budget `clamp(len/3, 1, 3)`, prefix/extension matches allowed, ties alphabetical, at most 3; distance ≤1 high, 2 medium, else low. Then "remove and keep the argument text". | The list is the app's; `DEFAULT_SUPPORTED_COMMANDS` is only a fallback and mirrors the Mac `Completion.defaultSupported` (core from `main`, math from `de1020c`). |
| Math on `main` | Remove the `$…$` delimiters (keep words) or the whole formula; plus advice that math is outstanding. | Only the next unescaped `$` is paired. |
| Math (foundation) | Group `{x^a}^b` for double scripts (backward scan over one operand and one marker), `^{}` filler, `\$` escape for a nested `$`, `}` at the formula's last token, wrap `x_i` in `$…$` or escape `\_` when scripts appear in text. | Backward operand scan is raw (does not skip comments). |
| Preamble body command | Delete from the preamble and insert after `\begin{document}` (two edits), or delete. | Requires a `\begin{document}` later in the same file. |
| Macros | `\newcommand`⇄`\renewcommand` swap (high); insert/replace `[n]` on the definition found by `\newcommand{\name}` text search; drop an invalid `[…]` option. | Definition search is textual and takes the first match. |
| Paths / protocol | Advice only; a sanitised relative path is proposed in text. | No document edit exists for these. |

Stale text: when the span does not fit the supplied text (different revision) or
splits a scalar, builders emit advice without edits and `context.span_in_bounds`
is false. A property test drives every template with pseudo-random spans and
asserts no panics and that every emitted edit still applies.

## Fix previews (follow-up 1)

`preview::preview(text, &suggestion, Some((start, end)))` applies the edits to a
copy (validated: one path, in bounds, on scalar boundaries, non-overlapping;
stable order for insertions at one offset), returns the full `after_text`,
whole-line `before_excerpt` / `after_excerpt` capped like the context window,
and:

- `changed`: the single differing region by common prefix/suffix widened to
  scalar boundaries: `SourceMapping.changedRegion` in
  `apps/mac/Sources/FlashTeXProtocol/SourceMapping.swift`, ported byte for byte.
- `original_span_after` / `span_still_valid`: the diagnostic's span rebased with
  `SourceMapping.rebase` semantics (before the region: unchanged; after: shifted
  by the length delta; overlapping: `None`), and additionally required to spell
  the same bytes. When `false`, the underline must be dropped until recompile,
  which is what `EditorDiagnostics.marks` already does.

## JSON

`Explanation::to_json()` writes compact JSON with a fixed key order
(`catalog_id, title, category, severity, message, why, what_happened,
suggestions, context`), integers for offsets, RFC 8259 escaping. The parser is
minimal (objects, arrays, strings with `\u` and surrogate pairs, numbers,
literals, depth limit 64) and is used to read `compile_result` envelopes or bare
payloads and for the round-trip test.

## Native integration proposal (follow-up 2)

Target: the Mac diagnostics panel (`apps/mac`, owner mac-claude-a). Proposed,
not implemented; nothing here changes `runtime-v1`.

1. **Transport.** Add an FFI entry in the Mac's Rust bridge (or a tiny CLI in
   this crate, `flashtex-explain`, reading a `compile_result` JSON line plus the
   documents and writing the explanations array) so Swift decodes
   `[Explanation]` with `Codable` from the fixed-key JSON above. No new message
   type in `runtime-v1`; explanations are derived client-side from the
   `compile_result` the app already holds and the text it compiled.
2. **"Explain" affordance.** Each row in the diagnostics list gains an
   *Explain* disclosure: title as the row's headline (falls back to the
   message), category as an icon/tag, `why` and `what_happened` as two short
   paragraphs, and the context excerpt rendered monospaced with the
   `span_in_window()` range highlighted. `line`/`column` feed the existing
   jump-to-source action.
3. **"Preview fix".** Each suggestion with edits shows a *Preview fix* button.
   The app computes the preview against the *compiled* text, then rebases the
   edits onto the current buffer with `SourceMapping.rebase` (the crate uses the
   same rule, so the app can trust `span_still_valid` for the diagnostic
   underline). If any edit overlaps a region edited since the compile, the
   button is disabled with "document changed; recompile". Applying is a single
   undoable `NSTextView` replacement per edit, applied from highest offset to
   lowest, never automatic. Confidence maps to button prominence (high =
   default action, low = menu item).
4. **Supported list.** Pass `Completion.defaultSupported` (or the live list)
   into `explain`; the crate does not own that list.
5. **Batch.** Call `explain_all` once per `compile_result` and cache by
   `(project_id, revision)`; explanations for a stale revision are dropped with
   the result, as marks are today.
6. **Fallback.** Uncatalogued messages still render (category `unknown`), so
   compiler additions never blank the panel; the catalog test tells the owner
   of this crate when to add entries.

Limitations to state in the UI: suggestions are heuristics; the crate never
sees the compiler's parse state, only the message, span and text.

## Layout

```
src/lib.rs       types, explain / explain_all / explain_compile_result_json
src/catalog.rs   ENTRIES (templates + provenance), lookup
src/pattern.rs   template matcher (no regex)
src/context.rs   bounded, scalar-safe context window
src/suggest.rs   per-kind suggestion builders (heuristics)
src/text.rs      byte-offset scanning helpers, OSA edit distance
src/preview.rs   apply_edits, changed_region, rebase, preview
src/json.rs      writer + minimal parser
tests/catalog_strings.rs    every template ↔ a real compiler string, with citations
tests/suggestions.rs        behaviour, offsets, determinism, robustness
tests/json_and_preview.rs   round-trip, compile_result ingestion, previews
```
