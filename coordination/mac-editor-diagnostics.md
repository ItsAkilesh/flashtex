# mac-editor-diagnostics (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T10:20Z
- Agent / parent / machine alias: mac-editor-diagnostics / mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: lane "Partial recovery error marks
  preserve exact source identity" (+ stale invalidation, keyboard error
  navigation — merged into mac-shell) and refill "consume
  crates/diagnostic-explanations" (gate 3: error recovery and source
  navigation). Owned: `apps/mac/Sources/FlashTeXMac/EditorDiagnostics.swift`,
  `apps/mac/Sources/FlashTeXAccessibility/EditorDiagnosticsAccessibility.swift`,
  `apps/mac/Tests/FlashTeXMacTests/EditorDiagnosticsTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/EditorDiagnosticsExplanationsTests.swift` (new),
  `apps/mac/Tests/FlashTeXAccessibilityTests/EditorDiagnosticsAccessibilityTests.swift`,
  this handoff and `coordination/agents/mac-editor-diagnostics.json`.
- Branch / code revision / main integrated through:
  `agent/mac-editor-diagnostics/explanations` from
  `origin/agent/mac-claude-a/mac-shell` 271a366 (≥ 40d53b7; my first lane and
  the parent diffs are in). Previous branch
  `agent/mac-editor-diagnostics/exact-marks` (ef4ea4c) is superseded.
- State: ready for integration (parent review; consumer diffs below not
  applied; the crate owner needs to add the helper binary, see below).

## Finding: where the crate actually is and what it offers

- `crates/diagnostic-explanations` is **not on `origin/main`** (checked at
  60a498a…f0f4877 on 2026-09-12); it lives on
  `origin/agent/mac-diagnostic-explanations/explain` 2bf14cd (owner
  mac-diagnostic-explanations, state "ready for integration"). I built from
  that SHA in a scratch archive.
- It offers a **Rust library only**: `explain_all`, `explain_compile_result_json`,
  `json::explanations_to_json`, fixed-key JSON. No `[[bin]]`, no static
  table, no FFI. Its README proposes either a bridge FFI or a tiny CLI
  `flashtex-explain` and leaves the choice to the Mac owner.
- Chosen transport: **child process, JSON Lines**, `flashtex-explain`, a
  20-line front end over `explain_all` (exact source below). For tests I built
  it in the scratch archive against the crate at 2bf14cd; it is not in the
  repository yet because `crates/diagnostic-explanations/**` is not mine.

## Ready behavior and evidence (this refill)

- `EditorDiagnostics.Explanation` (+ `Suggestion`, `Edit`, `Context`):
  `Decodable` for the crate's fixed keys (`catalog_id, title, category,
  severity, message, why, what_happened, suggestions, context`); `line` =
  `"explain: <title> — <why>"` (catalogued) or `"explain: <title> (not in the
  catalogue)"`, cut at 240 characters. Nothing is ever applied from `edits`.
- Bounds (`ExplanationLimits`): reply ≤ 4 MiB (refused whole), count must
  equal the result's diagnostic count and ≤ 2000 (entries are matched by
  index), title ≤ 120, paragraphs ≤ 600, ≤ 4 suggestions × ≤ 8 edits, context
  ≤ 2000 chars, ≤ 8 results cached. Failures are typed
  (`ExplanationFailure.oversized/malformed/countMismatch/helper/transport/unavailable`).
