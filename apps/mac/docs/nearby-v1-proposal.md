# Nearby companion transport v1 — proposal

Status: **proposal** from FT-003 (Mac shell, mac-claude-a) for the Commander to
adopt as `docs/contracts/nearby-v1.md`. Until then nothing here is a published
contract; the Mac implementation in `apps/mac/Sources/FlashTeXMac/Nearby*.swift`
and `Pairing.swift` is the reference for the Mac side only. The companion side
belongs to FT-004. Updated September 12, 2026 (receive caps, image validation
and duplicate handling added by mac-nearby-transport; wire version unchanged).

Scope: how an iPad/iPhone companion on the same network finds a Mac, pairs with
it once, and then delivers runtime-v1 `capture_submit` messages to it with
authenticated encryption, as required by transfer-v1's "Nearby transport
boundary" (Bonjour is discovery only; no plaintext process protocol on a LAN
port; the adapter forwards captures to the local bridge and returns its
acknowledgement).

## 1. Discovery (Bonjour)

The Mac advertises one service while "Advertise" is on:

| Field | Value |
|---|---|
| Service type | `_flashtex._tcp` (local domain) |
| Instance name | the Mac's user-visible name (`Host.current().localizedName`) |
| Port | ephemeral, from the SRV record; changes only when the listener restarts |
| TXT `v` | `1` — nearby protocol version |
| TXT `name` | same as the instance name |
| TXT `fp` | 16 hex chars: first 8 bytes of SHA-256(`"flashtex-nearby-v1 mac-id"` ‖ salt). Stable per Mac; lets a companion match a service to a stored pairing without connecting |
| TXT `salt` | 32 hex chars: the Mac's 16-byte pairing salt (public) |

Nothing in the TXT record is secret and nothing in it is trusted: a companion
must not connect to a service merely because it advertises a known `fp`; the TLS
handshake below is the only authentication.

## 2. Pairing

Pairing is explicit and user-driven. The Mac (`Edit > Nearby Companion…`,
⌘⇧N, "Show Pairing Code") displays a 6-digit decimal code from the system CSPRNG
for 120 s. The user types it on the companion while the Mac's service is
selected. Both sides derive:

```
ikm      = UTF-8 bytes of the code, e.g. "123456"
salt     = TXT salt (16 bytes)
psk_boot = HKDF-SHA256(ikm, salt, info = "flashtex-nearby-v1 psk",     L = 32)
pair_id  = hex(HKDF-SHA256(ikm, salt, info = "flashtex-nearby-v1 pair-id", L = 8))  — 16 hex chars
```

Test vector (pinned in `PairingTests`): code `123456`, salt
`000102030405060708090a0b0c0d0e0f` →
`pair_id = 3917d7c3e5eef7ce`,
`psk_boot = b127a48a782dbece3edbed18d027d658890624ecf21da77d19f47d5e604d1221`,
`fp = 0e712816d64b7c47`, and `hello.proof` for nonce `n-1` =
`rIBrMvRrNjs2eQueTGVsBNF6K130BW6fqV93Rw//WgM=`.

`psk_boot` is a **bootstrap** secret only. It authenticates exactly one TLS
connection, whose `hello_ack` carries a fresh random 32-byte long-term PSK
(`pair_psk`, base64). Both sides persist `{pair_id, pair_psk}`; the Mac also
keeps the companion's name and timestamps. The bootstrap key is discarded when
the code expires or the pairing is confirmed, whichever comes first, and one code
confirms at most one pairing. A second device needs a new code.

Mac persistence: `~/Library/Application Support/FlashTeX/pairs.json`, mode
0600, directory 0700, written atomically — `{"version":1,"salt":hex,
"pairs":[{"pair_id","psk","companion_name","created_at","last_seen_at"}]}`.
Not the Keychain (see §6). "Forget" removes the record, restarts the listener
without that key, and closes that companion's live session.

## 3. Transport

TCP + TLS via Network.framework, parameters fixed on both sides:

