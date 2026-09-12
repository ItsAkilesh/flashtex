# HW1 visual audit — daniel-hw1-visual

Lane: `daniel-hw1-visual`. Branch: `agent/daniel-hw1-visual/audit`, worktree
`/Users/dqi26/ft-wt-hw1-visual`, based on `origin/main` (1275473b) merged with
`origin/agent/daniel-math-amsmath/compiler` (clean merge, no conflicts).
`origin/agent/daniel-parent/hw1-integration` did not exist at merge time, so the
compiler branch was used per fallback instructions.

**Status: first version, delivered under the 19:50Z partial-delivery deadline.**
Pages 1–3 are compared visually and the top defects are identified with source
pointers where I had time to confirm them; a few lower-priority items are
flagged as needing a follow-up pass (marked TBD below) rather than guessed at.

## Method

- Built `crates/compiler` (`flashtex-compiler`, JSON-Lines runtime-v1 worker) and
  `crates/pdf` (`flashtex-pdf`, envelope → PDF writer) in release mode. Neither
  crate is part of a Cargo workspace — each was built standalone from its own
  directory.
- Built a `compile` request envelope (`protocol_version:1`, `type:"compile"`,
  `payload.documents:[{path:"HW1.tex", text:<source>}]`, per the shape used in
  `crates/compiler/tests/acceptance.rs::compile_line`), fed
  `fixtures/real-world/hw1/HW1.tex` through `flashtex-compiler`, then piped the
  resulting `compile_result` envelope into `flashtex-pdf --out hw1_ours.pdf --verify`.
  Exit codes were 0 for both steps; `flashtex-pdf --verify` printed no warnings.
- The compile step reported 31 diagnostics (6 warnings, 25 errors) — not zero,
  but the task brief is right that a diagnostic count alone doesn't describe
  what's visually wrong, so the list below is built from the rendered pages.
- No `pdftoppm`/`pdftotext` on this machine. Rasterized both PDFs at 100 dpi
  with PyMuPDF (`pip install --user --break-system-packages pymupdf`, since
  `pip` refused an unmanaged install otherwise); `sips`/`qlmanage` were tried
  first but only produce page 1 of a multi-page PDF. Both PDFs are 3 pages.
  Viewed all 6 page images (ours vs. reference) directly.

## Ranked visible defects (most visible first)

1. **Bold/italic text formatting is silently dropped everywhere — text layout.**
   `crates/compiler/src/parser.rs`, the `"textbf" | "emph" | "textit"` arm
   (~line 531) does `para.extend(self.inlines_from_tokens(tokens))` — it
   re-emits the argument's tokens as plain inline text with no bold/italic
   attribute at all, and `"hfill" | "normalfont" | "bfseries"` (~line 538) is an
   explicit no-op with a comment that "the current layout model has no
   declaration-scoped font state." Reference: "**Problem 1**", "**Katie:**",
   "**Hasita:**", "**Michelle:**", title block in bold, all render bold.
   Ours: identical text, never bold, anywhere in the document (checked all 3
   pages). This is the single largest, most pervasive visual difference.

2. **`\mathbb{X}` renders as literal backslash-text, not blackboard bold — symbols.**
   Diagnostic: "`\mathbb` is not supported in math mode" (10 occurrences).
   Reference shows ℕ, ℤ, ℚ, ℝ as blackboard-bold glyphs throughout (e.g.
   "$a,b\in\mathbb{Z}_{>0}$" → "a, b ∈ ℤ>0"). Ours prints the raw command,
   e.g. "a,b∈\mathbbZ_{>0}" — the braces are silently dropped too, so it reads
   as one glued word "\mathbbZ". Appears on every page; this is the most
   visually confusing single defect since it looks like corrupted math rather
   than missing formatting.

3. **`\Longrightarrow` and `\setminus` also leak as literal command text — symbols.**
   Diagnostics: "`\Longrightarrow` is not supported in math mode" (1x),
   "`\setminus` is not supported in math mode" (2x). Page 2: reference shows
   "⟹" between two quantified formulas; ours prints the literal string
   "\Longrightarrow". Page 2 problem 6: reference shows "ℚ ∖ {0}" and
   "ℝ ∖ ℚ"; ours prints "\mathbbQ\setminus{0}" and "\mathbbR\setminus\mathbbQ".

4. **Custom enumerate labels from `enumitem`'s `[shortlabels]` are ignored, and
   the optional argument leaks onto the page — packages (enumitem).**
   `crates/compiler/src/parser.rs` `"item"` handling (~line 508) hardcodes
   `format!("{}.", count)` for any `enumerate` list — there is no path for a
   custom label scheme, and `\begin{enumerate}[(a)]`'s `[(a)]` argument itself
   is not consumed as a list option; it is typeset as a literal, bracketed line
   `[(a)]` immediately before the list. Reference: "(a) Suppose that a∣b...",
   "(b) Is the converse...". Ours: a stray "[(a)]" line, then "1. Suppose..."
   and "2. Is the converse...". Every lettered sub-list in the document is
   affected (Problems 1, 4, 5, and the bonus problem).

