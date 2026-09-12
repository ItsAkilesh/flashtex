# Ask Grok live checks (lane mac-grok-assistant, 2026-09-12)

Three live xAI calls total, all with the fast model `grok-4.20-0309-non-reasoning`
through the relocating helper (`agent/mac-assistant-context/relocate-edits`
d1795267, built `--features grok`), key from the Keychain (never recorded),
real `flashtex-compiler` compile as the binding. Prompt bodies are not recorded;
`live.json`/`README.md` carry timings, byte counts, edit ranges, the
replacement text and the helper's notes.

| dir | case | elapsed | outcome |
|---|---|---|---|
| `grok-assistant-20260912T221011Z` | Ask: selection `$$…$$` on line 4, "convert this displayed equation to an align* environment" | 5.92 s | 1 edit, **relocated** (the model's offsets were wrong again, as in grok-live-20260912T210600Z; the helper found the unique `removed_text`), reviewed, Apply → one pending edit, recompiled (`recovered`: the compiler does not implement `align*`, 4 diagnostics — a compiler gap, not an edit error) |
| `grok-assistant-20260912T221103Z-fix` | Fix with Grok on "math group is missing its closing brace" with the 1-byte diagnostic span as the selection | 2.97 s | explanation (492 bytes) arrived; the edit was **dropped with a note** (the destination was the single byte `$`, so no relocation target) — this is why `fixWithGrok` now selects the whole line |
| `grok-assistant-20260912T221228Z-fix` | same, whole-line selection | 2.93 s | 1 edit, relocated, `\frac{1}{2$` → `\frac{1}{2}$`, reviewable (Apply not exercised: no extra call needed) |

Before this lane the fast model was refused 2/2 ("removed source differs");
with relocation it answered 3/3 in 2.9–5.9 s and produced applicable edits 2/3
(the third was the narrow-destination case above). `GrokCredential.defaultAskModel`
is therefore the fast model; the review sheet keeps `grok-4.6`.
