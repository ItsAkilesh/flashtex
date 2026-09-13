# CONTRACT: adoption proposal for `crates/compiler` and `crates/bibliography`

Status: **proposal, not agreed**. This document changes no code outside
`crates/bibtex-bst` and no shared contract. Owners: the compiler owner
(`crates/compiler`) and the bibliography owner (`crates/bibliography`,
mac-bibliography) decide whether and how to adopt this. It is written
alongside, not instead of, `crates/bibliography/ADAPTER-PROPOSAL.md`, which
already proposes wiring `flashtex-bibliography`'s hard-coded
`plain`/`unsrt`/`alpha` resolver into the compiler; this document describes
what changes if `crates/bibtex-bst` is adopted for the same job instead, or as
a fallback next to it.

## What `crates/bibtex-bst` is

A from-scratch Rust port of BibTeX 0.99e (`bibtex.web`, with the TeX Live
change file `bibtex.ch`) that reads an `.aux`/`.bst`/`.bib` set through a
`FileSource` trait and produces the exact `.bbl` and `.blg` bytes TeX Live's
`bibtex` binary would. It executes the actual `.bst` postfix language — every
built-in, `format.name$`, sorting, crossrefs, error recovery — rather than
re-implementing three styles' output by hand. See `src/lib.rs` for the public
API (`run`, `DirSource`, `Session`, `Output`).

## Why this is a different tradeoff from `crates/bibliography`

`flashtex-bibliography` hand-implements the *behavior* of `plain`/`unsrt`/
`alpha` against its own `Database`/`resolve`/`format_bibliography` API,
producing styled runs the compiler's layout consumes directly — no `.bbl`
text to re-parse, no `.bst` file needed, small and fast. Its ceiling is those
three styles (plus whatever else someone hand-ports) and its own
interpretation of edge cases.

`crates/bibtex-bst` runs *any* `.bst` file (the 11 that ship in its own
oracle: `plain`, `unsrt`, `alpha`, `abbrv`, `acm`, `ieeetr`, `siam`,
`apalike`, `plainnat`, `abbrvnat`, `unsrtnat`, and in principle any
third-party `.bst`), matches real BibTeX's output byte-for-byte on the 99
oracle cases (below), and inherits BibTeX's own error/warning text for
malformed `.bib` input instead of a re-derived approximation. The cost: its
output is `.bbl` text (LaTeX-ish: `\bibitem`, `\newblock`, TeX accent
commands like `{\'a}`), which must be *parsed*, not consumed as ready styled
runs, before the compiler's layout can use it — this crate does not do that
parsing today. It also needs a `.bst` file on disk (or embedded) rather than
being self-contained.

## How `\bibliography`/`\bibliographystyle`/`\cite` would flow

Today (`crates/compiler/src/parser.rs`, `bibliography`/`bibliographystyle`
arms, and `crates/compiler/src/bib.rs`) the compiler only supports manual
`thebibliography`/`\bibitem`/`\cite`; `\bibliography{...}` reports
`"\\bibliography requires BibTeX/biblatex .bib input, which this compiler
does not read"` and points the user at the manual path.

Adopting this crate would replace that error with, per compiled document:

