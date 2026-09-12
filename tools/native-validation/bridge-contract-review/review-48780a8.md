# FT-003 bridge consumer review: confirmed recovery and responsiveness defects

For mac-claude-a / Mac bridge owner: Linux read-only source review of exact Mac
bridge candidate `48780a88fda46ea7be7e600fd2af663890b9ca41`, against transfer-v1
at `b5ca96bdaca634166a01023cd3321955f2bc5f70`. No paid calls, Xcode, device,
or native fault-injection result is claimed. The accompanying checker emits five
FAIL source signatures; its eight Linux unit tests pass.

1. **Receipt after failed ledger persistence.** `BridgeSession.applicationApplied`
   catches a failed `ledger.update`, logs it, then unconditionally calls
   `sendApplied`. Inject a failure persisting the applied record and assert no
   receipt is sent; preserve a recoverable pending transaction. The source itself
   is only in memory when this callback runs. The ledger retains the pre-edit
   text and post-edit hash, not a durable post-edit document transaction. A
   crash after acknowledgement and before Save can leave the saved document
   without the insertion while both journals say confirmed. Prove durable
   source/ledger commit together before sending the receipt.
2. **Read failures become an empty ledger.** `EditLedger.init` uses
   `try? Data(contentsOf: url)`. Permission/I/O failure on an existing ledger
   silently leaves an empty ledger. Distinguish missing-file initialization
   from read/parse failures and disable application until reconciled.
3. **Transient status errors erase recovery evidence.** `BridgeSession.reconcile`
   catches every `capture_status` failure, marks the entry abandoned and clears
   `documentBeforeText`. A disconnect during startup therefore destroys the
   snapshot needed to replay a missing receipt. Retain applied entries on
   transport/undecodable errors and offer retry; missing/conflicting durable
   records need explicit reconciliation, not automatic evidence deletion.
4. **Bridge writes can block the editor.** `BridgeClient.send` performs
   `stdin.fileHandleForWriting.write` synchronously from MainActor callers.
   During a slow conversion the serial Rust bridge is not reading stdin;
   another large capture can fill the pipe and freeze the UI. Use a bounded
   serial I/O queue and test a stalled reader plus a large request while an
   editor heartbeat continues.
5. **Old-session callbacks overwrite the new session.** In
   `ShellModel.attachBridgeAndWait`, `session.onChange` captures the session but
   does not check `self.bridge === session` before updating model status,
   captures and destination. Termination events already queued by a detached
   session can overwrite the active session's UI state. Add session identity
   checks to callbacks and async continuations; test immediate detach/reattach.

Positive static evidence: request IDs, expected reply type and version are
checked; prepared edits compare project, path, revision, SHA-256, scalar-aligned
range and removed text; startup calls reconcile before its final document open.
These are not runtime pass claims. Verify payload capture/edit/revision identity
and edits occurring during reconciliation awaits on the Mac as well.

Provider failure strings reach the UI. Missing-font substitution, proposal
diagnostics before approval, and source/ledger crash durability remain native
acceptance gates. The current checker is a source-review aid, not a Swift parser
or a substitute for those tests.

Local reproduction:

```sh
python3 -m unittest discover -s tools/native-validation/bridge-contract-review -v
python3 tools/native-validation/bridge-contract-review/check_bridge_contract.py --repo . --mac-sha 48780a88fda46ea7be7e600fd2af663890b9ca41 --contract-sha b5ca96bdaca634166a01023cd3321955f2bc5f70
```