| Parameter | Value |
|---|---|
| TLS version | 1.2 only (`sec_protocol_options_set_{min,max}_tls_protocol_version(.TLSv12)`) |
| Cipher suite | `TLS_PSK_WITH_AES_128_GCM_SHA256` (0x00A8) only, via `sec_protocol_options_append_tls_ciphersuite(tls_ciphersuite_t(rawValue: 0x00A8))` |
| PSK | `sec_protocol_options_add_pre_shared_key(psk, identity)`; identity = `pair_id` (ASCII); the Mac adds every stored pairing plus the pending bootstrap key |
| Resumption | disabled on both sides (`sec_protocol_options_set_tls_resumption_enabled(false)`, `…_tls_tickets_enabled(false)`) |
| Certificates | none (PSK ciphersuite); no peer trust evaluation |
| TCP | keepalive on, Nagle off, `allowLocalEndpointReuse`, `includePeerToPeer = false` (infrastructure Wi‑Fi/Ethernet only in v1) |

Why TLS 1.2 rather than 1.3: Network.framework's TLS 1.3 PSK support is for
resumption tickets, not externally provisioned PSKs; the 1.2 PSK suite is the
one it negotiates with `add_pre_shared_key`. Measured in `NearbyListenerTests`
(server and client both Network.framework on macOS 26): handshake completes
with the parameters above, and the server verifies
`sec_protocol_metadata_get_negotiated_tls_protocol_version == TLSv12` and the
negotiated suite == 0x00A8 before creating a session; anything else is closed.

Three facts learned while implementing, all mandatory for implementers:

1. **The stack does not report which table PSK a session used.**
   `sec_protocol_metadata_access_pre_shared_keys` returns every configured
   key. Hence `hello.proof` (§4) — the companion proves it holds the key for the
   `pair_id` it claims. Without it a paired device could speak for another
   pairing, or for a pending bootstrap and receive its long-term key.
2. **TLS session resumption bypasses the PSK check.** With resumption on, a
   client that had connected with a since-removed key resumed successfully
   against a listener that no longer held it. Resumption/tickets must be off
   (regression covered by `testBootstrapPairingHandsOverLongTermPSKAndSurvivesRestart`).
3. **A failed handshake still needs `cancel()`.** A server-side
   `NWConnection` that reaches `.failed` (plaintext or wrong-key peer) keeps
   its socket until cancelled; the peer then hangs instead of seeing a close
   (`NearbyPlaintextTests`).

## 4. Framing and messages

After the handshake the stream carries runtime-v1 JSON Lines exactly like the
bridge: one `{protocol_version:1,id,type,payload}` object per line, UTF-8,
`\n`-terminated, **each line including its newline at most 12 MiB** (an
oversized complete or unterminated line gets an `error` with `id: null`, code
`line_too_long`, then the connection closes). Request `id`s are 1–128 bytes;
replies preserve them. Errors carry `{code, message}`.

The first line **must** be `hello`; anything else is answered with
`hello_required` and closed. `hello` is accepted once per connection.

