# Lane daniel-symbol-font log

## 2026-09-12T19:56:06Z — finding + fix, stopped on quota
- Branch agent/daniel-symbol-font/compiler @ c6024460 (product commit, crates/pdf only), base origin/main 79986817.
- Finding: warning-only on HW1, no wrong glyphs. Compiler emits Symbol hints correctly (∈12 ∀10 ∃6 ∣4 ⇒4 ∨4 √2). crates/pdf writer.rs resolve() treated "Symbol" as unknown family -> Times-Roman + warning, but encoding.rs falls back per char to /F2 Symbol (built-in encoding) for non-WinAnsi chars. HW1 content streams show /F2 bytes CE 7C D6 22 24 DE DA (element, bar, radical, universal, existential, arrowdblright, logicalor); pymupdf reports those chars in font Symbol; page 2 raster viewed, glyphs correct.
- Latent defect fixed: × ÷ ± · (in both encodings) would draw from Times with Symbol widths. Symbol hint now resolves to primary Font::Symbol (Face::Times) and encode_runs tries Symbol first, then Times, then embedded. No warning. Re-rendered HW1: identical F2 bytes and text, warning gone.
- Tests: pdf 89 passed 0 failed (new render.rs symbol_font_hints_use_base14_symbol_with_its_builtin_encoding); clippy -D warnings, fmt clean. Compiler not touched (tests not rerun).
- Next: none required; stopped per quota instruction.
