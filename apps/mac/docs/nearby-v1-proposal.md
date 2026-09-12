# Nearby companion transport v1 — proposal

Status: **proposal** from FT-003 (Mac shell, mac-claude-a) for the Commander to
adopt as `docs/contracts/nearby-v1.md`. Until then nothing here is a published
contract; the Mac implementation in `apps/mac/Sources/FlashTeXMac/Nearby*.swift`
and `Pairing.swift` is the reference for the Mac side only. The companion side
belongs to FT-004. Updated September 12, 2026.

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

Two facts learned while implementing, both mandatory for implementers:

1. **The stack does not report which table PSK a session used.**
   `sec_protocol_metadata_access_pre_shared_keys` returns every configured
   key. Hence `hello.proof` (§4) — the companion proves it holds the key for the
   `pair_id` it claims. Without it a paired device could speak for another
   pairing, or for a pending bootstrap and receive its long-term key.
2. **TLS session resumption bypasses the PSK check.** With resumption on, a
   client that had connected with a since-removed key resumed successfully
   against a listener that no longer held it. Resumption/tickets must be off
   (regression covered by `testBootstrapPairingHandsOverLongTermPSKAndSurvivesRestart`).

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
`mime_type` ∈ {image/png, image/jpeg}, `instructions` ≤ 4096 bytes. Rejections
are `error` replies that keep the session open. Image decoding, the 8 MiB /
8192×8192 limits and durability are the bridge's job (transfer-v1); the nearby
adapter does not re-implement them.

Error codes used: `bad_request`, `hello_required`, `pair_mismatch`,
`pairing_expired`, `unsupported_version`, `unsupported_image`, `unknown_type`,
`line_too_long`, `capture_id_conflict`, `unavailable`.

Acknowledgement semantics: `durable: true` may only be reported when the local
bridge has journaled the capture (transfer-v1 `capture_received`). The Mac's
current sink is an in-memory inbox and answers `durable: false`; the bridge
client replaces it and forwards the bridge's actual acknowledgement. An identical
retry of a known `capture_id` is acknowledged again; a different payload with a
known id is `capture_id_conflict`.

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
  types answered without closing; no limit yet on concurrent connections or on
  the inbox beyond its last 50 captures (in memory).
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
