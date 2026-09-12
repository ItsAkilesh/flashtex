# mac-completion-sync — CompletionTests green against the synchronized vocabulary

Lane of parent `mac-claude-a` on mac-m1max-a (Claude Code subagent, shared Claude
Max quota, no purchases). Bounded task from
`prompt-completion-sync.md`: make `swift test --filter CompletionTests` pass
after main f803a711 (Codex "synchronize completion vocabulary with compiler")
without weakening the tests.

## Checkpoint

- Branch `agent/mac-completion-sync/tests`, tip `6de0549e` (product) + this
  coordination commit. Worktree `.claude/worktrees/agent-a4bdf9366990b89b4`.
- Base: the parent's **local** `agent/mac-claude-a/mac-shell` at `624fcb14`
  (origin's copy was behind at `8d1ab8e2` and lacks f803a711). Consumed main
  through `738714a9` (font-size no-op arm), already in 624fcb14.
- Dirty files: none after commit.
- Files touched: `apps/mac/Sources/FlashTeXMac/Completion.swift`,
  `apps/mac/Tests/FlashTeXMacTests/CompletionTests.swift`, this file and
  `coordination/agents/mac-completion-sync.json`. No parent-retained file, no
  crate, no fixture.
- Next command for the parent: merge the branch into mac-shell; re-run
  `swift test --filter 'CompletionTests|EditorIntelligenceTests'` with the
  helper env when the 1-minute load is below 15 (it was 53–77 here).

## Starting failures (base 624fcb14)

`swift test --filter CompletionTests`: 27 tests, 38 failures. Two causes:
the vocabulary table lagged the README/parser (`\bigskip`, `\medskip`,
`\smallskip` missing; `\hfill`/`\hfil`/`\hspace`/`\quad`/`\qquad` sourced as
parser-only or math although the README list paragraph names them), and the
tests still expected the old 44-entry table (`\se` → one item, `\s` → five).

## Decisions

1. **Vocabulary sources follow the README's leading list paragraph.** The
   drift test now parses only that paragraph for "Supported commands": the
   prose after it quotes the user macro `\problem{...}{...}`, which is not a
   compiler command, and its dimension units (`pt`, `em`, …) are not
   environments. 51 documented commands, 9 environment mentions.
2. **Added entries** (all truthful to `crates/compiler/README.md` /
   `src/parser.rs` on main): `\bigskip` (12pt), `\medskip` (6pt),
   `\smallskip` (3pt) as README commands; `\tiny` … `\Huge` (10 names) as
   parser-only "accepted size declaration; a no-op because body text has one
   fixed size". `\quad`/`\qquad` became text entries (they work in text and
   math, so no `math ·` prefix). `\hfill`/`\hfil`/`\hspace` descriptions now
   say what the compiler does (infinite-stretch glue; units accepted).
3. **`; math mode only` dropped**, expectation fixed: `Entry.detail` already
   states the mode once via the `math ·` prefix; a test now asserts no
   description repeats it and that the prefix matches the mode for every entry.
4. **Ranking rule made explicit and implemented:** within the vocabulary the
   command spelled exactly as typed ranks first (`\sec` before `\section{...}`,
   `\it` before `\itshape`/`\item`); the rest keep table order (text entries,
   then math structures, operators, symbols in `COMMAND_GLYPHS` order). Open
   `\end{X}` closers still precede everything. Doc comment updated.
5. **Custom `supported:` list** — the document-typed unknown command is still
   listed after it, marked; the old expectation was wrong, not the code.
6. **Keyboard / Tab-VoiceOver / popup tests** use `\su` (seven candidates,
   under the 12 cap) so "narrowed" (`\sub`, three) and "widened" (seven) are
   exact counts; the cancel test asserts the session equals the pure
   function's (capped) output for `\s` instead of pasting twelve labels.
7. **Parser-arm regex** accepts capitals and multi-line arm groups so the
   `"tiny" | … | "Huge"` arm is checked; every parser-only entry must be
   backed by an arm (stale-entry guard, mirroring the math one).
8. **Known compiler README gap, pinned rather than hidden:** `COMMAND_GLYPHS`
   emits `\in`, `\forall`, `\exists`, `\vee`, `\Rightarrow`, `\mid` but
   README "Supported math" does not list them. `crates/compiler` is not this
   lane's to edit; the test asserts that exact list so a README fix (or a new
   omission) fails it.

## Evidence

- `swift test --filter CompletionTests` at 6de0549e: **27 tests, 0 failures,
  2 skipped** (both load-gated timing tests: 1-min load 66.7 > 20).
  Log: scratchpad `completion-run4.log` (session-local).
- `swift test --filter EditorIntelligenceTests`: **10 tests, 0 failures.**
- There is no `CommandDocsTests` class in `apps/mac/Tests` (the brief named
  one); `CommandDocs` is covered by EditorIntelligenceTests.
- Full `swift test` not run: 1-min load 53–77 during the lane (> 15 rule).

## Limitations

- Timing tests skipped under load; not measured here.
- The exact-match-first rule changes what a user who types a complete short
  command sees first (`\em` now precedes `\emph{...}`); this is intended and
  documented, but the parent may prefer the old table-only order.
