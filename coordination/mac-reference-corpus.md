# mac-reference-corpus — user-assigned reference testing

State: local reference increment ready; publication/notification blocked.
Agent: Codex on mac-m1max-a. Assignment: direct user request 2026-09-12;
no Commander task/revision issued, no ACK invented.
Capabilities: Python, installed MacTeX 2026 pdfTeX/pdfLaTeX/LuaLaTeX/XeLaTeX,
BibTeX/Biber, Ghostscript, native image contact-sheet review.

Owned paths: `tests/extended-tex-corpus/`, `tools/extended-tex-corpus/`, this handoff.
Role: original test projects and development-only reference outputs; no compiler,
native product, existing corpus, shared interfaces or global control mutations.

Ready: [corpus guide](../tests/extended-tex-corpus/README.md), 49 original projects,
42 positive PDFs/51 pages, seven expected diagnostic cases, eight exact UTF-8 edit
scenarios, package/font/dependency/source/PDF hashes and full logs. Covers TeX
execution, broad math/alignment, layout/tables, graphics/TikZ/PGFPlots, packages,
multifile/class/bibliography projects and Unicode profiles. Finite coverage only.
Validation: all 42 positive sources compiled with their declared engines; all 51
positive pages rasterized and inspected in contact sheets plus full-size spot
checks; seven negative diagnostic fragments observed. Seven harness tests pass:
`python3 -m unittest discover -s tools/extended-tex-corpus -v`.
`python3 tools/extended-tex-corpus/reference.py --validate`: 49 projects.
Incomplete: product compiler comparison, exhaustive coverage and remote integration.
Required next acceptance gate: original-compiler results against pinned references;
unsupported is not a rendering pass. Binary PNG case needs project-files adapter.

Reviewed: local product plan, existing tex-corpus/visual-corpus/real-world-corpus
READMEs and runtime-v1 compile contract. Adaptation: separate namespace; preserve
all existing ownership, distinguish engine profiles/diagnostics/pixels/bytes.
No fresh remote peer-diff review claimed after sandbox/network failures.
Git: actual local branch main at 82376ac9b028e3895fb98b7739703859cbd2eddb;
last successful fetched origin/main 7fad3182. Intended task branch
agent/mac-reference-corpus/extended-suite NOT created. Worktree path remains
/Users/jay3332/Projects/flashtex. All owned changes are new untracked files.
Unrelated existing .claude/, Cargo.toml, Cargo.lock, src/ preserved; no tracked diff.

Blockers: Git worktree creation cannot write .git refs; coord.py checkpoint cannot
write .git/flashtex/checkpoint.lock; coord.py register refuses main. Approval policy
never. GitHub issue creation failed connecting to api.github.com. Therefore neither
role notice nor recovery issue was delivered remotely; no commit/push exists.
[Prepared orchestrator notice](../tools/extended-tex-corpus/orchestrator-notice.md)
contains precise recovery evidence and requested role registration/integration.

Resources: current user-authorized Codex session only, quota/cost unknown; zero
child agents, external inference, Cursor/Claude calls, purchases or overages.
No build jobs or uncertain publications remain in flight. Initial oracle failures
were cache/font lookup and two unintended overfull paragraphs; corrected/rebuilt.
Scratch evidence: /private/tmp/flashtex-extended-reference-v1 through -v4;
final relevant PDFs/logs/provenance copied into owned repository paths.
ETA: local initial gate complete; publication/integration blocked, time unknown.
Next commands when access is restored: fetch, create intended branch in a separate
worktree from current origin/main, copy ONLY owned new paths, run seven tests,
review/stage those paths, publish using authorized truthful Mac author/provenance.
Do not merge the divergent local main wholesale. Send prepared issue body through
`gh issue create --repo flash-tex/flashtex --title 'mac-reference-corpus: reference suite' --body-file tools/extended-tex-corpus/orchestrator-notice.md`.
Further coverage priorities are listed in the corpus guide.
Context telemetry/native compaction control unavailable; no percentage inferred.

Updated: 2026-09-12T15:30:13.468589+00:00

## Access-restored publication checkpoint

Updated: 2026-09-12T16:24:57.954278+00:00
Worktree: /private/tmp/flashtex-reference-corpus
Branch: agent/mac-reference-corpus/extended-suite
Baseline: a91fc1015e6884ab4a451d5d32a626271edacbd4 (origin/main).
Previous sandbox/network blockers are resolved; old entries above are historical.
Own registration now created through coord.py register. Seven tests pass in this
isolated worktree. Reviewed current authority, kabir-transfer-pending and retained
root Text handoff: Commander transfer is pending; compiler and native ownership
remain fenced. Adaptation: publish corpus only, request integration/assignment,
then provide bounded compiler-result and minimal-reproducer work. No compiler code
changes or global control edits. Existing machine work is preserved in its checkout.
Publication uses documented observed Cursor-limit fallback, jay3332 primary author
and truthful Codex executor. No Cursor inference or new charge.

Publication outcome: corpus commit c4bed94f pushed non-force to own branch.
Draft review PR: https://github.com/flash-tex/flashtex/pull/42
Role notification delivered: https://github.com/flash-tex/flashtex/issues/41
Seven harness tests pass; raw TeX logs retain their original trailing whitespace
as evidence (git diff --check flags those generated logs). No authored-source
whitespace issue was reported. Await Commander integration/assignment; next
bounded scope is pinned-compiler diagnostics and minimal reproductions.
