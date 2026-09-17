# Kernel inventory audit (slice 1)

## Method

`crates/compiler/supported/canonical-latex.tsv` (header: `set\tkind\tname`, where `set` is the source package and `kind` is `command` or `environment`) was filtered to its 440 `kernel` rows (410 commands, 30 environments). For each name, `grep -rnwF -e 'NAME' crates/compiler/src/` (fixed-string, whole-word, recursive) was run from the repository root to test for any implementation reference, and `grep -rnwF -e 'NAME' crates/compiler/tests/` for integration-test coverage; inline `#[cfg(test)]` modules were separated from production code by treating every `#[cfg(test)] mod …` region (verified to run to end-of-file in each file) as test code. Short names that whole-word grep over-counts (single letters colliding with Rust identifiers, SI-unit symbols, roman numerals, column specifiers) and comment-only mentions were adjudicated by reading every match in context; matches confined to `vocabulary.rs` `KNOWN_UNIMPLEMENTED_*` lists, `//` comments, or unrelated tables (SI units, roman numerals, tabular alignment) were ruled *spurious*, not implementation. Cross-checks: `crates/compiler/src/supported.rs` (the inventory-as-data; `EXPANSION_COMMANDS` entries are executed by the `flashtex-tex-expansion` engine in `crates/tex-expansion/`, so they leave no string trace in `crates/compiler/src/`) and `crates/compiler/tests/supported_latex.rs::every_inventory_entry_compiles_without_an_unsupported_diagnostic` (run 2026-09-14: 1 passed, 0 failed — every inventory entry compiles without a "not supported" diagnostic in at least one context). Table 1A rows reproduce as empty output; Table 1B/2 rows give the discriminating evidence found.

## Revision (2026-09-15)

This audit was originally written at commit `06007472` (2026-09-14T08:39:56Z) and went stale within about a day: this repo merged roughly 247 commits from parallel agent lanes in the interim, several of which fixed or tested rows this audit had just flagged (`#320`/PR-labelled-"#319" for `\hrulefill`/`\dotfill`; `#428`/`#430` for all 41 Table 2 rows; separate slices for `\includeonly`, the kernel text accents, `\eqnarray`, and several `KNOWN_UNIMPLEMENTED_COMMANDS`-listed commands that turned out to already have real dispatch arms sitting ahead of that stale list in the match order).

**Every row was re-checked against `dbe3cac8` (current `main` as of this revision)** using the same method (Section "Method" above), not just the four rows an independent review had already caught. Net changes:

| | original | current (`dbe3cac8`) | resolved | moved | still valid |
| --- | --- | --- | --- | --- | --- |
| Table 1A (unimplemented) | 153 | 125 | 22 | 6 → Table 2 | 125 |
| Table 1B (falsely-flagged) | 61 | 44 | 16 | 1 → Table 2 | 44 |
| Table 2 (untested) | 41 | 7 | 41 | — | 0 carried over; 7 new |
| **unimplemented total (1A+1B)** | **214** | **169** | | | |

86 of the original 255 rows (about a third) changed status in roughly 24 hours. The four rows the independent review flagged (`\hrulefill`, `\dotfill`, `\includeonly` — all Table 1A → resolved; `\displaystyle` — Table 2 → resolved) are among the 79 fully resolved rows, not the whole story: 7 more Table 1A/1B rows are now implemented but still untested (moved into the new Table 2), and dozens of others besides the four named ones flipped.

**This document is a point-in-time snapshot, not a living one, and it cannot be hand-maintained at this repo's commit rate.** A grep-based check like this is entirely mechanical (see Method) and belongs in `crates/compiler/scripts/` as a script that runs the same greps against `canonical-latex.tsv` and prints current Table 1A/1B/2 membership on demand — not as a markdown file someone re-derives by hand after the fact. Recommendation: either (a) land a `kernel_inventory_audit.py`-style generator alongside `render_supported_latex.sh` and regenerate this doc from it, or (b) close this PR once its useful findings (the resolved rows already fixed, and the 7 new Table 2 gaps) are turned into tracked follow-up issues, since the document itself will be wrong again within days.

## Revision 2 (independent review, 2026-09-17)