| Request | Payload | Reply |
|---|---|---|
| `hello` | `pair_id`, `companion_name` (≤ 64 chars kept), `protocol_version: 1` (nearby version, not the envelope's), `nonce` (1–128 bytes, fresh per connection), `proof` | `hello_ack` `{mac_name, nonce (echoed), destination, pair_psk?}` |
| `destination_query` | `{}` | `destination` `{destination: … \| null}` |
| `capture_submit` | unchanged runtime-v1 payload | `capture_received` `{capture_id, durable, has_proposal, applied}` or `error` |

`proof` = base64(HMAC-SHA256(key = the PSK used for this connection, data =
`"flashtex-nearby-v1 hello"` ‖ nonce)). The Mac looks the claimed `pair_id` up
in its key table and verifies; a miss or bad MAC → `pair_mismatch`, closed. A
`pair_id:nonce` pair the Mac has seen recently (last 256) → `bad_request`,
closed. `pair_psk` is present only when the connection was authenticated by a
bootstrap key; the companion must store it and use it from the next connection
on (the current connection stays open and usable).

`destination` is `{destination_id, project_id, path, base_revision}` for the
Mac's currently pinned insertion anchor (`Edit > Pin Insertion Point`) or an
explicit `null`. The companion copies these into `capture_submit`; the user
never types IDs or revisions (transfer-v1 requirement).

`capture_submit` validation on the Mac before any sink is called: decodable
envelope, `capture_id`/`destination_id` are 1–128 ASCII `[A-Za-z0-9_-]`,
`mime_type` ∈ {image/png, image/jpeg}, `instructions` ≤ 4096 bytes,
`base_revision` ≥ 0, and the image itself (validated off the listener queue,
before anything reaches the main thread): base64 decodes to 1–8 MiB
(`image_too_large`, same bound as transfer-v1), the bytes are a structurally
complete PNG (signature, IHDR first, chunk CRCs, consecutive IDAT, IEND last,
nothing after it, IDAT inflates to exactly the scanline size with a matching
Adler-32) or JPEG (SOI, one SOF frame header, EOI reached through the scan
data) matching the declared `mime_type`, width and height ≤ 8192 and the
decoded pixel size ≤ 64 MiB (`invalid_image`, the bridge's code for the same
conditions). Rejections are `error` replies that keep the session open. The
bridge still performs its own full decode (transfer-v1); the nearby check is
what lets the Mac refuse a bad image without touching the run loop.

Receive caps (defaults in `NearbyReceiveLimits`; every one is an explicit
error, never a silent drop):

| Cap | Default | Refusal |
|---|---|---|
| frame (line incl. newline) | 12 MiB | `line_too_long`, `id: null`, connection closed |
| per-session frame bytes accepted but not yet acknowledged | 24 MiB | `too_many_in_flight`, session stays open; retry after acks |
| listener-wide accepted-but-unacknowledged bytes ("inbox") | 64 MiB | `inbox_full`, session stays open |
| sessions per `pair_id` | 4 | `too_many_sessions` at `hello`, connection closed |
| accepted TCP connections | 16 | closed before the handshake (no bytes to an unauthenticated peer); Mac logs `too many connections` |

Duplicate handling in the session, keyed by `(session, capture_id)` with the
accepted `base_revision` and a digest of the rest of the payload (last 256
per session): an identical retry is acknowledged again with the *new*
request id and **not re-delivered** (a retry that arrives while the first
delivery is pending is coalesced onto it); the same `capture_id` with another
`base_revision` is refused with `revision_mismatch`; the same id and revision
with another payload is `capture_id_conflict`. A capture the sink refused is
forgotten, so a retry is delivered again. The Mac window surfaces refusals
(`code`, `capture_id`, `pair_id`, message) and the duplicate count
(`NearbyState.lastReceiveError` / `receiveErrors` / `duplicateCaptureCount`).
Retries on a *new* connection after a disconnect are not deduplicated here;
they reach the inbox (identical → acknowledged) or the bridge (its journal
rules), as before.

Error codes used: `bad_request`, `hello_required`, `pair_mismatch`,
`pairing_expired`, `unsupported_version`, `unsupported_image`,
`image_too_large`, `invalid_image`, `unknown_type`, `line_too_long`,
`too_many_in_flight`, `inbox_full`, `too_many_sessions`, `revision_mismatch`,
`capture_id_conflict`, `unavailable`. Codes are additive to the ones listed
before; the nearby `protocol_version` stays 1 (no existing message changed).

Acknowledgement semantics: `durable: true` may only be reported when the local
bridge has journaled the capture (transfer-v1 `capture_received`). With a
bridge attached (`Edit > Attach Capture Bridge`) the Mac forwards the capture
to it and returns the bridge's acknowledgement or error code verbatim; without
one, an in-memory inbox answers `durable: false`. In the inbox, an identical
retry of a known `capture_id` is acknowledged again and a different payload
with a known id is `capture_id_conflict` (the bridge applies its own journal
rules).

## 5. Threat model (honest version)

- **Passive LAN attacker** sees Bonjour, TCP metadata and TLS records; the
  content is AES-128-GCM under a 256-bit PSK once paired. Not protected:
  traffic analysis (a capture is a big write; timing), and the Mac's name/fp.
- **Active LAN attacker / rogue service** cannot complete the handshake without
  a PSK, so no application byte is parsed from an unpaired peer; the listener
  never creates a session before TLS reports `.ready`. A rogue Mac advertising
  the same name gets a failed handshake from a paired companion, not data.
- **The 6-digit code is ~20 bits.** During the ≤120 s window an attacker who
  captures the bootstrap handshake *and* the `hello_ack` can brute-force the
  code offline (PSK-only suites have no forward secrecy) and thereby recover
  `pair_psk`. Mitigations in v1: the window is short, one code confirms one
  pairing, and an online guess costs a full TLS handshake with a fresh code
  needed after expiry. This is the weakest point; the Commander may prefer a
  longer alphanumeric code or a QR code carrying 32 random bytes — the
  derivation and every message are unchanged, only `ikm` grows. `Pairing.derive`
  takes any string.
- **Forward secrecy: none.** Compromise of `pairs.json` (mode 0600, user-only)
  decrypts recorded sessions of that pairing. Rotating to ECDHE-PSK would fix
  this but Network.framework offers no such suite.
- **Replay** of application data across connections is prevented by TLS (fresh
  randoms per handshake, resumption off); the hello nonce additionally binds
  `hello_ack` to its `hello` and refuses a companion re-sending a nonce.
- **Cross-pairing impersonation** by a paired device is prevented by `proof`.
- **Resource exhaustion:** per-line cap 12 MiB, per-read cap 64 KiB, unknown
  types answered without closing; connections, sessions per pairing,
  per-session and listener-wide unacknowledged bytes are capped (§4 table);
  image validation runs on a utility queue so a slow or hostile peer cannot
  stall the listener queue or the main thread. Still unbounded: the in-memory
  inbox keeps its last 50 captures by count, not bytes (up to 50 × 8 MiB).
- **Device loss:** "Forget" on the Mac invalidates the pairing immediately
  (listener restarted without the key, live session closed). There is no remote
  wipe of the companion's copy.

## 6. Not provided in v1

- No certificate PKI, no device identities beyond the PSK, no Keychain storage
  (plain 0600 file; Keychain migration is a follow-up and changes no wire byte).
- No remote or cloud relay; same-LAN only (no AWDL peer-to-peer either).
- No forward secrecy; no TLS 1.3.
- No companion → Mac notification of proposals or insertion results; the
  companion only learns `capture_received`. Proposal review stays on the Mac.
- No multi-Mac routing on the companion beyond "pick a service".
- Local Network privacy: a bundled `FlashTeX.app` will trigger macOS's Local
  Network prompt the first time it advertises; the bare SwiftPM executable and
  `swift test` did not prompt on the development Mac (macOS 26). The .app needs
  `NSLocalNetworkUsageDescription` and `NSBonjourServices = [_flashtex._tcp]`
  in its Info.plist (not yet added to `scripts/make-app.sh`).

## 7. Companion checklist (FT-004)

1. Browse `_flashtex._tcp`; show `name`, match `fp` against stored pairings.
2. Pairing: read `salt`, take the typed code, derive `pair_id`/`psk_boot`
   (CryptoKit HKDF, vector above), connect with `NearbyListener.clientParameters`
   equivalents (TLS 1.2, suite 0x00A8, `add_pre_shared_key(psk_boot, pair_id)`,
   resumption off), send `hello` with a UUID nonce and `proof`, store
   `hello_ack.pair_psk` under `pair_id` + `fp`.
3. Every later connection: same parameters with `pair_psk`; `hello` first;
   read `destination` from `hello_ack` (or `destination_query` before sending).
4. Send `capture_submit` (one image per capture), wait for `capture_received`,
   keep `durable` visible to the user; retry with the *same* `capture_id` and
   payload after a disconnect.
5. Treat any `error` with a closing code as "re-pair or fix input", never retry
   blindly.

## 8. Delta from the current companion (FT-004, `companion-capture` e7ce5b9)

`apps/companion/FlashTeXCompanion/Services/BonjourTransport.swift` today:
browses `_flashtex._tcp.`, connects to the **first** browse result with plain
`NWParameters.tcp`, then after `.ready` sends one `hello` line and
`capture_submit` lines, `\n`-framed, and parses only `capture_received`
(`payload.capture_id`) from the replies. Its hello, as observed:

```json
{"protocol_version":1,"type":"hello","id":"<uuid>","payload":{"role":"companion"}}
```

Against this listener that connection is **refused at the TLS handshake**:
the Mac logs one `closed unauthenticated: handshake failed: …` line, parses
nothing, and keeps serving paired peers (`NearbyPlaintextTests`). The
companion sees the socket close right after its first write, with no JSON
reply. What stays and what changes:

**Keep exactly as is**
- Service type `_flashtex._tcp` (`NWBrowser(for: .bonjour(type:domain:))`).
- `hello` as the first line, newline framing, one JSON object per line,
  runtime-v1 envelope `{protocol_version:1, id, type, payload}`.
- `capture_submit` payload (unchanged runtime-v1; `CapturePayload.swift`).

**Add to `hello.payload`** (`role` may stay; the Mac ignores unknown keys)
- `pair_id` — 16 hex chars from HKDF (§2), or the stored one after pairing.
- `companion_name` — `UIDevice.current.name` (≤ 64 chars kept).
- `protocol_version: 1` — the nearby version; keep the envelope's `1` too.
- `nonce` — a fresh `UUID().uuidString` per connection.
- `proof` — base64 HMAC-SHA256 over `"flashtex-nearby-v1 hello"` ‖ nonce,
  keyed with the PSK this connection was opened with (§4).

**Read from the TXT record** (`NWBrowser.Result.metadata` → `.bonjour(NWTXTRecord)`)
- `salt` (32 hex) — needed only while pairing, to derive the bootstrap key.
- `fp` (16 hex) — the key under which to store the pairing; use it to pick
  the right Mac instead of "first result" (and to show the Mac's `name`).
- `v` — must be `"1"`; otherwise do not connect.

**Replace `NWParameters.tcp`** with TLS-PSK parameters (copy-pasteable, iOS 14+):

```swift
import CryptoKit
import Network
import Security

enum NearbyCrypto {
    static func derive(code: String, saltHex: String) -> (pairId: String, psk: SymmetricKey) {
        let salt = Data(hex: saltHex)                       // 16 bytes from TXT `salt`
        let ikm = SymmetricKey(data: Data(code.utf8))       // the 6 digits the user typed
        let psk = HKDF<SHA256>.deriveKey(inputKeyMaterial: ikm, salt: salt,
                                         info: Data("flashtex-nearby-v1 psk".utf8), outputByteCount: 32)
        let id = HKDF<SHA256>.deriveKey(inputKeyMaterial: ikm, salt: salt,
                                        info: Data("flashtex-nearby-v1 pair-id".utf8), outputByteCount: 8)
        return (id.withUnsafeBytes { Data($0) }.map { String(format: "%02x", $0) }.joined(), psk)
    }

    static func helloProof(psk: SymmetricKey, nonce: String) -> String {
        Data(HMAC<SHA256>.authenticationCode(for: Data("flashtex-nearby-v1 hello".utf8) + Data(nonce.utf8), using: psk))
            .base64EncodedString()
    }

    /// Same parameters the Mac listener uses (NearbyListener.clientParameters).
    static func parameters(pairId: String, psk: SymmetricKey) -> NWParameters {
        let tls = NWProtocolTLS.Options()
        let sec = tls.securityProtocolOptions
        sec_protocol_options_set_min_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_set_max_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_append_tls_ciphersuite(sec, tls_ciphersuite_t(rawValue: 0x00A8)!) // TLS_PSK_WITH_AES_128_GCM_SHA256
        sec_protocol_options_set_tls_resumption_enabled(sec, false)
        sec_protocol_options_set_tls_tickets_enabled(sec, false)
        let key = psk.withUnsafeBytes { DispatchData(bytes: $0) }
        let identity = Data(pairId.utf8).withUnsafeBytes { DispatchData(bytes: $0) }
        sec_protocol_options_add_pre_shared_key(sec, key as __DispatchData, identity as __DispatchData)
        let tcp = NWProtocolTCP.Options()
        tcp.enableKeepalive = true
        tcp.noDelay = true
        let params = NWParameters(tls: tls, tcp: tcp)
        params.allowLocalEndpointReuse = true
        return params
    }
}

// In BonjourTransport.connect(to:), replacing `NWParameters.tcp`:
let (pairId, psk) = stored ?? NearbyCrypto.derive(code: typedCode, saltHex: txt["salt"]!)
let conn = NWConnection(to: endpoint, using: NearbyCrypto.parameters(pairId: pairId, psk: psk))
conn.stateUpdateHandler = { state in
    if case .ready = state {
        let nonce = UUID().uuidString
        let hello: [String: Any] = [
            "protocol_version": 1, "id": UUID().uuidString, "type": "hello",
            "payload": ["role": "companion", "pair_id": pairId, "companion_name": UIDevice.current.name,
                        "protocol_version": 1, "nonce": nonce,
                        "proof": NearbyCrypto.helloProof(psk: psk, nonce: nonce)]]
        // JSONSerialization → append "\n" → conn.send; then start receiving.
    }
}
```

(`Data(hex:)` is a 4-line helper; `stored` is the `{pair_id, pair_psk}` the
app persisted for this Mac's `fp`. Use the Keychain on iOS.)

**Parse these reply lines** (today only `capture_received` is handled):
- `hello_ack` — `payload.mac_name`, `payload.nonce` (must equal the sent
  nonce), `payload.destination` (object or `null`; copy `destination_id`
  and `base_revision` into every `capture_submit`), and, on the pairing
  connection only, `payload.pair_psk` (base64, 32 bytes) — persist it and
  use `SymmetricKey(data:)` of it for every later connection.
- `capture_received` — `payload.capture_id`, `payload.durable`,
  `payload.has_proposal`, `payload.applied` (transfer-v1 shape; `durable`
  is true only when the Mac's bridge journaled it).
- `error` — `id` (may be `null`), `payload.code`, `payload.message`. Codes
  in §4; `pair_mismatch`, `pairing_expired`, `hello_required`,
  `unsupported_version` and `line_too_long` are followed by a close and
  mean "re-pair or fix the client", not "retry".

**Behavioural changes**
- Connect only to a result whose `fp` matches a stored pairing (or the one
  the user picked while pairing), never blindly to the first result.
- One pairing code confirms one device; after `hello_ack.pair_psk` the
  code-derived key is gone. Reconnect with `pair_psk`.
- Retry an unacknowledged capture with the same `capture_id` and payload
  after reconnecting; do not mint a new id for the same image.

## 9. Simulator test (companion in the booted iPad simulator against the Mac)

The iOS simulator shares the Mac's network stack, so Bonjour and TCP work
over the Mac's own interfaces (loopback included) with no extra setup.

1. Mac: `cd apps/mac && swift build && FLASHTEX_REPO=$(git rev-parse --show-toplevel) .build/debug/FlashTeXMac`,
   then `Edit > Nearby Companion…` (⌘⇧N) → **Advertise** on → **Show Pairing
   Code**. The window shows the Bonjour name, port, `fp`, and the code with
   its countdown. (Attach the bridge first, ⌘⇧U menu group, if you want
   `durable: true` acknowledgements; otherwise the inbox answers
   `durable: false`.)
2. Verify the advertisement from a terminal:
   `dns-sd -B _flashtex._tcp local.` then
   `dns-sd -L "<Mac name>" _flashtex._tcp local.` — the TXT line must show
   `v=1 name=… fp=… salt=…`.
3. Companion: `xcrun simctl boot "<iPad>"` (or from Xcode), build and run
   `apps/companion` on it (`apps/companion/build.sh` / the Xcode scheme).
   Bonjour browsing inside the simulator sees the Mac's service; enter the
   6-digit code from step 1 when prompted (after the FT-004 changes in §8).
4. Expected on the Mac window: "paired <device> (<pair_id>)", the device
   listed under *Paired companions* with a green "connected" badge, and the
   countdown gone. Expected on the companion: `hello_ack` with
   `pair_psk` and the current `destination` (pin one first with ⌘⇧P).
5. Send a capture from the companion; the Mac window's *Received captures*
   shows its `capture_id` and, with the bridge attached, the capture appears
   in the bridge list ready for **Convert Capture** (⌘⇧G).
6. Negative check with the *current* companion (before §8 is implemented):
   it connects with plain TCP, the Mac's *Activity* log shows one
   `closed unauthenticated: handshake failed: …` line and the companion's
   connection drops; nothing is acknowledged.

Without the companion, the same path is exercised by
`swift test --filter NearbyStateTests` (a Network.framework client in the
test process pairs, sends the fixture capture, is forgotten, and is refused).

`swift test --filter NearbyTranscriptAcceptanceTests` replays
`apps/mac/Tests/FlashTeXMacTests/Fixtures/nearby-companion-session.jsonl` on
loopback: a recorded-shape companion session (the simulator run's envelope
ordering, `\/` escaping, `capture-<hex>` ids, 400×300 photo and 1408×1510
RGBA pencil PNGs, each capture line emitted twice as that build did) with a
§8 `hello`. Expected: `hello_ack`, four `capture_received` (the two
duplicates acknowledged, not re-delivered), two captures in the inbox; a
verbatim replay on a second connection is refused at `hello` (stale nonce);
a reconnect with a fresh `hello` re-sending the same captures is
acknowledged again without storing them twice. The original simulator
stdout was not committed and the companion still speaks plaintext, so this
fixture is re-synthesized to the recorded shape, not the original bytes.
