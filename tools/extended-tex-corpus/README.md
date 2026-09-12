# Development reference runner

See [the corpus guide](../../tests/extended-tex-corpus/README.md) for commands,
coverage, artifact provenance, edit scenarios, and acceptance criteria.
`reference.py` invokes installed reference TeX engines only on explicit generation
commands. Validation and request emission do not invoke TeX. No product module
imports this runner. `test_reference.py` checks fixture/reference integrity,
lossless multi-file requests, binary-asset rejection and timeout behavior.
