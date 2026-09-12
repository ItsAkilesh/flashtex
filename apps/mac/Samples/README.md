Samples for manual checks of the Mac shell. These are hand-written examples in
runtime v1 shape owned by apps/mac; the authoritative contract fixtures live in
`protocol/fixtures/` (Commander-owned). `capture-proposal.json` loads via
Edit > Open Capture Proposal… (⌘⇧I) after pinning an insertion point (⌘⇧P).
`multipage-result.json` / `multipage-request.json` load via File > Open Compile
Result Fixture… (⌘O) on the result file: two pages, non-ASCII source text, and
two diagnostics for click-to-source, caret-sync, and diagnostics checks.

`recovery-demo.tex` is a short document that exercises sections, `\textbf`/`\emph`,
non-ASCII words, and two deliberately unsupported constructs (`$x^2$`, `\foo`) so
the compiler's `recovered` status and diagnostics are visible; see README "Launch hooks".
