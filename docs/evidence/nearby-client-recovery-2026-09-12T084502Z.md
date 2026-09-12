# nearby-client native recovery measurement — 2026-09-12T08:45:02Z

Commit: `6ee016325459c6c19e3d7d524f6965b9f7a9e895` (apps/mac dirty); macOS 26.3.1; Apple M1 Max; `Apple Swift version 6.2.4 (swiftlang-6.2.4.1.4 clang-1700.6.4.2)`.
Mac side: `NearbyReferenceClientTests.testServeForExternalClient` (real `NearbyState` + `NearbyListener`, loopback only).
Client side: `.build/debug/nearby-client` run as a separate process (this script).
Numbers are wall-clock seconds for whole CLI invocations (process start → exit), one machine, loopback.

- built `nearby-client` (debug) in 2.6s
- built apps/mac tests in 2.9s

Serving as `FlashTeX Native 98778` fp `b08d73b4acad0baa` on port 50077 (loopback_only=True), pinned destination `mac-anchor-1` @ rev 1; serve info available 3.26s after launching the test runner.

| step | exit | expected | seconds | at +t | verdict |
|---|---|---|---|---|---|
| pair via Bonjour (`--mac <fp>`), bootstrap TLS, hello → pair_psk, destination_query | 0 | 0 | 0.747 | +0.0 | ok |
| send #1 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 1.035 | +0.7 | ok |
| send #2 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.041 | +1.8 | ok |
| send #3 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.039 | +1.8 | ok |
| send #4 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.042 | +1.9 | ok |
| send #5 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.043 | +1.9 | ok |
| send #1 direct `--host 127.0.0.1 --port 50077` (no browse) | 0 | 0 | 0.024 | +1.9 | ok |
| send #2 direct `--host 127.0.0.1 --port 50077` (no browse) | 0 | 0 | 0.023 | +2.0 | ok |
| send #3 direct `--host 127.0.0.1 --port 50077` (no browse) | 0 | 0 | 0.022 | +2.0 | ok |
| send #4 direct `--host 127.0.0.1 --port 50077` (no browse) | 0 | 0 | 0.031 | +2.0 | ok |
| send #5 direct `--host 127.0.0.1 --port 50077` (no browse) | 0 | 0 | 0.028 | +2.0 | ok |
| send: identical retry of native-direct-1 (Mac de-duplicates, acknowledged again) | 0 | 0 | 0.021 | +2.1 | ok |
| send: same capture_id, different payload → capture_id_conflict (terminal, exit 2) | 2 | 2 | 0.019 | +2.1 | ok |
| send during a 3s listener outage (port released, new ephemeral port after): `--attempts 8 --retry-delay 0.5 --max-delay 2 --seconds 2`, re-browse by fp before every attempt | 0 | 0 | 3.380 | +18.4 | ok |
| send after the Mac forgot the pairing (`--attempts 5`): refused at the TLS handshake, no retry, exit 3 | 3 | 3 | 0.478 | +34.5 | ok |
| send with the Mac gone, direct port: `--attempts 3 --retry-delay 0.2` → unreachable ×3, exit 4 | 4 | 4 | 0.596 | +47.9 | ok |
| send with the Mac gone, Bonjour: `--attempts 2 --seconds 1 --retry-delay 0` → no matching Mac ×2, exit 4 | 4 | 4 | 2.154 | +48.4 | ok |

## Outage recovery transcript (client)

```
browsing _flashtex._tcp for fp b08d73b4acad0baa (up to 2s)…
attempt 1 failed: no matching Mac: no _flashtex._tcp service with fp b08d73b4acad0baa within 2.0s (seen: none); retrying in 0.44s
attempt 2 of 8
browsing _flashtex._tcp for fp b08d73b4acad0baa (up to 2s)…
found FlashTeX Native 98778 (fp b08d73b4acad0baa, v=1)
hello_ack from "FlashTeX Native 98778" (attempt 2); destination: mac-anchor-1 (demo/main.tex @ rev 1)
capture_received native-outage-1: durable=false has_proposal=false applied=false after 2 connection attempts (same capture_id re-sent)
```

## Revoked pairing transcript (client)

```
error: TLS-PSK handshake failed: -9824: handshake failure
hint: the pairing was refused — the pairing code expired/was used, the Mac forgot this pairing, or this is not a FlashTeX nearby listener; run `nearby-client pair` again (exit 3)
```

## Mac gone transcripts (client)

