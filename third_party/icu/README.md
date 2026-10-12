# `third_party/icu`

TeX Live 2026's ICU line-break rules, **unmodified**: the compiled
rule-based break iterator tables (`.brk`) that `ubrk_open(UBRK_LINE, locale)`
loads, for `\XeTeXlinebreaklocale` (XeTeX_ext.c's `linebreakstart`,
`crates/flashtex-xetex/src/native/linebreak.rs`, which runs them as ICU's
`RuleBasedBreakIterator::handleNext` does). Only these tables are vendored:
ICU's code is not, and the Southeast Asian dictionaries are ICU4X's
(`icu_segmenter`, see linebreak.rs).

## Pin

| | |
|---|---|
| Upstream | TeX Live source tree, GitHub mirror <https://github.com/TeX-Live/texlive-source> |
| Tag | `tags/texlive-2026.1` |
| Commit | `6a300188053b8f2ded89dbd52293732a706b9c0e` (the same pin as `third_party/xetex`, `third_party/harfbuzz`, `third_party/teckit`) |
| Path | `libs/icu/icu-src/source/data/in/icudt78l.dat` (sha256 `d7da8e2e31312a3cb574d1a0ebd7b187daa988e15a9b70b78eb68b213a590608`), items `brkitr/line*.brk` |
| Version | 78.2 (`libs/icu/version.ac`; `xetex --version`: "Compiled with ICU version 78.2") |
| Fetched | 2026-10-12 |

Extracted with ICU's `icupkg -x brkitr/line.brk -d out icudt78l.dat` (and
likewise each file); the six files are byte-identical to the same items of
ICU's own `icu4c-78.2-data-bin-l.zip`. TeX Live's data holds no LSTM models
(`icupkg -l` lists none), so ICU breaks Thai, Lao, Khmer and Myanmar by
dictionary.

| file | ICU rules (`icu4c/source/data/brkitr/rules/`) | used for |
|---|---|---|
| `line.brk` | `line.txt` (CSS strict) | every locale but `zh`, `ja`, `ko` |
| `line_cj.brk` | `line_cj.txt` | `zh`; `ja`/`ko` with `@lb=strict` |
| `line_normal.brk` | `line_normal.txt` | `ja`, `ko`; others with `@lb=normal` |
| `line_normal_cj.brk` | `line_normal_cj.txt` | `zh`, `ja`, `ko` with `@lb=normal` |
| `line_loose.brk` | `line_loose.txt` | others with `@lb=loose` |
| `line_loose_cj.brk` | `line_loose_cj.txt` | `zh`, `ja`, `ko` with `@lb=loose` |

`crates/flashtex-xetex/tests/pinned_libs.rs` fails if a file differs from
`SHA256SUMS`, a file of `brkitr/` is missing from it, or this table pins
another version. Licence: Unicode License v3 (`LICENSE`, ICU 78.2's).
