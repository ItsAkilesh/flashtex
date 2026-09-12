# nearby-client native recovery measurement — 2026-09-12T09:28:14Z

Commit: `d1da72cc74343cdba2d02430eb02bb1c884e6521`; macOS 26.3.1; Apple M1 Max; `Apple Swift version 6.2.4 (swiftlang-6.2.4.1.4 clang-1700.6.4.2)`.
Mac side: `NearbyReferenceClientTests.testServeForExternalClient` (real `NearbyState` + `NearbyListener`, loopback only).
Client side: `.build/debug/nearby-client` run as a separate process (this script).
Numbers are wall-clock seconds for whole CLI invocations (process start → exit), one machine, loopback.

- built `nearby-client` (debug) in 0.7s
- built apps/mac tests in 0.7s

Serving as `FlashTeX Native 86148` fp `6c12e28b90d80b3e` on port 51239 (loopback_only=True), pinned destination `mac-anchor-1` @ rev 1; serve info available 2.04s after launching the test runner.

| step | exit | expected | seconds | at +t | verdict |
|---|---|---|---|---|---|
| pair via Bonjour (`--mac <fp>`), bootstrap TLS, hello → pair_psk, destination_query | 0 | 0 | 0.762 | +0.0 | ok |
| send #1 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.970 | +0.8 | ok |
| send #2 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.047 | +1.7 | ok |
| send #3 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.034 | +1.8 | ok |
| send #4 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.039 | +1.8 | ok |
| send #5 via Bonjour (browse by fp, reconnect with pair_psk, capture_submit → capture_received) | 0 | 0 | 0.035 | +1.9 | ok |
| send #1 direct `--host 127.0.0.1 --port 51239` (no browse) | 0 | 0 | 0.017 | +1.9 | ok |
| send #2 direct `--host 127.0.0.1 --port 51239` (no browse) | 0 | 0 | 0.018 | +1.9 | ok |
| send #3 direct `--host 127.0.0.1 --port 51239` (no browse) | 0 | 0 | 0.017 | +1.9 | ok |
| send #4 direct `--host 127.0.0.1 --port 51239` (no browse) | 0 | 0 | 0.017 | +1.9 | ok |
| send #5 direct `--host 127.0.0.1 --port 51239` (no browse) | 0 | 0 | 0.018 | +2.0 | ok |
| send: identical retry of native-direct-1 (Mac de-duplicates, acknowledged again) | 0 | 0 | 0.017 | +2.0 | ok |
| send: same capture_id, different payload → capture_id_conflict (terminal, exit 2) | 2 | 2 | 0.016 | +2.0 | ok |
| send during a 3s listener outage (port released, new ephemeral port after): `--attempts 8 --retry-delay 0.5 --max-delay 2 --seconds 2`, re-browse by fp before every attempt | 0 | 0 | 3.418 | +18.4 | ok |
| send after the Mac forgot the pairing (`--attempts 5`): refused at the TLS handshake, no retry, exit 3 | 3 | 3 | 0.283 | +34.5 | ok |
| send with the Mac gone, direct port: `--attempts 3 --retry-delay 0.2` → unreachable ×3, exit 4 | 4 | 4 | 0.629 | +47.9 | ok |
| send with the Mac gone, Bonjour: `--attempts 2 --seconds 1 --retry-delay 0` → no matching Mac ×2, exit 4 | 4 | 4 | 2.151 | +48.5 | ok |

## Outage recovery transcript (client)

```
browsing _flashtex._tcp for fp 6c12e28b90d80b3e (up to 2s)…
attempt 1 failed: no matching Mac: no _flashtex._tcp service with fp 6c12e28b90d80b3e within 2.0s (seen: none); retrying in 0.54s
attempt 2 of 8
browsing _flashtex._tcp for fp 6c12e28b90d80b3e (up to 2s)…
found FlashTeX Native 86148 (fp 6c12e28b90d80b3e, v=1)
hello_ack from "FlashTeX Native 86148" (attempt 2); destination: mac-anchor-1 (demo/main.tex @ rev 1)
capture_received native-outage-1: durable=false has_proposal=false applied=false after 2 connection attempts (same capture_id re-sent)
```

## Revoked pairing transcript (client)

```
error: TLS-PSK handshake failed: -9824: handshake failure
hint: the pairing was refused — the pairing code expired/was used, the Mac forgot this pairing, or this is not a FlashTeX nearby listener; run `nearby-client pair` again (exit 3)
```

