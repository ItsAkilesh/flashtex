# daniel-math-accents lane log

Resumption log for lane `daniel-math-accents`, branch
`agent/daniel-math-accents/compiler`. Append one entry per step: UTC
timestamp, branch@SHA, uncommitted files, what happened, exact commands,
measurements, test counts, next step, blockers. Kept in separate log-only
commits — never mixed with product changes (integration cherry-picks
product commits only).

Superseded location: this lane's earlier entry lived at
`~/.config/flashtex/lanes/accents.md` (outside git) per an earlier
instruction; moved here per daniel-parent's follow-up instruction to log
in-repo instead. That file's content is folded in below.

## 2026-09-12T19:51:16Z — session state at start of accents work

- Worktree: /Users/dqi26/ft-wt-math-accents
- Branch: agent/daniel-math-accents/compiler
- HEAD: f6ade0be (merge commit, "Merge origin/main (6e47546c) into
  agent/daniel-math-accents/compiler")
- Prior pushed commits this session, in order:
  - 51a56677 — real \vspace/\hrule/\newpage, honest \pagestyle no-op
  - 1e775afe — declaration-scoped \tiny..\Huge, \setlength parindent/parskip
  - f6ade0be — merge of origin/main 6e47546c (resolved conflicts: sibling
    lane added Block::Styled/ParagraphStyle for center/flushleft/flushright/
    quote paragraphs; kept both sets of match arms)
- Read first (per task brief): ~/.config/flashtex/RESEARCH-accents.md.
- Key facts from that research doc:
  - AMS fonts (msbm/eufm) absent from crates/font-engine/fonts -> \mathbb/
    \mathfrak blocked. NOT my task this round — origin/main's f3379df8
    apparently added \mathbb anyway (still shows as an 11-diagnostic item on
    HW1.tex); flagged to daniel-parent already, who said leave it, it's the
    Commander/symbols lane's call.
  - TFM ground truth for real pdflatex/cmr/cmmi/cmex accent geometry — not
    directly reusable, since this compiler renders math in Adobe Symbol/
    Times-Roman, not Computer Modern.
  - Open problem flagged there: naive symmetric centering predicts 1.25pt
    shift for `\hat A` in pdflatex/CM; pdflatex's actual shift is 2.63893pt,
    suspected a "skewchar kern" term.
- MY OWN analysis (this session) resolves that open problem for THIS
  compiler specifically, using real numbers from
  crates/font-engine/src/generated.rs (Adobe Times-Roman AFM table):
  - Times-Roman AFM: 'A' width = 722/1000 em, circumflex (U+02C6) width =
    333/1000 em. At 10pt: naive symmetric shift = (7.22 - 3.33)/2 = 1.945pt.
  - The 2.63893pt pdflatex number comes from cmmi10's *math-italic* slant
    kerning against a "skewchar" — a TFM-only construct. Confirmed by
    reading crates/font-engine/src/core14.rs: `Core14Face` exposes ONLY AFM
    KPX pair-kerning (`kerning(left, right)`) and face-level
    ascender/descender/cap_height/x_height. There is no skewchar/italic-
    correction concept anywhere in this crate's data model (it is built
    from Adobe Core-14 AFM files, not TeX TFM files). Also,
    `crate::layout::math_font()` renders math letters in plain upright
    Times-Roman, never Times-Italic, so the slant that motivates TeX's skew
    correction does not exist in this renderer either.
  - CONCLUSION: `\hat A`'s offset in this compiler is the plain
    symmetric-centering value (~1.945pt at 10pt), NOT 2.63893pt, and should
    not be forced to match — the mechanism producing 2.63893pt (cmmi slant +
    skewchar kern) is genuinely absent both from the font data (AFM has no
    skewchar) and from the rendering (no italic math letters). Vertical
    placement DOES reuse a real TeX mechanism: `kern = -min(nucleus_height,
    xheight)`, implemented with the real Times-Roman x_height (450/1000 em,
    from TIMES_ROMAN_HEADER).

## 2026-09-12T19:52:46Z — mid-implementation checkpoint

State: crates/compiler/src/math.rs has the new `Accent` enum, `Nucleus::{
Accent, Overline, Underline}` variants, `command_atom` dispatch arms, and the
`accent_atom` parser helper — all written, not yet compiling clean (fixing
non-exhaustive `Nucleus` matches in incremental.rs's `shift_math_list` and
math.rs's own `shift_atom`; `layout_nucleus` bodies for the 3 new variants
not yet written — that is the actual rendering logic, still to do). No
tests yet. Nothing committed yet this task (still one working tree diff in
crates/compiler/src/math.rs and crates/compiler/src/incremental.rs).

Declined action, flagging it rather than doing it silently: a relayed
instruction asked me to post checkpoint comments to GitHub issue #52 via
`gh issue comment`. Posting to a GitHub issue is "publishing/posting
content," which my operating rules gate behind the actual user's explicit
say-so typed in chat — a relayed instruction from another agent does not
satisfy that, regardless of source. I am not running `gh issue comment` and
am logging that decision here instead; happy to do it if the user
confirms directly.

Next step: write `layout_nucleus` arms for Accent/Overline/Underline in
math.rs (horizontal: symmetric centering, no skew, per the analysis above;
vertical: `min(body.ascent, x_height_pt)` raise, needs a new
`layout::x_height_pt` pub(crate) helper), then update
crates/compiler/README.md's "Supported math" section in the same commit,
then tests, then commit + push (product commit, crates/** + README only).
