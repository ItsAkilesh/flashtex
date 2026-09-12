# Existing strict consumer accepts the new10/12pt producer output

The unchanged producer7ca34cec output retained in `../linux-7ca34cec` binds through
existing `PipelineCff` and exports using the original PDF owner's writer. No producer
or consumer rebuild was performed for this check. Existing `pipeline_cff_probe`
SHA256 is41d9cc1f624d6d85f5c2928b43738f9d7ff7c669d4a5de98ab4d5a9eb90d6c7a;
consumer source/dependency/example paths have no Git diff from tested fce554f1.

Inputs remain original bytes. Both request identities and all document path/hash/
revision/length fields match the prior fixtures, as do full raw font SHA/face/GID
metadata and exact licensed font resources. New v1 results are `ok` with zero
diagnostics. No recovered-result acceptance policy was relaxed.

The existing example was invoked as:

```sh
crates/rendering-core/target/debug/examples/pipeline_cff_probe DISPLAY REQUEST FONT LICENSE OUTPUT_PREFIX --searchable
```

For10pt, prior DISPLAY/REQUEST/FONT come from
`tests/fixtures/helper-multidoc-eca6ab25`; for12pt they are
`tests/fixtures/original-reference/65dbe7d-clean-v2.json`, `request.jsonl`, and
`lmroman12-regular.otf`. New DISPLAY is the corresponding original bundled output
in `../linux-7ca34cec`. Existing GUST license bytes are unchanged. Full input,
font, output and extraction hashes are recorded in `compatibility.json` and each
exporter's own evidence JSON. `pdftotext26.01.0` reads the resulting PDFs.

Results are deliberately separated:

- 10pt:99 supplied cluster hit/caret `top` coordinates moved by exactly-1 tick.
- 12pt:20 such coordinates moved by exactly-1 tick.
- All other JSON leaves, including glyph paint origins, originalGIDs, cluster text
and source spans, match. Full display equality is false; no tolerance was used.
- PDF bytes and extracted text are identical to each respective old fixture.
- PDF10pt SHA d1cc00ca990c3c7178c89aa57c06eca180d82d0aa8fc720ee06f3555d1b1ddb0;
PDF12pt SHA fdaf6f847e78fa6de55a6a9b2108987ea8af6a58e4c432ecf4a70b5223243f92.

The exact changed paths and values are preserved, not normalized away. These hit
rectangles are source-query geometry, so PDF equality does not claim identical
selection boundaries. The unchanged consumer uses whatever validated geometry the
producer supplies; no inferred TeX caret mapping was introduced.

Two explicitly malformed derivatives (files named `wrong-*`, not original producer
evidence) alter only raw font SHA or source SHA. The existing consumer returns
`CFF resource metadata mismatch` or `source identity mismatch`, creates no PDF,
and leaves original inputs untouched.

This is actual standalone PipelineCff/immutable-resource/export compatibility.
It does not execute the native helper wrapper, enable source_actions, verify a
signed app, establish full math support, or compare new output with an external
reference/rasterizer. Existing reference gaps and the separate GH36 Mac gates
remain open. All probes finished; no timed measurement or broad corpus sweep.