1. **Collect**, during the existing parse (same step as
   `ADAPTER-PROPOSAL.md`'s step 1): `\cite{a,b}` spans, `\nocite{...}`,
   `\bibliographystyle{name}` (default `plain`), `\bibliography{stems}`.
2. **Synthesize an in-memory `.aux`**: one `\citation{key}` per collected
   cite (`\citation{*}` for a bare `\nocite{*}`), `\bibdata{stems}`,
   `\bibstyle{name}` — exactly what `latex` would have written. No `.aux`
   file from a prior LaTeX run is required or consulted.
3. **Run it**: `run(synthetic_aux_name, &fs, &Options::default())` where `fs`
   is a `FileSource` implementation backed by the compiler's already-loaded
   `documents` array for `.bib` (matching `\bibliography{...}`'s stems, `.bib`
   appended, same resolution rule `ADAPTER-PROPOSAL.md` step 2 describes) and
   by a bundled set of `.bst` files (the 11 above at minimum) for
   `FileKind::Bst`, keyed by `\bibliographystyle`'s argument. An unknown style
   name is a `FileSource::read` miss, which surfaces through `Output` as
   BibTeX's own `"I found no style file"`-shaped error — forward it as a
   `Diagnostic` on the `\bibliographystyle{...}` argument span the same way
   `ADAPTER-PROPOSAL.md` step 2 forwards `flashtex_bibliography`'s.
4. **Parse the `.bbl`** this crate does not parse `Output::bbl` today. A new,
   small parser (out of scope for this crate; proposed to live in
   `crates/compiler` or a new `crates/bibtex-bbl`) would turn each
   `\bibitem[label]{key} ... \newblock ...` block into the same
   `{key: (label, blocks-of-styled-runs)}` shape `flashtex_bibliography::
   format_bibliography` already produces, so the rest of the adapter
   (`ADAPTER-PROPOSAL.md` steps 3–5: citation typesetting, hanging-indent
   list, click-to-source spans) is unchanged and shared between both
   backends. This is the one genuinely new piece of work adoption requires;
   everything upstream of it (collect, synthesize `.aux`, run) is done.
5. **Diagnostics from `Output::warnings()`/`diagnostics()`** (already
   implemented, see `src/lib.rs`) carry no byte spans into the `.tex` or
   `.bib` source — BibTeX's own messages are line/entry-oriented text, not
   structured. Mapping "Warning--empty journal in knuth1984" back to a
   `Diagnostic` with a real span means matching the entry key against the
   collected citations' `.bib` position, which is possible but not done here.

## Incremental behavior

`Session` (`src/lib.rs`) is the piece this proposal reuses directly for
"recompile on keystroke": it records the exact file lookups one `run()` made
(the `.aux` chain, `\bibstyle`'s `.bst`, every `\bibdata` `.bib`, including
failed lookups) with content hashes, and `Session::run` replays the cached
`Output` when every one of those hashes still matches and the aux name and
`Options` are unchanged. Concretely, in the compiler:

- Editing a paragraph that doesn't touch `\cite`/`\bibliography`/the `.bib`
  content: the synthetic `.aux` text is byte-identical (same citations, same
  stems, same style), so `Session::run` is a hit — the whole `.bst` machine
  does not re-execute.
- Adding, removing, or reordering a `\cite`: the synthetic `.aux` changes
  (different `\citation` lines in different order), so `Session::run` is a
  miss and reruns in full. This is required, not just simpler: entry order
  (`unsrt`), labels (`alpha`), and crossref-inclusion counts all depend on
  the complete citation list, so a partial/incremental BibTeX run is not
  sound in general — full re-run on any citation-set change is the correct
  behavior, matching what BibTeX itself guarantees.
- Editing the `.bib` file: its hash changes, so `Session::run` reruns even
  though the `.aux` is unchanged (`session_tests::
  reruns_when_bib_changes_but_aux_does_not`, added by this revision).
- Switching `\bibliographystyle`: same, via the `.bst` hash
  (`reruns_when_bst_changes_but_aux_does_not`).

One `Session` per document (or per document set, if cross-document citation
lists are ever supported) is the natural granularity; `Session` itself holds
one cache entry, matching the compiler's existing one-`Session`-per-editable-
unit patterns elsewhere (see `crates/compiler/src/incremental.rs`, not
otherwise touched by this proposal).

## Oracle status (this revision, commit to follow)

- 99 cases (the original 95 plus 4 new capacity-limit cases described below).
- `.bbl` byte-identical: **99/99**.
- `.blg` byte-identical: **99/99** (also 99/99 after the looser
  banner/`Capacity:`/statistics-block normalization `tests/oracle.rs` reports
  separately).
- The oracle (`tests/oracle/generate.py`) shells out to the installed
  `bibtex` (BibTeX 0.99e, TeX Live 2026) exactly once, to *generate*
  `tests/data/expected/*`; `cargo test` never invokes an external binary.

### What changed from r2 (31571478)

The 6 previously-failing cases (`badstyle`, `builtins-accents`,
`builtins-crossref`, `builtins-explicit`, `builtins-people`,
`builtins-strings`) all first differed at
`Reallocated singl_function (elt_size=4) to 100 items from 50.` — TeX Live's
web2c `BIB_XRETALLOC` log line, emitted whenever one of BibTeX's originally
fixed-size Pascal arrays is dynamically grown. This revision emulates that
logging for the arrays exercised by the oracle:

| Array(s) | `.ch` section | Initial → growth | Verified |
|---|---|---|---|
| `singl_function` | [187]/[188] | 50 → +50 | measured (fixed all 6 failures) |
| `wiz_functions` | [200] | 3000 → +3000 | measured |
| `cite_list`, `type_list`, `entry_exists`, `cite_info` | [138] | 750 → +750 | measured (new `cite-overflow` case) |
| `field_info` | [226] | 5000 → jumps to `total_fields + 5000`, rechecked as cites are discovered | measured (same case) |
| `bib_list`, `bib_file`, `s_preamble` | [242]/[123] | 20 → +20 | measured (new `bib-files-overflow` case) |
| `glb_str_ptr`, `global_strs`, `glb_str_end` | [216] | 10 → +10 | measured (new `glob-str-overflow` case) |
| `lit_stack`, `lit_stk_type` | [307] | 50 → +50 | measured (new `lit-stack-overflow` case) |

"Measured" means: a standalone `.aux`/`.bst`/`.bib` triple was built to force
each specific reallocation, run through the real `bibtex` binary installed on
this machine (`/Library/TeX/texbin/bibtex`, BibTeX 0.99e, TeX Live 2026), and
the resulting `.blg` line — including the previously-unverified `elt_size`
values — was read back and matched against this engine's output. Element
sizes are C `sizeof` values baked into that specific `bibtex` build; they are
not derived from source alone: `hash_ptr2`/`str_number`/`integer` measured as
4, `ASCII_code`/`boolean-as-char` as 1 (`lit_stk_type`'s `stk_type` measured
as 1 byte), `alpha_file` (the `bib_file` array) measured as 8 (a `FILE*`-sized
handle), `boolean` (the `entry_exists` array) measured as 4, and
`global_strs`' `elt_size` as `glob_str_size + 1` where `glob_str_size` is a
`texmf.cnf`-configurable value (this machine has it raised to 200000 via its
TeX Live config, matching `Options::default().glob_str_size`).

### Not emulated (documented limitation, not silently wrong)

Two reallocations are **not** implemented, and are not exercised by any
oracle case:

- **`str_pool`** ([53], `POOL_SIZE=65000`, grows by 65000): BibTeX's
  `str_pool` is one shared character array holding every permanent string
  plus the currently-live literal-stack strings, checked at every
  `str_room(n)` call site throughout `bibtex.web`. This engine represents
  strings as individually-allocated `Rc<[u8]>` (`Str` in `src/engine.rs`),
  with no equivalent shared-pool cursor to check against; emulating this
  precisely would mean tracking a synthetic `pool_ptr` across every string
  operation in `bib.rs`/`bst.rs`/`exec.rs`/`names.rs`, which is a
  significant redesign, not a local fix. Confirmed empirically (a 40-entry,
  300-byte-author `.bib` reliably triggers it against real `bibtex`,
  `elt_size=1`, 65000 → 130000) but not implemented here.
- **`buffer`/`sv_buffer`/`ex_buf`/`out_buf`/`name_tok`/`name_sep_char`**
  ([46]/[251], `BUF_SIZE=20000`, grows by 20000, plus the `copy_char`
  `"Field filled up at N, reallocating."` message): six buffers reallocated
  together whenever a single input line or field value exceeds 20000
  characters. Confirmed empirically (elt_size 1 for the byte buffers, 4 for
  `name_tok`) but this engine's buffers (`self.buffer`, `self.ex_buf`, etc.
  in `src/engine.rs`) already grow unconditionally via `Vec::resize` at
  several independent call sites (`input_ln`, `copy_to_ex_buf`, ...) without
  a shared capacity variable to check the TeX Live growth points against;
  wiring in the log line without a shared cursor risks emitting it at the
  wrong point or the wrong count. Left out rather than guessed.