- `ExplanationCache` keyed by result id (oldest evicted);
  `EditorDiagnostics.attach(_:to:)` sets `Mark.explanation` by diagnostic
  index — O(marks), no I/O; `Mark.toolTip` and `spokenDescription` add the
  line after the recovery line; `EditorDiagnosticNavigation.Item.explanation`
  makes the ⌘⇧]/⌘⇧[ announcement
  `"Error 1 of 1, line 5: <message> — recovery: … — explain: …"`.
- `ExplanationClient` (same file): `LineProcessClient` over `flashtex-explain`
  (`FLASHTEX_EXPLAIN`, bundled beside the app, or
  `crates/diagnostic-explanations/target/{release,debug}/flashtex-explain`),
  request `{"id","type":"explain","compile_result":<payload>,"documents":[…],"supported":[…]}`,
  reply `{"id","type":"explanations","explanations":[…]}` /
  `{"id","type":"error","error":{code,message}}`; replies decoded and bounded
  off the caller's path, delivered on the main queue, 10 s deadline.
- **Real crate output** (`testRealHelperExplainsCompilerDiagnostics`, runs
  when `FLASHTEX_EXPLAIN` is set, otherwise `XCTSkip` with build
  instructions): the live compiler's `\foo` diagnostic on
  `Samples/recovery-demo.tex` (message cross-checked against
  `FLASHTEX_COMPILER`) explains as catalogue `unsupported-command`, title
  `\foo is not supported`, edit `main.tex 181..<190 → "bar"` (exactly
  `\foo{bar}`), context line 5 column 45; deterministic across two calls; the
  multipage sample's two uncatalogued messages fall back (`catalog_id: null`,
  count preserved, nothing hidden). Fake helper (`/bin/sh` answering with a
  canned file): malformed entry → `.malformed("type mismatch … title")`,
  4 KB reply over a 512-byte limit → `.oversized`, one entry for two
  diagnostics → `.countMismatch`, helper `error` envelope → `.helper`, a
  non-JSON line → `.transport("no reply within 1 s")` (never hangs).
- **Cost after the first fetch** (`testMarksPlusExplanationsStayFastAfterFirstFetch`,
  62 000-byte document, 200 diagnostics, 134 826-byte reply decoded once =
  the first fetch, then `report` + `attach` per keystroke, 25 samples, fresh
  String each): **first-fetch decode 3.49 ms (off the keystroke path); per
  keystroke best 1.066 ms / median 1.081 ms / worst 1.199 ms (debug build,
  machine shared with other agents)**; asserted `< 2 ms` on the best sample.
  The marks-only bench in the same run: best 1.02 / median 1.04 ms (it was
  0.83 ms on an idle machine earlier today; release 0.245 ms).
- Validation: see "Validation" below.

## Exact diffs needed in files I do not own (not applied)

### 1. `crates/diagnostic-explanations` (owner mac-diagnostic-explanations) — add the helper binary

`Cargo.toml`:

```toml
[[bin]]
name = "flashtex-explain"
path = "src/bin/flashtex-explain.rs"
```

`src/bin/flashtex-explain.rs` (this exact file is what the tests ran against,
built in the scratch archive with `flashtex-diagnostic-explanations = { path = … }`):