## Mac gone transcripts (client)

```
attempt 1 failed: Mac unreachable: POSIXErrorCode(rawValue: 61): Connection refused; retrying in 0.24s
attempt 2 of 3
attempt 2 failed: Mac unreachable: POSIXErrorCode(rawValue: 61): Connection refused; retrying in 0.35s
attempt 3 of 3
error: gave up after 3 attempts; last failure: Mac unreachable: POSIXErrorCode(rawValue: 61): Connection refused
hint: the Mac stayed unreachable for the whole retry budget; check it is advertising and try again (exit 4)
---
attempt 1 failed: no matching Mac: no _flashtex._tcp service with fp 6c12e28b90d80b3e within 1.0s (seen: none); retrying in 0.00s
attempt 2 of 2
error: gave up after 2 attempts; last failure: no matching Mac: no _flashtex._tcp service with fp 6c12e28b90d80b3e within 1.0s (seen: none)
hint: the Mac stayed unreachable for the whole retry budget; check it is advertising and try again (exit 4)
```

## Mac-side log (NearbyState, timestamps relative to serve start)

```
mac: +0.000s ready on port 51239
mac: +0.000s pairing code issued for 3ccdaad8fdf9b6aa
mac: +0.000s ready on port 51239
mac: +1.066s authenticated connection
mac: +1.066s stored pairing 3ccdaad8fdf9b6aa for Native CLI
mac: +1.066s paired Native CLI (3ccdaad8fdf9b6aa)
mac: +1.066s ready on port 51239
mac: +1.066s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.034s authenticated connection
mac: +2.034s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.034s capture native-bonjour-1
mac: +2.034s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.034s authenticated connection
mac: +2.034s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.034s capture native-bonjour-2
mac: +2.034s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.135s authenticated connection
mac: +2.135s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.135s capture native-bonjour-3
mac: +2.135s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.135s authenticated connection
mac: +2.135s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.135s capture native-bonjour-4
mac: +2.135s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.135s authenticated connection
mac: +2.135s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.237s capture native-bonjour-5
mac: +2.237s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.237s authenticated connection
mac: +2.237s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.237s capture native-direct-1
mac: +2.237s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.237s authenticated connection
mac: +2.237s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.237s capture native-direct-2
mac: +2.237s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.237s authenticated connection
mac: +2.237s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.237s capture native-direct-3
mac: +2.237s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.237s authenticated connection
mac: +2.237s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.237s capture native-direct-4
mac: +2.237s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.237s authenticated connection
mac: +2.237s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.237s capture native-direct-5
mac: +2.237s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.237s authenticated connection
mac: +2.348s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.348s capture native-direct-1
mac: +2.348s closed 3ccdaad8fdf9b6aa: peer closed
mac: +2.348s authenticated connection
mac: +2.348s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +2.348s refused capture_id_conflict: native-direct-1 from 3ccdaad8fdf9b6aa — capture_id native-direct-1 already received with a different payload
mac: +2.348s closed 3ccdaad8fdf9b6aa: peer closed
mac: +18.048s outage: stopped advertising (listener gone, port released)
mac: +18.048s stopped
mac: +21.022s outage: advertising again
mac: +21.127s ready on port 51263
mac: +22.091s authenticated connection
mac: +22.091s hello from Native CLI (3ccdaad8fdf9b6aa)
mac: +22.091s capture native-outage-1
mac: +22.091s closed 3ccdaad8fdf9b6aa: peer closed
mac: +34.022s revoked: forgot ["3ccdaad8fdf9b6aa"]
mac: +34.022s forgot 3ccdaad8fdf9b6aa
mac: +34.131s ready on port 51263
mac: +35.093s closed unauthenticated: handshake failed: -9858: handshake failed
mac: served 48s; pairs=[]; inbox=["native-bonjour-1", "native-bonjour-2", "native-bonjour-3", "native-bonjour-4", "native-bonjour-5", "native-direct-1", "native-direct-2", "native-direct-3", "native-direct-4", "native-direct-5", "native-outage-1"]
```

## Serve harness result

```
	 Executed 1 test, with 0 failures (0 unexpected) in 48.140 (48.140) seconds
	 Executed 1 test, with 0 failures (0 unexpected) in 48.140 (48.140) seconds
	 Executed 1 test, with 0 failures (0 unexpected) in 48.140 (48.142) seconds
```

## Verdict

17/17 steps exited as expected.
