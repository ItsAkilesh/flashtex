# Text AST producer adapter candidate

Root fixed the isolated compiler type to `Nucleus::Text(String)`, with the outer `MathAtom.span` covering command through group end and no inner span fields. Compiler `MathItem` gains optional explicit font intent; Text uses TimesRoman on the direct v1 path, existing symbols/rules retain prior selection. The candidate compiler file hash at review is pinned, not claimed as an adopted compiler revision.

`adapter.patch` targets exact producer `9aaec57a` in two files and three exhaustive-match sites:

- `incremental::hash_math`: distinct tag3 plus exact Text content. It must never collide semantically with Symbol of the same string; existing script recursion remains. Text's Roman intent is encoded by its variant. A future font/style field must join the hash explicitly.
- `incremental::shift_math`: Text has no nested spans; existing outer span shift and script recursion suffice. Preserve document identity. Do not shift literal content or infer per-character source offsets.
- `typeset::convert_math`: Text becomes `AtomClass::Ord` with existing math-layout `Nucleus::Text`, retaining script attachment. Do not call `text_op` or convert characters to math-italic symbols.

`git apply --check` passes in an isolated scratch tree containing only the two unchanged producer files. This is a mechanical compatibility patch, **not a built or acceptance-complete implementation**. Producer Cargo.toml pins vendor/compiler; the owner must update that dependency to the approved compiler candidate in a separate explicit repin. Top-level and vendored compiler/math-layout files relevant to this review are identical at the pinned producer base. No authoritative producer/compiler or native file was edited.

## Required owner acceptance before adoption

Use the existing five HW1 arguments `(a)`, `(b)`, `(c)`, `(d)`, `and`. Verify distinct Symbol/Text hash behavior, changed Text content invalidates the cache, unchanged Text with preceding source edits relocates only outer/script spans, and incremental output equals clean output. Check the actual Roman metric/outline resource at each size and original cmap GID; preserve optical-design/missing-resource diagnostics. Existing `TexMathMetrics::text_glyph` selects Roman metric slots, and `otf_glyph` prefers Roman faces, but can fall back to math outlines with existing profile warnings. Neither compiler TimesRoman intent nor this adapter is a promise of matching resource identity across different output routes.

**Scope blocker for broader literal text:** root's compiler candidate can collect normalized spaces and escaped braces, but producer `make_text` currently places per-character Roman metrics with no text-space glue, ligature/kern shaping or general encoding layer. ASCII character codes used as TFM slots are not universally faithful for arbitrary literal punctuation. The adapter does not fix this. Before repinning, the owner must either implement the required existing text-layout/resource path or emit explicit unsupported diagnostics for forms outside its verified subset. Do not silently advertise full `\text` because the new enum compiles. The mechanical patch alone has no error-return channel in `convert_math`, so a complete subset refusal policy needs an owner-defined diagnostic hook.

Math output currently assigns the entire math expression range to emitted clusters; keep that truthful provenance. Exact command/group spans in compiler AST do not automatically provide token-level producer hit mapping. The new `MathItem.font` is used by the compiler's layout path; producer consumes AST independently and must preserve Roman intent via the conversion above. No mathbb substitution, arbitrary shaping, native parity or full-corpus claim is authorized here.
