# Independent remaining HW1 logical mapping audit

Rechecked immutable HW1 source and the actual combined response at `97a15831`. Source occurrence and remaining diagnostic counts agree: `\mid` four, `\vee` four, `\Rightarrow` three. `independent-pins.json` retains exact byte ranges/output hashes. Root's `0abadb9f` handoff reaches the same conclusion; this is independent evidence, not a duplicate implementation lane.

| Command | Unicode | Safe Adobe Symbol export | Existing downstream class / CM Symbol slot | Pinned LM Math original GID |
| --- | --- | --- | --- | --- |
| `\vee` | U+2228 | 0xDA, logicalor (AFM C218) | Bin / 0x5F | 2770, logicalor |
| `\Rightarrow` | U+21D2 | 0xDE, arrowdblright (AFM C222) | Rel / 0x29 | 2100, arrowdblright |
| `\mid` | U+2223 | **No mapping in checked Adobe table** | Rel / 0x6A | 2670, divides |

The [Unicode-hosted Adobe Symbol table](https://www.unicode.org/Public/MAPPINGS/VENDORS/ADOBE/symbol.txt) explicitly maps U+2228/DA and U+21D2/DE; local StandardSymbolsPS.afm corroborates the same encoded names. U+2223 is absent. Its available OpenType glyph and CM slot do not create a supported direct compiler PDF encoding. Adobe `bar` at0x7C represents U+007C, which is not U+2223; do not silently substitute it or misclassify the relation as an ordinary ASCII character.

Recommend only the existing-table `vee`/`Rightarrow` pair for the next bounded candidate. Current compiler recognition and direct export mappings need those two additions; existing single-scalar AST, class selection, CM metrics and actual cmap lookup already provide the downstream chain. Preserve scripts and original command spans, keep `Longrightarrow` distinct, and do not invent a long-arrow assembly or source precision. Pinned original GIDs are asset evidence, never hardcoded encoding constants.

`mid` remains a direct-export mapping blocker until an owner approves explicit representation semantics that preserve the correct scalar and relation spacing. Existing unknown-command diagnostics should remain instead of falsely claiming complete support. Whole-expression pipeline source provenance and unrelated mathbb/spacing/environment/optical-profile gaps remain unchanged. No compiler build, new parser, native run or corpus sweep was performed; cmap lookups used the installed reader and existing immutable LM Math bytes.