A real `.bst`/`.bib` pair that hits either of these will still produce a
byte-correct `.bbl` (the underlying operation succeeds; this engine has no
fixed capacity to actually overflow) but a `.blg` missing these specific log
lines — a cosmetic diagnostics gap, not a correctness bug in the bibliography
output itself.

## Known edge cases, deliberately not emulated (carried over from r2)

- Hash-probe coincidences in §259, when `macro_name_loc` is not found.
- Uninitialized-memory reads on malformed `format.name$` input.

## Verified vs. believed, summary

**Verified** (measured against the installed `bibtex` binary, BibTeX 0.99e,
TeX Live 2026): `.bbl` byte-identical on 99 cases; `.blg` byte-identical on
99 cases including 4 new capacity-limit cases; all 6 reallocated-array types
listed in the table above, including their previously-unverified `elt_size`
values; `Session` cache-hit/rerun behavior (6 unit tests).

**Believed, not verified**: the adoption plan in this document (no compiler
code was changed to implement it); that the `.bbl`-parsing step it requires
is straightforward (it has not been prototyped); that other TeX Live
installations' `texmf.cnf` produce the same `elt_size`/`glob_str_size`
values this machine's does (the `Options` fields exist precisely so a caller
can override them, but the *defaults* here are this machine's, not a spec).

**Explicitly not attempted**: `str_pool` and buffer-group reallocation
logging (see above); mapping BibTeX's own diagnostic text back to `.tex`/
`.bib` byte spans (see "Diagnostics from `Output::warnings()`" above).
