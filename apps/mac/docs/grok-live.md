# Live Grok (xAI) on the Mac

Lane `mac-grok-live` (child of `mac-claude-a`), branch `agent/mac-grok-live/wiring`.
The Rust transports already existed (`crates/bridge --enable-grok`,
`crates/assistant-context --features grok --provider-session`); this is the Mac
side that supplies the credential, selects the provider, runs the children and
keeps the review/approval boundary unchanged. **No xAI key exists on this
machine at the time of writing; nothing here has made a live call.** (Updated below: a key was supplied later and the live check was run — see "Live results".) Everything
below the "live-ready" line is exercised by tests without a network.

## Supplying the key

Either of:

```sh
security add-generic-password -U -s tech.jay3332.flashtex.xai -a xai -w '<key>'
```

(`-U` replaces an existing item; remove with
`security delete-generic-password -s tech.jay3332.flashtex.xai -a xai`), or
**FlashTeX → Preferences (⌘,) → Grok (xAI) → xAI API key → Save to Keychain**.
The item is a generic password in the login keychain (service
`tech.jay3332.flashtex.xai`, account `xai` — an item under account `XAI_API_KEY` is read too — label "FlashTeX xAI API
key"), written with `SecItemAdd`/`SecItemUpdate` — no third-party code. After
an ad-hoc re-sign of a rebuilt app macOS may ask once whether the new build may
read the item; that is the system's prompt.

For one process only, the environment also works: `XAI_API_KEY` (checked
first) or `FLASHTEX_GROK_API_KEY`. `FLASHTEX_KEYCHAIN_OFF=1` skips the Keychain
entirely (tests, headless runs). Source order is fixed: Keychain → `XAI_API_KEY`
→ `FLASHTEX_GROK_API_KEY` → none (`GrokCredential.resolve`).

The key never appears in argv, JSON, logs, `captureNote`, status text or
evidence: every text derived from a resolution says only
`key present (Keychain)` / `key present (environment: XAI_API_KEY)` / absent.
The value leaves `GrokCredential.Resolution` in exactly two places: a child
process environment (`inject(into:as:)`) and the probe's bearer header.

## Selecting Grok for editor assistance

- `FLASHTEX_ASSISTANT_PROVIDER=grok` in the environment, or
- Preferences → Grok (xAI) → **Use Grok (xAI) for editor assistance**
  (`UserDefaults` `FlashTeX.Grok.v1.providerEnabled`; the environment variable,
  when set to anything, wins — a path still selects a local provider command).
- Model: `FLASHTEX_GROK_MODEL`, else the Preferences model field
  (`FlashTeX.Grok.v1.model`), else `grok-4.6` (the bridge's default). One model
  id serves both helpers.

The review sheet's "Explain" then runs, per request:

1. `flashtex-assistant-context` one-shot `prepare` (offline, credential-free
   environment, as before) → bound context;
2. `flashtex-assistant-context --provider-session mac-explain-N-xxxxxxxx <model>`
   with `FLASHTEX_GROK_API_KEY` **in that child's environment only**
   (`GrokProviderSession`): `provider/admit` with the exact prepare request
   (`user_requested:true`, allocation `flashtex-mac explain <id> <project>`),
   then `provider/poll` every 0.4 s with the current shadow sources until the
   helper returns a `validated_proposal` (`applied:false`) or a terminal state;
   the child is closed afterwards;
3. the one-shot helper's `review` (edits) or `validate` (explanation only),
   then the reviewer's explicit **Approve** → `approve` → amended draft, exactly
   as for a local provider command. Nothing is written to the document by any
   of this.

The helper used for the session is `FLASHTEX_ASSISTANT_CONTEXT_GROK` when set,
else the same helper as the one-shot stages. A helper built without
`--features grok` exits 2 at launch and the sheet says so. The default
(non-grok) pinned helper remains what the packaged app bundles; a packaged live
build needs `scripts/make-app.sh --assistant <grok build>`.

Grok selected without a key: the context is prepared and the request fails
with "Grok (xAI) is selected but no API key is present … sent nowhere". Nothing
selected and no key: unchanged — the deterministic local provider or none.

## Capture-to-LaTeX

`attachDiscoveredBridge()` (auto-attach and Edit › Attach Capture Bridge)
resolves the credential once: with a key the bridge is launched
`--enable-grok` with `XAI_API_KEY` and `FLASHTEX_GROK_MODEL` in an environment
from which every credential/token/proxy-like variable was first removed
(`GrokCredential.bridgeEnvironment`); without a key it is launched as before
(no `--enable-grok`, same stripping, no key). A Nearby/file capture is then
`capture_submit` → **Edit › Convert Capture** (`capture_convert`, one Grok
vision request inside the bridge) → `capture_proposal` queued in the review
sheet → the existing prepare/verify/apply flow on explicit approval. Bridge
provider errors are shown as guidance: `provider_disabled` / `provider_auth_missing`
(add the key, re-attach), `provider_auth_error` (401/403: Test Connection),
`provider_rate_limited` (429), `provider_timeout` (90 s), `provider_transport_error`.
Re-attach the bridge after changing the key.

## Test connection

Preferences → Grok (xAI) → **Test connection** performs `GET
https://api.x.ai/v1/models` with the key as a bearer token (ephemeral
`URLSession`, no redirects, no cookies, 10 s) and reports the HTTP class only:
2xx accepted, 401/403 rejected, 429 rate limited, 5xx server error, timeout,
transport failure. No body is read or shown. This is the one network request
the Mac app makes itself, only on that click; the helper has no probe command
yet (request R3). `FLASHTEX_GROK_BASE_URL` (loopback `http://` or any
`https://`) redirects the probe, which is how the tests exercise 401/429/503/
timeout against `Fixtures/fake_xai_http.py`.

## Live-ready vs blocked on the credential

Live-ready (code paths complete, tested against doubles, waiting only for a key):

- credential adapter, Preferences row, Keychain store/remove (a real `SecItem`
  round trip runs in tests under a throwaway service name);
- provider selection and the session driver (`admit`/`poll`/cancel/timeouts,
  key placement, review/approve continuation);
- bridge `--enable-grok` launch with the key, conversion error guidance;
- the probe;
- `apps/mac/scripts/grok-live-check.sh` and `GrokLiveAcceptanceTests`.

Blocked on the credential (never run here): any actual xAI request, the
evidence under `docs/evidence/grok-live-<UTC>/`, and therefore any claim that
model access, schema behaviour or latency work end to end.

## Live acceptance

```sh
apps/mac/scripts/grok-live-check.sh          # builds the grok helper + bridge, then runs
apps/mac/scripts/grok-live-check.sh --no-build
```

Exits 3 and does nothing when no key is present. With a key it makes exactly
three requests through the Mac's own code (probe; one explanation via the real
helper's provider session; one capture conversion via the real bridge) by
running `GrokLiveAcceptanceTests` with `FLASHTEX_GROK_LIVE=1`, and records
`probe.json`, `explanation.json`, `conversion.json`, `swift-test.log` and a
`README.md` (commit, key source, model, ids, byte counts, edit counts,
timings, HTTP class) under `docs/evidence/grok-live-<UTC>/`. Prompt and reply
bodies are not recorded; the script refuses to keep evidence that looks like a
key. xAI response ids and token usage cannot be recorded until the helpers
surface them (R4 below).

## Tests (no network)

`GrokLiveTests` (12) + `CaptureFeaturesTests` (2): resolution order and env-only mode; no text contains the
key; real Keychain round trip under a throwaway service; model/preferences;
provider selection from environment vs preference; the key on the grok child
only (helper/grok minimal env, bridge env with/without key); session double
end to end (prepare → admit/poll → review → approve, launches/roles/argv/env
recorded); Grok selected without key sends nothing; helper `failed` state,
provider timeout, reviewer cancellation with late-reply discard, helper without
the feature; the real helper started/cancelled in provider-session mode
(no admit); probe classes 200/401/429/503/timeout/transport against the
loopback stub; fake bridge grok gate (`%grokgate`) with and without a key.
`GrokLiveAcceptanceTests` (3) skip unless `FLASHTEX_GROK_LIVE=1` and a key
resolves.

## Requests for the helper owners (crates read-only for this lane)

R1. **`crates/assistant-context` (grok):** an endpoint override for local
stubs — `FLASHTEX_GROK_BASE_URL` (accept only `https://…` or loopback
`http://127.0.0.1:…`), read once at `--provider-session` startup and passed to
`GrokClient::new`, so the Mac can drive the real helper against
`Fixtures/fake_xai_http.py` (which already answers `POST /v1/responses` in the
shape `grok.rs` parses). Today `ENDPOINT` is a private const and `request_at`
is private.

R2. **`crates/assistant-context` (grok):** surface the failure class in
`provider_status`. `ProviderQueue::submit_with` maps every provider error to
`Failure::new(FailureKind::Provider, "provider failed; no retry")`, so
`state:"failed"` cannot tell a 401/403 from a 429, a timeout or a transport
error although `GrokClient::send_at` already produces
`provider HTTP 401; no retry` / `provider timeout; no retry`. Proposal: keep the
scheduler failure but retain a bounded `reason` (`auth`, `rate_limited`,
`timeout`, `transport`, `http_<code>`, `invalid_reply`) alongside the record and
emit it as `{"type":"provider_status","state":"failed","reason":"auth"}`. The
Mac then maps it to the same guidance the bridge path gets.

R3. **`crates/assistant-context` (grok):** a `probe` provider command
(`{"operation":"probe"}` → `{"type":"provider_probe","http_class":"2xx"|"401"|…}`)
using a minimal unbilled request (`GET /v1/models` or `GET /v1/api-key`), so
"Test connection" moves behind the helper and the Mac app makes no network
request of its own.

R4. **Both crates:** carry provider evidence without bodies: the xAI response
`id`, echoed `model`, and `usage` (input/output tokens) — in
`validated_proposal` (helper, e.g. `"provider": {"response_id","model","usage"}`)
and in `capture_proposal` (bridge). The live-check evidence currently records
only the Mac's session/request ids and byte counts.

R5. **`crates/bridge`:** `FLASHTEX_GROK_BASE_URL` for the same reason as R1
(`ENDPOINT` is `pub const` but not configurable at runtime).

None of these block the live path; they block hermetic end-to-end tests of the
real helpers and richer evidence.

## Parent-retained files

No change was needed in `ShellModel.swift`, `ShellModel+Controller.swift`,
`ContentView.swift`, `PreviewView.swift`, `FlashTeXMacApp.swift` or
`SourceEditorView.swift`: the Preferences row is one line in
`EditorPreferencesView` (`GrokPreferencesSection()`), the bridge hook lives in
`attachDiscoveredBridge()` (`ShellModel+Bridge.swift`), and the review sheet
picks the provider up through `ExplanationConfiguration.fromEnvironment()` at
sheet creation. `LineProcessClient`, `BridgeClient` and `BridgeSession` gained an
optional explicit child environment.

## Live results (2026-09-12, key supplied by the user in the Keychain)

`docs/evidence/grok-live-20260912T191209Z/` — all three cases passed:

| call | route | model | elapsed | result |
|---|---|---|---|---|
| probe | Mac `GET /v1/models` | — | 0.17 s | HTTP 200, key accepted (Keychain); 12 model ids listed |
| explanation | real helper `--provider-session` | grok-4.6 | 69.7 s | READY: 570-byte explanation, 1 reviewed edit, `applied:false` |
| capture → LaTeX | real bridge `--enable-grok` | grok-4.20-0309-non-reasoning | 2.8 s | proposal (68 bytes, 4 ambiguities); compiled by our compiler: **ok, 0 diagnostics** |

Measured along the way (numbers in that README): the 1×1 protocol fixture PNG
is rejected by xAI with HTTP 400; grok-4.6 on a real image exceeds the bridge's
fixed 90 s timeout; grok-4.6 explanations take 53–70 s, so the Mac now defaults
the Grok flight timeout to 100 s (the helper's HTTP client stops at 90 s); the
non-reasoning model answers captures in 2–3 s but once proposed an explanation
edit the helper refused ("removed source differs" — validation as designed).
Defaults after this: explanations `grok-4.6`, captures `grok-4.20-0309-non-reasoning`
(`FLASHTEX_GROK_CAPTURE_MODEL`, else `FLASHTEX_GROK_MODEL`, else the preference).
HTTP 401/403/429 were not observed with the supplied key.

## `supported_features` (demo gate, issue #2)

`CaptureFeatures.swift` is the checked-in list sent with every
`capture_convert`: what the compiler renders at
origin/agent/claude/compiler-foundation `49e6eb43` (`COMMAND_GLYPHS` — 60
symbols, re-derived by `CaptureFeaturesTests` from `git show` when the commit
is present — plus `$…$`, `\[…\]`/`equation`, `\frac`, `\sqrt`, `^`/`_`,
`\left…\right`, `\section`/`\subsection`, `\textbf`/`\emph`/`\textit`,
`itemize`/`enumerate`, `\label`/`\ref`), an explicit NOT-supported list
(amsmath/amssymb environments, `\mathbb`, `\text`, `\bigl`, `\quad`, TikZ,
tables, macros), the OUTPUT rule (every formula wrapped in `$…$` or `\[…\]`)
and the RULE that unsupported constructs are reported in `ambiguities`, never
substituted. 20 entries, each ≤128 bytes (the bridge's limits). The live check
inserts the returned LaTeX at the pin and compiles it with `flashtex-compiler`
(`compile_gate` in `conversion.json`: status, new diagnostics, new errors).

## Accessibility note (request for the mac-accessibility lane)

The Grok section is shown by the app's `EditorPreferencesView()` and hidden by
`EditorPreferencesView(preferences:)` unless `showGrok: true`, so the pinned
Settings table (`PanelFocusOrder.panels[0]`, checked against
`EditorPreferences.swift` markers) stays exact; its controls are keyboard-walked
by `PanelAccessibilityTests.testGrokPreferencesSectionControlsTakeKeyboardFocus`
(11 controls: secure key field, Save/Remove, provider switch, model field, Use,
Test connection). Adding them to the table needs `Control` markers that can
point at `GrokPreferencesView.swift` (the table is per-file today).