```
attempt 1 failed: Mac unreachable: POSIXErrorCode(rawValue: 61): Connection refused; retrying in 0.18s
attempt 2 of 3
attempt 2 failed: Mac unreachable: POSIXErrorCode(rawValue: 61): Connection refused; retrying in 0.39s
attempt 3 of 3
error: gave up after 3 attempts; last failure: Mac unreachable: POSIXErrorCode(rawValue: 61): Connection refused
hint: the Mac stayed unreachable for the whole retry budget; check it is advertising and try again (exit 4)
---
attempt 1 failed: no matching Mac: no _flashtex._tcp service with fp b08d73b4acad0baa within 1.0s (seen: none); retrying in 0.00s
attempt 2 of 2
error: gave up after 2 attempts; last failure: no matching Mac: no _flashtex._tcp service with fp b08d73b4acad0baa within 1.0s (seen: none)
hint: the Mac stayed unreachable for the whole retry budget; check it is advertising and try again (exit 4)
```

## Mac-side log (NearbyState, timestamps relative to serve start)

```
mac: +0.000s ready on port 50077
mac: +0.000s pairing code issued for 0ead4eb626a27ff3
mac: +0.000s ready on port 50077
mac: +0.982s authenticated connection
mac: +1.091s stored pairing 0ead4eb626a27ff3 for Native CLI
mac: +1.091s paired Native CLI (0ead4eb626a27ff3)
mac: +1.091s ready on port 50077
mac: +1.091s closed 0ead4eb626a27ff3: peer closed
mac: +2.058s authenticated connection
mac: +2.058s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.058s capture native-bonjour-1
mac: +2.058s closed 0ead4eb626a27ff3: peer closed
mac: +2.166s authenticated connection
mac: +2.166s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.166s capture native-bonjour-2
mac: +2.166s closed 0ead4eb626a27ff3: peer closed
mac: +2.166s authenticated connection
mac: +2.166s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.166s capture native-bonjour-3
mac: +2.166s closed 0ead4eb626a27ff3: peer closed
mac: +2.166s authenticated connection
mac: +2.166s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.166s capture native-bonjour-4
mac: +2.166s closed 0ead4eb626a27ff3: peer closed
mac: +2.277s authenticated connection
mac: +2.277s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.277s capture native-bonjour-5
mac: +2.277s closed 0ead4eb626a27ff3: peer closed
mac: +2.277s authenticated connection
mac: +2.277s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.277s capture native-direct-1
mac: +2.277s closed 0ead4eb626a27ff3: peer closed
mac: +2.277s authenticated connection
mac: +2.277s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.277s capture native-direct-2
mac: +2.277s closed 0ead4eb626a27ff3: peer closed
mac: +2.277s authenticated connection
mac: +2.277s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.277s capture native-direct-3
mac: +2.277s closed 0ead4eb626a27ff3: peer closed
mac: +2.383s authenticated connection
mac: +2.383s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.383s capture native-direct-4
mac: +2.383s closed 0ead4eb626a27ff3: peer closed
mac: +2.383s authenticated connection
mac: +2.383s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.383s capture native-direct-5
mac: +2.383s closed 0ead4eb626a27ff3: peer closed
mac: +2.383s authenticated connection
mac: +2.383s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.383s capture native-direct-1
mac: +2.383s closed 0ead4eb626a27ff3: peer closed
mac: +2.383s authenticated connection
mac: +2.383s hello from Native CLI (0ead4eb626a27ff3)
mac: +2.383s capture native-direct-1
mac: +2.383s closed 0ead4eb626a27ff3: peer closed
mac: +18.010s outage: stopped advertising (listener gone, port released)
mac: +18.010s stopped
mac: +21.102s outage: advertising again
mac: +21.212s ready on port 50126
mac: +22.062s authenticated connection
mac: +22.062s hello from Native CLI (0ead4eb626a27ff3)
mac: +22.062s capture native-outage-1
mac: +22.062s closed 0ead4eb626a27ff3: peer closed
mac: +34.089s revoked: forgot ["0ead4eb626a27ff3"]
mac: +34.090s forgot 0ead4eb626a27ff3
mac: +34.199s ready on port 50126
mac: +35.257s closed unauthenticated: handshake failed: -9858: handshake failed
mac: served 48s; pairs=[]; inbox=["native-bonjour-1", "native-bonjour-2", "native-bonjour-3", "native-bonjour-4", "native-bonjour-5", "native-direct-1", "native-direct-2", "native-direct-3", "native-direct-4", "native-direct-5", "native-outage-1"]
```

## Serve harness result

```
	 Executed 1 test, with 0 failures (0 unexpected) in 48.100 (48.101) seconds
	 Executed 1 test, with 0 failures (0 unexpected) in 48.100 (48.101) seconds
	 Executed 1 test, with 0 failures (0 unexpected) in 48.100 (48.104) seconds
```

## Verdict

17/17 steps exited as expected.