The prior revision's own Method (engine primitive/prelude macro in `flashtex-tex-expansion` counts as implementation) was stated but not fully applied, and four canonical rows were checked but never actually entered into any table. Fixed:

- **9 Table 1A rows removed** (`closein`, `closeout`, `openin`, `openout`, `lineskip`, `lineskiplimit`, `topskip`, `pdfpageheight`, `pdfpagewidth`): the prior revision's own triage notes already found these have real `crates/tex-expansion/src/expand.rs` primitive/register entries — the same standard that correctly excludes `\day`/`\month`/`\year`/`\space` from both tables — but left them in Table 1A with a hedging "may be partially handled" note instead of actually removing them. Removed, applying the stated rule consistently.
- **`tabbing` removed from Table 1B**: it is genuinely implemented (`IMPLEMENTED_ENVIRONMENTS`, `vocabulary.rs:109`, and a real row/column-stop dispatch arm at `parser.rs:6056`/`6239`), not merely name-listed. The cited "explicit not-implemented" tests (`tests/recovery.rs:68-69`, `tests/diagnostic_codes.rs:65-66`) test `picture`, not `tabbing` — misattributed evidence, not a real finding.
- **`marginpar` removed from Table 2**: `parser.rs` already has a dedicated inline test, `marginpar_parses_to_a_margin_note_without_diagnostics` (`parser.rs:14277`), covering exactly this at the audited commit; the claim that both test greps came back empty was wrong.
- **`part`, `vbox`, `linespread` added to Table 1B**: all three are canonical `kernel` rows (`part`/`vbox` commands, confirmed in `canonical-latex.tsv`) that were checked during this revision but never entered into any table. All three have real `crates/compiler/src/` matches (so Table 1A's "zero matches" doesn't apply) that are not genuine implementation — see the rows below for the discriminating evidence.
- **`minipage` needs no row**: it has a real dispatch check (`environment == "minipage"`, `parser.rs:10796`) with genuine footnote-numbering behavior, and a real test exercising it (`tests/footnote_counters.rs:134`) — implemented and tested, like `\day`/`\month`/`\year`/`\space`. (It is also stale-listed in `vocabulary.rs`'s `KNOWN_UNIMPLEMENTED_ENVIRONMENTS`, which is a pre-existing inconsistency in that list, not an audit gap — the same shape as the `hss`/hard-coded-list staleness the original revision already noted for other names.)

Net effect on the counts below: Table 1A 125 → 116 (-9), Table 1B 44 → 46 (-1 `tabbing`, +3 `part`/`vbox`/`linespread`), Table 2 7 → 6 (-1 `marginpar`). Unimplemented total (1A+1B): 169 → 162.

## Table 1A — kernel names with ZERO matches in `crates/compiler/src/` (116)

| kind | name | reproducing grep (run from repo root) | result |
| ---- | ---- | ------------------------------------- | ------ |
| command | AtBeginDvi | `grep -rnwF -e 'AtBeginDvi' crates/compiler/src/` | (no output) |
| command | AtEndDvi | `grep -rnwF -e 'AtEndDvi' crates/compiler/src/` | (no output) |
| command | AtEndOfClass | `grep -rnwF -e 'AtEndOfClass' crates/compiler/src/` | (no output) |
| command | AtEndOfPackage | `grep -rnwF -e 'AtEndOfPackage' crates/compiler/src/` | (no output) |
| command | CheckCommand | `grep -rnwF -e 'CheckCommand' crates/compiler/src/` | (no output) |
| command | ClassError | `grep -rnwF -e 'ClassError' crates/compiler/src/` | (no output) |
| command | ClassInfo | `grep -rnwF -e 'ClassInfo' crates/compiler/src/` | (no output) |
| command | ClassWarning | `grep -rnwF -e 'ClassWarning' crates/compiler/src/` | (no output) |
| command | ClassWarningNoLine | `grep -rnwF -e 'ClassWarningNoLine' crates/compiler/src/` | (no output) |
| command | CurrentOption | `grep -rnwF -e 'CurrentOption' crates/compiler/src/` | (no output) |
| command | DeclareFontEncoding | `grep -rnwF -e 'DeclareFontEncoding' crates/compiler/src/` | (no output) |
| command | DeclareOption | `grep -rnwF -e 'DeclareOption' crates/compiler/src/` | (no output) |
| command | DeclareTextAccent | `grep -rnwF -e 'DeclareTextAccent' crates/compiler/src/` | (no output) |
| command | DeclareTextAccentDefault | `grep -rnwF -e 'DeclareTextAccentDefault' crates/compiler/src/` | (no output) |
| command | DeclareTextCommand | `grep -rnwF -e 'DeclareTextCommand' crates/compiler/src/` | (no output) |
| command | DeclareTextComposite | `grep -rnwF -e 'DeclareTextComposite' crates/compiler/src/` | (no output) |
| command | DeclareTextCompositeCommand | `grep -rnwF -e 'DeclareTextCompositeCommand' crates/compiler/src/` | (no output) |
| command | DeclareTextSymbol | `grep -rnwF -e 'DeclareTextSymbol' crates/compiler/src/` | (no output) |
| command | DeclareTextSymbolDefault | `grep -rnwF -e 'DeclareTextSymbolDefault' crates/compiler/src/` | (no output) |
| command | ExecuteOptions | `grep -rnwF -e 'ExecuteOptions' crates/compiler/src/` | (no output) |
| command | IfFileExists | `grep -rnwF -e 'IfFileExists' crates/compiler/src/` | (no output) |
| command | InputIfFileExists | `grep -rnwF -e 'InputIfFileExists' crates/compiler/src/` | (no output) |
| command | LastDeclaredEncoding | `grep -rnwF -e 'LastDeclaredEncoding' crates/compiler/src/` | (no output) |
| command | LoadClass | `grep -rnwF -e 'LoadClass' crates/compiler/src/` | (no output) |
| command | LoadClassWithOptions | `grep -rnwF -e 'LoadClassWithOptions' crates/compiler/src/` | (no output) |
| command | OptionNotUsed | `grep -rnwF -e 'OptionNotUsed' crates/compiler/src/` | (no output) |
| command | PackageError | `grep -rnwF -e 'PackageError' crates/compiler/src/` | (no output) |
| command | PackageInfo | `grep -rnwF -e 'PackageInfo' crates/compiler/src/` | (no output) |
| command | PackageWarning | `grep -rnwF -e 'PackageWarning' crates/compiler/src/` | (no output) |
| command | PackageWarningNoLine | `grep -rnwF -e 'PackageWarningNoLine' crates/compiler/src/` | (no output) |
| command | PassOptionsToClass | `grep -rnwF -e 'PassOptionsToClass' crates/compiler/src/` | (no output) |
| command | ProcessOptions | `grep -rnwF -e 'ProcessOptions' crates/compiler/src/` | (no output) |
| command | ProvideTextCommand | `grep -rnwF -e 'ProvideTextCommand' crates/compiler/src/` | (no output) |
| command | ProvideTextCommandDefault | `grep -rnwF -e 'ProvideTextCommandDefault' crates/compiler/src/` | (no output) |
| command | RequirePackageWithOptions | `grep -rnwF -e 'RequirePackageWithOptions' crates/compiler/src/` | (no output) |
| command | UseTextAccent | `grep -rnwF -e 'UseTextAccent' crates/compiler/src/` | (no output) |
| command | UseTextSymbol | `grep -rnwF -e 'UseTextSymbol' crates/compiler/src/` | (no output) |
| command | addtocontents | `grep -rnwF -e 'addtocontents' crates/compiler/src/` | (no output) |
| command | arraycolsep | `grep -rnwF -e 'arraycolsep' crates/compiler/src/` | (no output) |
| command | baselinestretch | `grep -rnwF -e 'baselinestretch' crates/compiler/src/` | (no output) |
| command | bigbreak | `grep -rnwF -e 'bigbreak' crates/compiler/src/` | (no output) |
| command | bottomfraction | `grep -rnwF -e 'bottomfraction' crates/compiler/src/` | (no output) |
| command | capitalacute | `grep -rnwF -e 'capitalacute' crates/compiler/src/` | (no output) |
| command | capitalcircumflex | `grep -rnwF -e 'capitalcircumflex' crates/compiler/src/` | (no output) |
| command | capitaldieresis | `grep -rnwF -e 'capitaldieresis' crates/compiler/src/` | (no output) |
| command | capitaldotaccent | `grep -rnwF -e 'capitaldotaccent' crates/compiler/src/` | (no output) |
| command | capitalgrave | `grep -rnwF -e 'capitalgrave' crates/compiler/src/` | (no output) |
| command | capitalmacron | `grep -rnwF -e 'capitalmacron' crates/compiler/src/` | (no output) |
| command | capitalnewtie | `grep -rnwF -e 'capitalnewtie' crates/compiler/src/` | (no output) |
| command | capitaltie | `grep -rnwF -e 'capitaltie' crates/compiler/src/` | (no output) |
| command | capitaltilde | `grep -rnwF -e 'capitaltilde' crates/compiler/src/` | (no output) |
| command | circle | `grep -rnwF -e 'circle' crates/compiler/src/` | (no output) |
| command | columnseprule | `grep -rnwF -e 'columnseprule' crates/compiler/src/` | (no output) |
| command | contentsline | `grep -rnwF -e 'contentsline' crates/compiler/src/` | (no output) |
| command | dashbox | `grep -rnwF -e 'dashbox' crates/compiler/src/` | (no output) |
| command | floatpagefraction | `grep -rnwF -e 'floatpagefraction' crates/compiler/src/` | (no output) |
| command | floatsep | `grep -rnwF -e 'floatsep' crates/compiler/src/` | (no output) |
| command | fontshape | `grep -rnwF -e 'fontshape' crates/compiler/src/` | (no output) |
| command | frenchspacing | `grep -rnwF -e 'frenchspacing' crates/compiler/src/` | (no output) |
| command | ignorespacesafterend | `grep -rnwF -e 'ignorespacesafterend' crates/compiler/src/` | (no output) |
| command | indexspace | `grep -rnwF -e 'indexspace' crates/compiler/src/` | (no output) |
| command | intextsep | `grep -rnwF -e 'intextsep' crates/compiler/src/` | (no output) |
| command | labelitemii | `grep -rnwF -e 'labelitemii' crates/compiler/src/` | (no output) |
| command | labelitemiii | `grep -rnwF -e 'labelitemiii' crates/compiler/src/` | (no output) |
| command | labelitemiv | `grep -rnwF -e 'labelitemiv' crates/compiler/src/` | (no output) |
| command | lefteqn | `grep -rnwF -e 'lefteqn' crates/compiler/src/` | (no output) |
| command | leftmarginii | `grep -rnwF -e 'leftmarginii' crates/compiler/src/` | (no output) |
| command | leftmarginiii | `grep -rnwF -e 'leftmarginiii' crates/compiler/src/` | (no output) |
| command | leftmarginv | `grep -rnwF -e 'leftmarginv' crates/compiler/src/` | (no output) |
| command | leftmarginvi | `grep -rnwF -e 'leftmarginvi' crates/compiler/src/` | (no output) |
| command | linethickness | `grep -rnwF -e 'linethickness' crates/compiler/src/` | (no output) |
| command | makeglossary | `grep -rnwF -e 'makeglossary' crates/compiler/src/` | (no output) |
| command | makeindex | `grep -rnwF -e 'makeindex' crates/compiler/src/` | (no output) |
| command | marginparpush | `grep -rnwF -e 'marginparpush' crates/compiler/src/` | (no output) |
| command | mathversion | `grep -rnwF -e 'mathversion' crates/compiler/src/` | (no output) |
| command | medbreak | `grep -rnwF -e 'medbreak' crates/compiler/src/` | (no output) |
| command | multiput | `grep -rnwF -e 'multiput' crates/compiler/src/` | (no output) |
| command | newfont | `grep -rnwF -e 'newfont' crates/compiler/src/` | (no output) |
| command | newsavebox | `grep -rnwF -e 'newsavebox' crates/compiler/src/` | (no output) |
| command | newtie | `grep -rnwF -e 'newtie' crates/compiler/src/` | (no output) |
| command | newwrite | `grep -rnwF -e 'newwrite' crates/compiler/src/` | (no output) |
| command | nocorrlist | `grep -rnwF -e 'nocorrlist' crates/compiler/src/` | (no output) |
| command | nofiles | `grep -rnwF -e 'nofiles' crates/compiler/src/` | (no output) |
| command | nonfrenchspacing | `grep -rnwF -e 'nonfrenchspacing' crates/compiler/src/` | (no output) |
| command | normalmarginpar | `grep -rnwF -e 'normalmarginpar' crates/compiler/src/` | (no output) |
| command | normalsfcodes | `grep -rnwF -e 'normalsfcodes' crates/compiler/src/` | (no output) |
| command | obeycr | `grep -rnwF -e 'obeycr' crates/compiler/src/` | (no output) |
| command | oldstylenums | `grep -rnwF -e 'oldstylenums' crates/compiler/src/` | (no output) |
| command | onecolumn | `grep -rnwF -e 'onecolumn' crates/compiler/src/` | (no output) |
| command | oval | `grep -rnwF -e 'oval' crates/compiler/src/` | (no output) |
| command | poptabs | `grep -rnwF -e 'poptabs' crates/compiler/src/` | (no output) |
| command | prevdepth | `grep -rnwF -e 'prevdepth' crates/compiler/src/` | (no output) |
| command | qbezier | `grep -rnwF -e 'qbezier' crates/compiler/src/` | (no output) |
| command | restorecr | `grep -rnwF -e 'restorecr' crates/compiler/src/` | (no output) |
| command | reversemarginpar | `grep -rnwF -e 'reversemarginpar' crates/compiler/src/` | (no output) |
| command | savebox | `grep -rnwF -e 'savebox' crates/compiler/src/` | (no output) |
| command | shortstack | `grep -rnwF -e 'shortstack' crates/compiler/src/` | (no output) |
| command | smallbreak | `grep -rnwF -e 'smallbreak' crates/compiler/src/` | (no output) |
| command | spacefactor | `grep -rnwF -e 'spacefactor' crates/compiler/src/` | (no output) |
| command | subitem | `grep -rnwF -e 'subitem' crates/compiler/src/` | (no output) |
| command | subsubitem | `grep -rnwF -e 'subsubitem' crates/compiler/src/` | (no output) |
| command | suppressfloats | `grep -rnwF -e 'suppressfloats' crates/compiler/src/` | (no output) |
| command | textfloatsep | `grep -rnwF -e 'textfloatsep' crates/compiler/src/` | (no output) |
| command | textfraction | `grep -rnwF -e 'textfraction' crates/compiler/src/` | (no output) |
| command | thicklines | `grep -rnwF -e 'thicklines' crates/compiler/src/` | (no output) |
| command | thinlines | `grep -rnwF -e 'thinlines' crates/compiler/src/` | (no output) |
| command | topfraction | `grep -rnwF -e 'topfraction' crates/compiler/src/` | (no output) |
| command | typein | `grep -rnwF -e 'typein' crates/compiler/src/` | (no output) |
| command | typeout | `grep -rnwF -e 'typeout' crates/compiler/src/` | (no output) |
| command | unboldmath | `grep -rnwF -e 'unboldmath' crates/compiler/src/` | (no output) |
| command | unitlength | `grep -rnwF -e 'unitlength' crates/compiler/src/` | (no output) |
| command | usebox | `grep -rnwF -e 'usebox' crates/compiler/src/` | (no output) |
| command | usecounter | `grep -rnwF -e 'usecounter' crates/compiler/src/` | (no output) |
| command | wlog | `grep -rnwF -e 'wlog' crates/compiler/src/` | (no output) |
| environment | filecontents* | `grep -rnwF -e 'filecontents*' crates/compiler/src/` | (no output) |
| environment | theindex | `grep -rnwF -e 'theindex' crates/compiler/src/` | (no output) |

## Table 1B — names with matches but NO genuine implementation (46)

The word grep below returns hits, but every hit was read in context and is spurious: `//` comments, `vocabulary.rs` `KNOWN_UNIMPLEMENTED_*` entries (the compiler's own not-implemented list — itself stale in places, see the revision note above: several names it lists are dispatched *before* the fallback that would ever consult it), SI-unit symbol tables (`siunitx.rs`), the `roman()` numeral table (`parser.rs:5775`), tabular column-alignment words, or unrelated Rust identifiers. The "genuine-impl check" column gives the discriminating grep (empty output) proving no dispatch arm, builtin-table entry, or inventory claim exists.

| kind | name | word grep (run from repo root) | genuine-impl check (empty output) and what the word hits are |
| ---- | ---- | ------------------------------ | ------------------------------------------------------------ |
| command | t | `grep -rnwF -e 't' crates/compiler/src/` | `grep -rn -e '"t"' crates/compiler/src/parser.rs crates/compiler/src/text_builtins.rs` empty exc. test-region lines; hits are SI `tonne`, `VerticalPosition::Top`, identifiers |
| command | put | `grep -rnwF -e 'put' crates/compiler/src/` | `grep -rn -e '"put"' crates/compiler/src/parser.rs crates/compiler/src/math.rs crates/compiler/src/expansion.rs` empty; hits are a `text_builtins.rs` logo closure variable and `//` comments |
| command | hss | `grep -rnwF -e 'hss' crates/compiler/src/` | `grep -rn -e '"hss"' crates/compiler/src/parser.rs crates/compiler/src/math.rs` empty; hits are a `//` comment (`parser/lists.rs:15`) and `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:43`) |
| command | DeclareTextCommandDefault | `grep -rnwF -e 'DeclareTextCommandDefault' crates/compiler/src/` | `grep -rn -e 'DeclareTextCommandDefault' crates/compiler/src/ --include='*.rs' | grep -v '^.*://'` empty; sole hit is a `//` comment (`text_builtins.rs:129`) |
| command | addcontentsline | `grep -rnwF -e 'addcontentsline' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`layout.rs:88`) |
| command | bigskipamount | `grep -rnwF -e 'bigskipamount' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`parser.rs:887`) |
| command | boldmath | `grep -rnwF -e 'boldmath' crates/compiler/src/` | non-comment grep empty; hits are `//` comments (`text_builtins.rs:262,367`) |
| command | fontencoding | `grep -rnwF -e 'fontencoding' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`text_builtins.rs:150`) |
| command | fontseries | `grep -rnwF -e 'fontseries' crates/compiler/src/` | non-comment grep empty; hits are `//` comments (`parser.rs:3270,3339`) |
| command | labelenumi | `grep -rnwF -e 'labelenumi' crates/compiler/src/` | non-comment grep empty; sole hit is a `//!` doc comment (`parser/lists.rs:17`) |
| command | labelenumii | `grep -rnwF -e 'labelenumii' crates/compiler/src/` | non-comment grep empty; sole hit is a `//!` doc comment (`parser/lists.rs:17`) |
| command | labelenumiii | `grep -rnwF -e 'labelenumiii' crates/compiler/src/` | non-comment grep empty; sole hit is a `//!` doc comment (`parser/lists.rs:18`) |
| command | labelenumiv | `grep -rnwF -e 'labelenumiv' crates/compiler/src/` | non-comment grep empty; sole hit is a `//!` doc comment (`parser/lists.rs:18`) |
| command | labelitemi | `grep -rnwF -e 'labelitemi' crates/compiler/src/` | non-comment grep empty; sole hit is a `//!` doc comment (`parser/lists.rs:19`) |
| command | leftmargini | `grep -rnwF -e 'leftmargini' crates/compiler/src/` | non-comment grep empty; hits are `///` doc comments (`layout.rs:46,48`) |
| command | leftmarginiv | `grep -rnwF -e 'leftmarginiv' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`layout.rs:48`) |
| command | makelabel | `grep -rnwF -e 'makelabel' crates/compiler/src/` | non-comment grep empty; hits are `//`/`//!` comments (`layout.rs:1006`, `parser/lists.rs:15,22`) |
| command | medskipamount | `grep -rnwF -e 'medskipamount' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`parser.rs:886`) |
| command | nobreakspace | `grep -rnwF -e 'nobreakspace' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`parser.rs:6014`) |
| command | numberline | `grep -rnwF -e 'numberline' crates/compiler/src/` | non-comment grep empty; hits are `///` doc comments (`layout.rs:75,1030`) |
| command | refname | `grep -rnwF -e 'refname' crates/compiler/src/` | non-comment grep empty; sole hit is a `//` comment (`parser.rs:2958`) |
| command | sbox | `grep -rnwF -e 'sbox' crates/compiler/src/` | non-comment grep empty; sole hit is a `//` comment (`text_builtins.rs:338`) |
| command | shipout | `grep -rnwF -e 'shipout' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`color.rs:865`); no engine entry either |
| command | smallskipamount | `grep -rnwF -e 'smallskipamount' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`parser.rs:886`) |
| command | vector | `grep -rnwF -e 'vector' crates/compiler/src/` | non-comment grep empty; sole hit is a `///` doc comment (`math.rs:3334`, English word "vector") |
| command | vtop | `grep -rnwF -e 'vtop' crates/compiler/src/` | non-comment grep empty; sole hit is a `//!` doc comment (`tabular.rs:27`); no engine entry either |
| command | appendix | `grep -rnwF -e 'appendix' crates/compiler/src/` | non-comment grep empty exc. `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:38`); no dispatch, no engine entry |
| command | fbox | `grep -rnwF -e 'fbox' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:44`); no dispatch, no engine entry |
| command | fontfamily | `grep -rnwF -e 'fontfamily' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:55`); no dispatch, no engine entry |
| command | fontsize | `grep -rnwF -e 'fontsize' crates/compiler/src/` | hits are a `//` comment (`text_builtins.rs:339`) and `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:54`) |
| command | framebox | `grep -rnwF -e 'framebox' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:44`); no dispatch, no engine entry |
| command | listoffigures | `grep -rnwF -e 'listoffigures' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:39`); no dispatch, no engine entry |
| command | listoftables | `grep -rnwF -e 'listoftables' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:40`); no dispatch, no engine entry |
| command | makebox | `grep -rnwF -e 'makebox' crates/compiler/src/` | hits are `///` comments (`tabular.rs:235,249`) and `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:44`) |
| command | parbox | `grep -rnwF -e 'parbox' crates/compiler/src/` | hits are a `///` comment (`tabular.rs:329`) and `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:44`); `tests/` hits are incidental corpus text, not `\parbox` tests |
| command | raisebox | `grep -rnwF -e 'raisebox' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:44`); no dispatch, no engine entry |
| command | RequirePackage | `grep -rnwF -e 'RequirePackage' crates/compiler/src/` | hits are `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:62`) and `//`/`///` comments (`math.rs:488,547,553`); no parser arm, no engine entry — `\RequirePackage{…}` falls through to `unsupported()` |
| command | PassOptionsToPackage | `grep -rnwF -e 'PassOptionsToPackage' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:63`); no dispatch, no engine entry |
| command | selectfont | `grep -rnwF -e 'selectfont' crates/compiler/src/` | hits are a `//` comment (`text_builtins.rs:339`) and `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:54`) |
| command | usefont | `grep -rnwF -e 'usefont' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:55`); no dispatch, no engine entry |
| environment | abstract | `grep -rnwF -e 'abstract' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_ENVIRONMENTS` (`vocabulary.rs:110`); no dispatch, no engine entry |
| environment | filecontents | `grep -rnwF -e 'filecontents' crates/compiler/src/` | only hit is the unimplemented-environment list (`vocabulary.rs:113`); no dispatch, no engine entry |
| environment | picture | `grep -rnwF -e 'picture' crates/compiler/src/` | only hit is `KNOWN_UNIMPLEMENTED_ENVIRONMENTS` (`vocabulary.rs:111`); no dispatch, no engine entry |
| command | part | `grep -rnwF -e 'part' crates/compiler/src/` | hits are `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:40`), `xref.rs:61,153`'s counter-name-formatting table (applies to any counter named "part", not a `\part` dispatch arm), and unrelated uses of the English word "part" (`tabular.rs`, `parser/lists.rs`); no sectioning dispatch arm exists |
| command | vbox | `grep -rnwF -e 'vbox' crates/compiler/src/` | hits are `KNOWN_UNIMPLEMENTED_COMMANDS` (`vocabulary.rs:45`) and `//`/`///` comments describing other constructs (`tabular.rs`, `text_builtins.rs`, `math.rs`) in terms of what real `\vbox` would do; no dispatch arm |
| command | linespread | `grep -rnwF -e 'linespread' crates/compiler/src/` | only hit is the `KNOWN_ARITY_UNIMPLEMENTED` table (`parser.rs:9863-9865`): the argument is deliberately skipped and a diagnostic is emitted (tested, `parser.rs:13121-13132`), but line spacing itself never changes — recognised-and-diagnosed, not implemented |