5. **Raw command/argument tokens leak into the visible body text — text layout.**
   Beyond `[(a)]` above: "empty" appears as its own line at the very top of
   page 1 (from `\pagestyle{empty}`'s argument), and "0.6em" appears as its
   own line after the Instructions/hrule area (from `\vspace{0.6em}`'s
   argument). Diagnostics confirm both commands are rejected
   ("`\pagestyle` is not supported...", "`\vspace` is not supported...") but
   the rejection path still typesets the braced argument text rather than
   dropping it silently. Same root cause class as #2–4 (an "unsupported
   command" fallback that prints argument tokens instead of only recording a
   diagnostic) but the affected commands here are page-style/spacing, not math.

6. **Missing `\hrule` separator under the title block — text layout.**
   Diagnostic: "`\hrule` is not supported by this compiler version." Reference
   has a full-width horizontal rule between the submission-deadline paragraph
   and the "Instructions" paragraph. Ours has no rule at all — just a blank
   gap (compounded by the stray "0.6em" line from #5 landing right where the
   rule should be).

7. **Title block isn't enlarged — `\Large`/`\LARGE` are rejected outright — text layout.**
   Diagnostics: "`\Large` is not supported...", "`\LARGE` is not supported...".
   Reference: "21-128 and 15-151" and "Problem Sheet 1" render distinctly
   larger and bolder than body text (with #1's bold problem also masking
   this). Ours: both lines render at plain body-text size, so the title block
   no longer reads as a title — it looks like ordinary paragraph text.

8. **`\hfill` never right-flushes the point count — text layout.**
   Same no-op arm as #1 (~line 538). Reference: "Problem 1" left, "[4 points]"
   right-flushed to the margin via `\hfill` inside the `\problem` macro. Ours:
   "Problem  1 [ 4 points]" runs both together left-aligned with irregular
   spacing (also missing the bold from #1), losing the two-column look on
   every problem header (6 problems + bonus).

9. **TeX quote/dash ligatures aren't converted to typographic characters — text layout / lexer.**
   Reference page 3: curly quotes and a real em-dash, e.g. `"I do not know"—`.
   Ours: literal ASCII ligature sequences survive verbatim, e.g.
   `` `` I do not know''---characterize ``. Same pattern on page 2's
   `` ``m divides n,'' `` (problem 3), which reference renders with real curly
   quotes. Likely `crates/compiler/src/lexer.rs` (not yet pinned to a specific
   function — TBD for a follow-up pass) since this is a tokenization-level
   ligature substitution, not a per-command dispatch.

10. **Page reflow diverges from the reference despite an identical page count (3 vs 3) — text layout (consequence, not a separate bug).**
    Because #1, #6, #7, #8 all remove vertical space the reference actually
    uses (no enlarged title lines, no rule, no `\vspace`, no bold-driven
    metrics), our page 1 ends earlier and page 2/3 content is shifted up by
    roughly half a page relative to the reference at every point compared.
    Coincidentally both documents still total 3 pages, but content that is on
    page 1 in the reference (tail of the Instructions paragraph) appears
    correctly, while later content (e.g. all of Problem 6 plus the Bonus
    Problem) is pulled up onto page 2/3 boundaries that don't match the
    reference's. Not independently ownable — fix #1/#6/#7/#8 and re-check.

## Lower-priority / not fully run down (flagging honestly rather than guessing)

- **Font substitution (UNOWNED / cross-cutting):** the reference (pdflatex)
  uses a Computer/Latin Modern face; ours appears to use a different serif
  face consistent with `crates/pdf`'s base-14 default-face path (no
  `--embed-font` was passed in this run). Visible in letterforms/kerning on
  close inspection but did not by itself misplace or corrupt any content, so
  ranked below the items above. Whether this is intentional default behavior
  of `flashtex-pdf` (embedding is opt-in per its own `--help` text) or a real
  gap is a question for whoever owns `crates/pdf`, hence UNOWNED here.
  Worth a second pass with `--embed-font auto` to see if it changes ranking.
- **`geometry[margin=1in]` (packages):** diagnostic says the package is
  "recognised but not implemented." Margins look close to 1in in both PDFs by
  eye at 100 dpi; did not measure precisely enough in the time available to
  confirm this is visibly wrong. TBD.
- **`microtype` (packages):** no visible protrusion/kerning defect found;
  likely not independently visible at this resolution. Not ranked.

## Owner summary (per the brief's categories)

- **symbols** (`\mathbb`, `\setminus`, `\Longrightarrow`): #2, #3
- **text layout** (`\setlength`, `\Large`, `\vspace`, `\hrule`, `\newpage`,
  `\pagestyle`, plus bold/italic and `\hfill`): #1, #5, #6, #7, #8, #9, #10
- **packages** (geometry/enumitem/microtype): #4 (enumitem), geometry/microtype
  flagged as TBD/not ranked above
- **amsmath/environments**: no independently visible defect found — display
  math (`\[...\]`), `\forall/\exists/\Rightarrow/\Leftrightarrow`, and basic
  fractions/exponents all rendered correctly in the pages checked.
- **UNOWNED**: font substitution (see above)

## Artifacts (not committed — reproducible from this report)

Rasters and intermediate JSON were written under this session's scratchpad,
not the repo, since they're regenerable: request envelope, `compile_result`
JSON, `hw1_ours.pdf`, and `{ref,ours}_p{1,2,3}.png` at 100 dpi.
