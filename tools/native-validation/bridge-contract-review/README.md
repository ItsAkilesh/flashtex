# Mac bridge contract review

Owner: Linux product review worker, September 12, 2026. This isolated checker
reviews pinned Git objects without checking out or changing Mac implementation.
It emits known-bad structural signals with file evidence and exits 1 when any
are present. `OBSERVED` means a source guard exists, not that native behavior
passed. `REVIEW` remains an unmet verification obligation.

Run from the repository root:

```sh
python3 -m unittest discover -s tools/native-validation/bridge-contract-review -v
python3 tools/native-validation/bridge-contract-review/check_bridge_contract.py --repo . --mac-sha 48780a88fda46ea7be7e600fd2af663890b9ca41 --contract-sha b5ca96bdaca634166a01023cd3321955f2bc5f70
```

The Python scalar-boundary oracle tests adversarial byte ranges, revision,
document identity, removed text and source hash. It deliberately does not claim
to execute Swift, validate an Xcode build, perform a source transaction, or test
camera/Pencil/pairing and visual fonts. Source checks match this reviewed code
shape and can need maintenance after refactoring. Absence of a known-bad
signature is not proof that a rewritten implementation is safe.

Native acceptance must inject ledger read/write errors and process disconnects,
restart after each source/ledger/receipt boundary, reject stale session callbacks
and mismatched payload identities, and verify UI responsiveness when a large
capture write queues behind a slow conversion. Persist source together with the
applied-ID ledger before acknowledging insertion. Check font substitution and
proposal-in-document diagnostics on the actual Mac.