## Table 2 — implemented but with ZERO name matches in tests/ or inline test modules (6)

"Implemented" means a genuine dispatch arm, builtin-table entry, or `flashtex-tex-expansion` engine primitive/prelude macro (site noted per row). Both test greps below return no output for every row: `grep -rnwF -e 'NAME' crates/compiler/tests/` and the inline check (matches of the same word grep inside `#[cfg(test)] mod …` regions of `src/`). Caveat: `tests/supported_latex.rs::every_inventory_entry_compiles_without_an_unsupported_diagnostic` compiles every *inventory-listed* name generically (verified passing), so inventory-listed rows below do have one-context smoke coverage — the same coverage `\pagestyle` had when its preamble bug shipped. Name-specific tests (especially second-context tests: preamble vs body, math vs text) are what is missing. All 41 rows from the original slice-1 Table 2 now have dedicated name-specific tests (`crates/compiler/tests/kernel_untested_a.rs` + `kernel_untested_b.rs`, #428/#430) and are removed from this table; the 6 rows below are newly-discovered gaps of the same shape, found while re-deriving this audit against current main (`marginpar` was also flagged here in error — Revision 2 above — and is removed).

| kind | name | implementation evidence | test greps (both empty) |
| ---- | ---- | ----------------------- | ----------------------- |
| command | paperheight | `PREAMBLE_LENGTHS` dimen list (`src/parser.rs:1485`) + preamble-assignment dispatch guard `is_preamble_length` (`src/parser.rs:1512`, arm at `src/parser.rs:3126`) | `grep -rnwF -e 'paperheight' crates/compiler/tests/` ; inline empty |
| command | evensidemargin | `PREAMBLE_LENGTHS` dimen list (`src/parser.rs:1489`) + same `is_preamble_length` dispatch as `paperheight` | `grep -rnwF -e 'evensidemargin' crates/compiler/tests/` ; inline empty |
| command | headheight | `PREAMBLE_LENGTHS` dimen list (`src/parser.rs:1491`) + same `is_preamble_length` dispatch as `paperheight` | `grep -rnwF -e 'headheight' crates/compiler/tests/` ; inline empty |
| command | footskip | `PREAMBLE_LENGTHS` dimen list (`src/parser.rs:1493`) + same `is_preamble_length` dispatch as `paperheight` | `grep -rnwF -e 'footskip' crates/compiler/tests/` ; inline empty |
| command | marginparwidth | `PREAMBLE_LENGTHS` dimen list (`src/parser.rs:1494`) + same `is_preamble_length` dispatch as `paperheight` | `grep -rnwF -e 'marginparwidth' crates/compiler/tests/` ; inline empty |
| command | columnsep | `PREAMBLE_LENGTHS` dimen list (`src/parser.rs:1496`) + same `is_preamble_length` dispatch as `paperheight` | `grep -rnwF -e 'columnsep' crates/compiler/tests/` ; inline empty |

## Triage notes for the supervisor

- `\day`, `\month`, `\year`, `\space` (kernel rows with `src/` matches) are engine-implemented (`\day` etc. are `Count` registers at `expand.rs:4998`, `\space` is prelude-defined), so they appear in neither table — same treatment as `closein`/`closeout`/`openin`/`openout`/`lineskip`/`lineskiplimit`/`topskip`/`pdfpageheight`/`pdfpagewidth` now get (Revision 2 above): confirmed engine-side, so excluded from Table 1A rather than left in it with a caveat.
- Highest-value follow-ups as of this revision: 8 of the original 9 Table 1B text accents (`\b \c \d \H \k \r \u \v`) are now implemented and tested (`TEXT_ACCENTS`, `crates/compiler/tests/text_accents.rs`); `\t` (the two-letter tie accent) is still unimplemented and is the last of the nine. `\RequirePackage` and `\PassOptionsToPackage` are still unimplemented (real documents use `\RequirePackage`; both still fall to `unsupported()`). The 6 Table 2 preamble-length registers above are the new highest-value test gap.