```rust
//! flashtex-explain: JSON Lines front end for flashtex-diagnostic-explanations.
//! Request line: {"id","type":"explain","compile_result":<envelope|payload>,
//!   "documents":[{"path","text"}],"supported":["section",…]}
//! Reply line:   {"id","type":"explanations","explanations":[<Explanation>…]}
//!            or {"id","type":"error","error":{"code","message"}}
//! `explanations` is exactly json::explanations_to_json(explain_all(..)).
use flashtex_diagnostic_explanations::json::{self, Value};
use flashtex_diagnostic_explanations::{explain_all, Document, DEFAULT_SUPPORTED_COMMANDS};
use std::io::{self, BufRead, Write};

const MAX_LINE: usize = 12 * 1024 * 1024;

fn reply(out: &mut impl Write, id: &str, body: &str) {
    let mut s = String::from("{\"id\":");
    json::escape_into(&mut s, id);
    s.push(',');
    s.push_str(body);
    s.push_str("}\n");
    let _ = out.write_all(s.as_bytes());
    let _ = out.flush();
}

fn error(out: &mut impl Write, id: &str, code: &str, message: &str) {
    let mut m = String::new();
    json::escape_into(&mut m, message);
    reply(out, id, &format!("\"type\":\"error\",\"error\":{{\"code\":\"{code}\",\"message\":{m}}}"));
}

fn handle(line: &str, out: &mut impl Write) {
    let v = match json::parse(line) {
        Ok(v) => v,
        Err(e) => return error(out, "", "malformed_json", &e.to_string()),
    };
    let id = v.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    if v.get("type").and_then(Value::as_str) != Some("explain") {
        return error(out, &id, "bad_request", "type must be \"explain\"");
    }
    let Some(cr) = v.get("compile_result") else {
        return error(out, &id, "bad_request", "compile_result is required");
    };
    let payload = match cr.get("payload") {
        Some(p) if cr.get("diagnostics").is_none() => p,
        _ => cr,
    };
    let Some(diags) = payload.get("diagnostics").and_then(Value::as_arr) else {
        return error(out, &id, "bad_request", "compile_result.diagnostics must be an array");
    };
    let diagnostics: Vec<_> = match diags.iter().map(json::diagnostic_from_value).collect() {
        Ok(d) => d,
        Err(e) => return error(out, &id, "bad_request", &e.to_string()),
    };
    let docs: Vec<(String, String)> = v
        .get("documents")
        .and_then(Value::as_arr)
        .map(|a| {
            a.iter()
                .filter_map(|d| Some((d.get("path")?.as_str()?.to_string(), d.get("text")?.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();
    let documents: Vec<Document<'_>> = docs.iter().map(|(p, t)| Document { path: p, text: t }).collect();
    let supported_owned: Option<Vec<String>> = v
        .get("supported")
        .and_then(Value::as_arr)
        .map(|a| a.iter().filter_map(|s| s.as_str().map(str::to_string)).collect());
    let supported: Vec<&str> = match &supported_owned {
        Some(s) => s.iter().map(String::as_str).collect(),
        None => DEFAULT_SUPPORTED_COMMANDS.to_vec(),
    };
    let explanations = explain_all(&diagnostics, &documents, &supported);
    let body = json::explanations_to_json(&explanations);
    reply(out, &id, &format!("\"type\":\"explanations\",\"explanations\":{body}"));
}

fn main() {
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().split(b'\n') {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.len() > MAX_LINE {
            error(&mut out, "", "line_too_long", &format!("request line of {} bytes exceeds {MAX_LINE}", line.len()));
            continue;
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        match std::str::from_utf8(&line) {
            Ok(s) => handle(s, &mut out),
            Err(_) => error(&mut out, "", "malformed_json", "request line is not UTF-8"),
        }
    }
}
```

Until that lands: build the same file in a scratch crate depending on the
crate by path (as I did) and export `FLASHTEX_EXPLAIN=<path>` for the test.
The app bundle should ship it beside `flashtex-compiler` (packaging owner).

### 2. `apps/mac/Sources/FlashTeXMac/ShellModel.swift` (parent)

Add next to `editorMarkReport` (line ~212 at 271a366):

```swift
    /// Offline explanations per result id (crates/diagnostic-explanations via
    /// flashtex-explain); attached to marks, never blocking a keystroke.
    private(set) var explanations = EditorDiagnostics.ExplanationCache()
    @ObservationIgnored private var explanationClient: ExplanationClient?
    var explanationStatus: String?

    /// Asks the helper once per result; the cache is read by `editorMarkReport`.
    private func fetchExplanations(for result: RuntimeV1.CompileResult, id: String, documents: [RuntimeV1.Document]) {
        guard explanations[id] == nil else { return }
        if explanationClient == nil || explanationClient?.isRunning == false {
            guard let exe = ExplanationClient.locate() else { explanationStatus = nil; return }
            explanationClient = try? ExplanationClient(executable: exe) { [weak self] e in self?.explanationStatus = "flashtex-explain: " + e }
        }
        explanationClient?.explain(result: result, documents: documents, supported: Completion.defaultSupported) { [weak self] outcome in
            guard let self else { return }
            switch outcome {
            case .success(let list):
                self.explanations.store(list, for: id)
                self.editorMarksCache = nil            // re-attach on the next read
                self.explanationStatus = nil
            case .failure(let f):
                self.explanationStatus = f.text        // shown in the footer; marks stay as they are
            }
        }
    }
```

