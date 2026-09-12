# HW1 bounded quantifier candidate

A combined `\forall` / `\exists` recognition and export-table change is compatible with the existing Symbol AST. It need not add a parser, new math class or special quantifier layout. The pinned immutable HW1 contains ten universal and six existential command tokens; exact original byte ranges are in `source-font-pins.json`.

| Command | Unicode | Adobe Symbol byte | Existing CM Symbol slot | Existing math class | Pinned LM Math cmap evidence |
| --- | --- | --- | --- | --- | --- |
| `\forall` | U+2200 | 0x22 (decimal34), universal | 0x38 | Ord | original GID2782, universal |
| `\exists` | U+2203 | 0x24 (decimal36), existential | 0x39 | Ord | original GID2784, existential |

Adobe bytes follow the [Unicode-hosted Adobe Symbol table](https://www.unicode.org/Public/MAPPINGS/VENDORS/ADOBE/symbol.txt); the existing local `/usr/share/fonts/urw-base35/StandardSymbolsPS.afm` independently has C34/universal and C36/existential. These encoding bytes are not CM metric slots or OpenType GIDs. GIDs above were read using the already installed fontTools cmap reader from immutable LM Math SHA `6075562b771f8b82f0c179e363389684f2dd09de30038269e2628e504bd7be0f`; they are evidence for this asset only, never constants for implementation. No new font parser or font download was used.

At producer `9aaec57a`, compiler COMMAND_GLYPHS and direct Symbol export mapping lack both commands. Existing `command_atom` can return a one-scalar `Nucleus::Symbol` while preserving its command span and normal script attachments. Existing `typeset::convert_math` then calls `Atom::symbol`; `default_class` yields Ord, `cm::symbol_slot` supplies the distinct slots above, and `TexMathMetrics::otf_gid` resolves the actual immutable face cmap. `MathFonts::math_char` leaves these two scalars unchanged. Missing resources/GIDs keep current diagnostics; do not substitute or cast a slot.

Keep `\exists!` as the existing quantifier token followed by the separate exclamation symbol. The patch must not consume `!`, alter user macro expansion, classify quantifiers as operators/relations, or synthesize limits. Tests should assert inline/display and macro recognition, scripts retained, exact direct-export byte mapping, unknown-command diagnostics explicitly nonempty, and incremental/clean equality. Existing helper/compiler ownership and the combined starred/membership baseline remain authoritative.

Compiler v1 command spans can remain exact original tokens (seven bytes for `\forall`, seven for `\exists`); Unicode output is three UTF-8 bytes. The pipeline currently retains whole-math-expression provenance for emitted clusters, as documented in the membership handoff. Do not promise new token-level navigation or infer spans from glyph/text lengths. Existing array, mathbb, spacing and optical-design diagnostics remain; these sixteen recognized tokens do not imply HW1 reference parity.

This review performed only source/table inspection and two cmap lookups. No candidate implementation, compiler run, corpus replay or native execution occurred. Root owns the next candidate and its focused tests.
