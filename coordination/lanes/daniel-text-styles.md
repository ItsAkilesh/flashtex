# Lane daniel-text-styles live log

## 2026-09-12T19:51:40Z — state checkpoint (first entry)
- Worktree: /Users/dqi26/ft-wt-text-styles, branch agent/daniel-text-styles/compiler
- HEAD: 862eca1c (pdf: resolve Courier and Helvetica font hints to base-14 variants)
- Pushed: 6e8a2542 (merge of origin/main on top of 0ca8b86c, compiler text styles). 862eca1c (pdf Courier/Helvetica) committed locally, NOT yet pushed.
- Uncommitted: 
- Design: parser.rs gains TextStyle {bold, italic, family Roman|Sans|Mono} on Inline::Text. Style commands (\textbf \textmd \textit \textsl \textup \emph \texttt \textrm \textsf \textnormal) re-enter their braced argument as an ordinary group (open_group pushes brace_stack/macro_scopes/style_stack) so math inside is parsed; unclosed arguments fall back to required_group (keeps recovery diagnostics). Declarations (\bfseries \mdseries \itshape \slshape \upshape \ttfamily \rmfamily \sffamily \normalfont \em \bf \it \sl \tt \rm \sf) mutate self.style; restored on '}' and on \end (env_styles stack). inlines_from_tokens (headings/captions/fallback) tracks style too; headings start TextStyle::BOLD.
- layout.rs style_font(): Roman -> Times-Roman/Bold/Italic/BoldItalic (Core 14 AFM metrics via font-engine); Sans -> Helvetica, Mono -> Courier (font-engine Core14 enum has no bold/oblique Helvetica/Courier, so those are upright-medium approximations). Slanted -> Times-Italic (no slanted Times).
- PDF: crates/pdf writer.rs already maps Times hints to base-14 variants; 862eca1c adds Courier/Helvetica family hints -> base-14 variants (previously substituted Times w/ warning). pdf README + tests/render.rs updated.
- Tests: compiler cargo test 107 passed (incl 3 new parser tests), clippy -D warnings clean, fmt clean. pdf cargo test 89 passed, clippy clean.
- HW1 (compiler --release + flashtex-pdf, caps rules-v1+font-hints-v1): bold title block, "11 PM ...", "Instructions.", "Problem N", "Katie:/Hasita:/Michelle:" render bold; "[4 points]" normalfont upright. Rasterized p0/p2 with pymupdf and viewed. Pre-existing warning: font "Symbol" substituted by Times-Roman (math lane, not touched).
- Known conflict: apps/mac CompletionTests.testStaticVocabularyMatchesTheCompilerDocs asserts README Supported commands count == 18 and mirrors Completion.Vocabulary; README now lists more commands -> Mac test needs Vocabulary update by Mac lane (I must not edit apps/mac).
- In progress: e2e sample (scratchpad s.pdf) shows only Times-Roman/Bold/Symbol fonts in PDF; investigating whether italic/Courier hints reach PDF.
- Next: debug that, push 862eca1c (git merge origin/main first), final report. Stop by 20:30Z.

## 2026-09-12T19:53:06Z — PDF faces verified, merged main, pushed
- Branch agent/daniel-text-styles/compiler @ f26f49af (pushed). Uncommitted: none (besides this log).
- Product commits for cherry-pick: 0ca8b86c (compiler text styles; README Supported commands updated in the same commit), 862eca1c (pdf Courier/Helvetica hints). Cherry-picking 0ca8b86c onto current main conflicts only in parser.rs BUILT_INS (main added vspace/hrule/newpage/pagestyle): keep both lists. Resolution is in merge f26f49af.
- The earlier "only Times-Roman/Bold in PDF" was a test-harness bug (bash expanded $x inside an unquoted heredoc), not a product bug.
- E2E sample (\textbf with math, \textit+\textbf, nested \emph, \textsl, \texttt, \textsf, {\bf}, {\it}, \bfseries...\normalfont) compiled + flashtex-pdf: PDF fonts Courier, Helvetica, Symbol, Times-Bold, Times-BoldItalic, Times-Italic, Times-Roman; raster viewed, every word in the right face, nested \emph upright.
- After merge: compiler cargo test 112 passed 0 failed; clippy -D warnings clean; fmt clean. pdf 89 passed.
- Commands: cd crates/compiler && cargo build --release; cd crates/pdf && cargo build --release; request JSON with payload.layout_capabilities ["rules-v1","font-hints-v1"] | crates/compiler/target/release/flashtex-compiler > res.jsonl; crates/pdf/target/release/flashtex-pdf --out x.pdf --verify < res.jsonl; rasterize with python3 pymupdf.
- Blocker for others: apps/mac CompletionTests.testStaticVocabularyMatchesTheCompilerDocs expects 18 README commands and mirrors Completion.Vocabulary; needs Mac-lane update (this lane may not edit apps/mac).
- Approximations: bold/italic sans and typewriter draw upright medium Helvetica/Courier in layout (font-engine Core14 has no Helvetica-Bold/Courier-Bold AFMs; compiler hints therefore say weight normal); \textsl/\slshape use Times-Italic. PDF writer can draw Courier-Bold etc. if a future hint asks.
- Next: GitHub #52 checkpoint comment; stop at 20:30Z. Possible follow-up: add Helvetica/Courier bold/oblique AFMs to font-engine (and vendored copy) so sans/mono bold is real.
