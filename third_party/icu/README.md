# `third_party/icu`

The part of TeX Live 2026's ICU that XeTeX's bidirectional layout runs,
**unmodified**: the source of record for the Rust port in
`crates/flashtex-xetex/src/native/ubidi/` (`docs/design/xetex/PLAN.md` §3.1).
XeTeX's `measure_native_node` (`XeTeX_ext.c`) splits each native word into
direction runs with ICU's `ubidi_setPara`, `ubidi_getDirection`,
`ubidi_countRuns` and `ubidi_getVisualRun`, the paragraph level coming from
the font's script (`getDefaultDirection`, `XeTeXLayoutInterface.cpp`), and
shapes each run with HarfBuzz in its direction. Nothing here is compiled:
the port follows these files routine by routine, and its property data
(`props_data.rs`) is generated from `ubidi_props_data.h` by
`gen_props_data.py` and checked against it by the crate's tests.

## Pin

| | |
|---|---|
| Upstream | TeX Live source tree, GitHub mirror <https://github.com/TeX-Live/texlive-source> |
| Tag | `tags/texlive-2026.1` ("texlive-2026.1 tag based on r78399") |
| Commit | `6a300188053b8f2ded89dbd52293732a706b9c0e` (the same pin as `third_party/xetex`, `third_party/harfbuzz`, `third_party/teckit`) |
| Path | `libs/icu/icu-src/` and `libs/icu/version.ac` |
| Version | 78.2 (`libs/icu/version.ac`; `U_ICU_VERSION` in `source/common/unicode/uvernum.h`), Unicode 17 bidi data (`ubidi_props_dataVersion` 0x11) |
| Fetched | 2026-10-12 |

`crates/flashtex-xetex/tests/pinned_libs.rs` fails if this table pins
another version, or if any file differs from `SHA256SUMS` or is missing from
it.

## Files

`icu-src/source/common/`: `ubidi.cpp` (`ubidi_setPara` and the algorithm:
paragraph levels P2–P3, explicit levels X1–X10, paired brackets N0, weak and
neutral types W1–N2 by state tables, implicit levels I1–I2, L1),
`ubidiln.cpp` (the runs and their visual order, L2), `ubidiimp.h`,
`ubidi_props.cpp`, `ubidi_props.h` and the generated `ubidi_props_data.h`
(the bidi class, paired-bracket and mirroring properties as a `UTrie2`),
`utrie2.h` (the trie lookup, `UTRIE2_GET16`), `unicode/ubidi.h` (the API and
its constants) and `unicode/uvernum.h` (the version). `icu-src/LICENSE` is
ICU's licence (the Unicode License V3, and the ICU licence for code from
before 2016); `version.ac` is TeX Live's version record. 11 files, 444 KB:

```sh
cd third_party/icu && shasum -a 256 -c SHA256SUMS
```

Left out: the rest of ICU, which XeTeX links but whose other services
(converters, line breaking, normalization) this tree does not port.

## Licence

ICU is under the Unicode License V3 (`icu-src/LICENSE`); the port in
`crates/flashtex-xetex` reproduces its notices in that crate's `LICENSE`, as
the licence requires. It is a permissive licence and may be combined with
the crate's GPL-2.0-or-later.
