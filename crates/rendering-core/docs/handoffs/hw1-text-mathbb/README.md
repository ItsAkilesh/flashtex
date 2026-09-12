# HW1 text and double-struck alphabet boundary

The immutable source has five `\text` arguments: `(a)`, `(b)`, `(c)`, `(d)`, `and`. Four `\mathbb` definitions introduce N/Z/Q/R aliases; N is unused, Z appears twice, Q twice and R seven times (eleven actual expansions). `source-pins.json` records the literal source inventory; the bounded regex inventory is not a TeX parser or execution.

Neither construct is a safe single-symbol command-table addition. Current compiler math AST has only Symbol/Fraction/Radical. `required_group` parses a math list, whose loop discards spaces and flattens nested braces. Treating `\text{and}` as ordinary symbols would italicize its letters and erase text-mode semantics; treating `\mathbb{R}` as R would silently lose the requested alphabet.

## Recommended next scope: explicit bounded literal text nucleus

The existing compiler/math owner can add a Text nucleus using its existing token stream and bounded balanced-group machinery, preserving literal argument text and original source range. Update the compiler's own layout/export treatment and the producer's exhaustive `convert_math` adapter coherently; do not add a second parser. Existing math-layout already has `Nucleus::Text(String)` and Roman `text_glyph` metrics. Adapt it as **AtomClass::Ord**, not `text_op` (which creates an operator atom with different spacing). Preserve scripts and source spans through the group boundary.

Initially scope acceptance to the actual simple literal arguments above and equivalent explicitly supported ASCII forms. Existing `make_text` walks characters with no general text shaping, ligature/kern processing or text-space glue, and `text_glyph` indexes Roman metrics by character code. It is not full amsmath `\text`. Do not silently admit nested formatting, arbitrary Unicode, text spaces or ligatures without a matching text-layout/resource path. A bounded collector must preserve those tokens for diagnostics or reject unsupported forms; it must not quietly drop them via math parsing. Missing/unterminated groups and nesting/work limits need existing recovery behavior. The supported subset and refusals must be explicit.

Acceptance should verify upright Roman selection, Ord spacing, exact visible text, script behavior, literal/group source preservation, malformed/unsupported arguments and incremental/clean equality. Reuse the five immutable source occurrences; broad array/environment layout and native oracle remain the existing corpus owner's work. Compiler per-argument span precision does not automatically change the producer's current whole-math provenance. No visual parity or complete HW1 claim follows.

## Why mathbb needs a separate representation task

Double-struck alphabet selection requires a structured alphabet argument, not a command alias. Existing compiler AST has no style-bearing math nucleus; the default single-character adapter uses ordinary symbol classification/CM metric mapping and math italic conversion for Latin letters. Neither substituting plain N/Z/Q/R nor mapping to unrelated Symbol black-letter glyphs preserves double-struck semantics. An OpenType font's available Unicode double-struck glyph alone does not provide its intended TeX alphabet metrics or a direct compiler PDF export encoding.

The owner must choose and bind a supported alphabet/resource/metric/export chain, preserving original GIDs, script sizing and source argument provenance. The direct base14/Symbol export route lacks the corresponding general alphabet representation. Leave `\mathbb` diagnosed until that chain exists; do not fake it to reduce the eleven warnings/errors.

This handoff is source/representation review only at producer `9aaec57a`; no code, font substitutions, compiler/native build or corpus replay was performed. Root's logical-symbol candidate remains independent and should finish first.