In `editorMarkReport`, replace the `let report = …` line with:

```swift
        let report = EditorDiagnostics.attach(explanations[resultID], to: EditorDiagnostics.report(
            for: result, resultID: resultID, path: activePath,
            compiledText: compiledDocuments[activePath], currentText: activeText))
```

and add `explanationsCount: Int` to `EditorMarksKey` (value
`explanations[resultID]?.count ?? -1`) so the memo refreshes when the reply
lands (or keep `editorMarksCache = nil` above; either works).

After each result is bound — fixture path (after
`compiledDocuments = Dictionary(uniqueKeysWithValues: req.payload.documents…)`,
line ~312) and worker path (after `compiledDocuments = …sent.documents…`,
line ~621):

```swift
            fetchExplanations(for: res.payload, id: res.id, documents: req.payload.documents)   // fixture
            fetchExplanations(for: incoming, id: env.id, documents: sent.documents)             // worker
```

`terminate` the client where the worker is torn down (`setBridge`/deinit).

### 3. `apps/mac/Sources/FlashTeXMac/ContentView.swift` (parent)

Diagnostics list row, after the recovery `Text`:

```swift
                        if let explain = model.explanations.explanation(resultID: model.resultID, index: i)?.line {
                            Text("↳ \(explain)").font(.caption).foregroundStyle(.secondary)
                        }
```

Footer: `Text(model.navigationNote ?? model.editorMarkReport.staleNote ?? model.explanationStatus ?? "Click text …")`.

`accessibleDiagnostic` (owner mac-accessibility) can append the same line to
its `accessibilityValue`; the mark's `spokenDescription` already has it.

## Incomplete behavior / needs from others

- No UI consumes explanations until the diffs above are applied; the helper
  binary is not in the repository (crate owner). No live VoiceOver run
  (Accessibility permission not granted); announcement text is unit-tested.
- The crate's `suggestions.edits` are decoded and bounded but not previewed
  or applied — "Preview fix" is the crate README's follow-up, not this lane.
- The catalogue is pinned to compiler wording at 1dd26c5/de1020c/6b13034;
  new compiler messages fall back to `(not in the catalogue)` (tested with the
  multipage sample), so the line never blanks.

## Validation

- `swift test` in `apps/mac` with `FLASHTEX_EXPLAIN` (scratch helper over
  crate 2bf14cd), `FLASHTEX_COMPILER/PDF/BRIDGE/EDIT_LEDGER` (main checkout
  release builds), `FLASHTEX_NO_ACTIVATE=1`: see the agent record for the
  exact totals at the tested SHA (recorded after the run).
- `EditorDiagnosticsExplanationsTests` 6/6 with the helper; 5 pass + 1 loud
  skip without it (verified both ways).

## Reviewed peer revisions / adaptations

- `origin/agent/mac-claude-a/mac-shell` 271a366: base; my previous lane and
  the ShellModel/Navigation/ContentView diffs are applied there — no change
  to them from this refill beyond the diffs above.
- `origin/agent/mac-diagnostic-explanations/explain` 2bf14cd: crate read in
  full (README, lib.rs, json.rs); adaptation: JSON Lines helper over its
  public API instead of FFI, fixed-key `Decodable` mirror, index-matched
  attach, fallback line for `catalog_id: null`.
- `origin/main` f0f4877: coordination only; the crate is not there (reported).

## Resources / next

- Resource pool: shared Claude Max 20x quota with parent mac-claude-a; no
  purchases; totals unknown to this worker.
- Dirty files / running jobs: none after commit; helper binary and crate
  export live only in the session scratch archive.
- Exact next action: parent applies diffs 2–3; crate owner adds diff 1;
  packaging ships `flashtex-explain`; re-run `swift test` with
  `FLASHTEX_EXPLAIN` set.
