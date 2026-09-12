# Development reference runner

See [the corpus guide](../../tests/extended-tex-corpus/README.md) for commands,
coverage, artifact provenance, edit scenarios, and acceptance criteria.
`reference.py` invokes installed reference TeX engines only on explicit generation
commands. Validation and request emission do not invoke TeX. No product module
imports this runner. `test_reference.py` checks fixture/reference integrity,
lossless multi-file requests, binary-asset rejection and timeout behavior.

`incremental.py` sends each recorded edit as original, edited, undo and reapplied
project snapshots in one original-compiler process. It compares the final three
complete JSONL replies byte for byte against separate fresh processes receiving
the identical request, including IDs, revisions and capability negotiation.
Requests, raw replies/stderr, worker exit/timeout state and artifact/source hashes
are retained. It does not invoke TeX or establish reference-PDF parity.

```sh
python3 tools/extended-tex-corpus/incremental.py \
  --compiler /absolute/path/to/flashtex-compiler \
  --output /private/tmp/flashtex-incremental-acceptance
```

The default runs all fourteen scenarios in both legacy (capabilities omitted)
and typed (`rules-v1`, `font-hints-v1`) modes. Use `--only CASE_ID` and
`--profile legacy` or `--profile typed` for bounded subsets. Each worker has a
20-second timeout; each scenario/profile launches one four-request warm worker
and three one-request clean workers. An error envelope, bad correlation, missing
reply, crash, timeout, raw-byte difference or changed compiler artifact fails.
Unsupported features may still produce consistent recovered output: this gate
does not treat consistency as feature support. Compiler source revision remains
unknown unless the caller supplies independently verified build provenance.
