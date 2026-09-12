# Nearby multi-interface LAN measurements — 2026-09-12 (mac-nearby-transport-3)

Machine: mac-m1max-a (user's M1 Max), macOS 26 (Darwin 25.3.0), Xcode 26.3 / Swift 6.2.4.
Branch `agent/mac-nearby-transport-3/lan-advertise` @ c1167c8f (from mac-shell 82749c26).
Command: `cd apps/mac && swift test --skip-build --filter 'Nearby|Pairing|Capture'` (load 15.7 / 18.1 / 23.7 at start).

No second device: the client dials the Mac's own `en*` addresses, so packets are delivered inside the
stack. What is measured is binding on every interface, address/interface scoping, Bonjour registration
and per-interface resolution, key/fingerprint refusal and the frame deadline on those paths — not
wire latency between two machines.

Active non-loopback interfaces at run time (getifaddrs, `en*` only; `utun0–5`, `awdl0`, `llw0`,
`bridge0` excluded): `en0` (wifi) IPv4 172.26.0.157 + link-local IPv6; `en9` (wiredEthernet) IPv4
169.254.173.203 (self-assigned) + link-local IPv6; `en11` (wiredEthernet) link-local IPv6 only.
`NWPathMonitor.availableInterfaces` listed `en0` only, so the client was pinned with
`requiredInterface` on `en0` (`scoped=true`) and dialled the scoped literal address on `en9`/`en11`
(`scoped=false`; the `%en9`/`%en11` zone still selects the interface — see the remote endpoints).

## `measured:` lines (host name redacted)

    measured: FlashTeX LAN Bonjour 979E32 advertised on ["en11/wiredEthernet", "lo0/loopback", "en0/wifi", "en9/wiredEthernet"]
    measured: foreign fp refused: no _flashtex._tcp service with fp 0000000000000000 within 4.0s (seen: FlashTeX LAN fp 4DFC8C fp=b0fc34e7719575b2)
    measured: frame timeout on ipv4 169.254.173.203 on en9: closed after 0.512 s (deadline 0.5 s)
    measured: frame timeout on ipv4 172.26.0.157 on en0: closed after 0.505 s (deadline 0.5 s)
    measured: interfaces known to NWPathMonitor ["en0"]; LAN addresses [ipv4 172.26.0.157 on en0, ipv6-link-local fe80::1cb7:cf63:4020:a044%en0 on en0, ipv6-link-local fe80::1871:a0ff:fe99:78c6%en11 on en11, ipv4 169.254.173.203 on en9, ipv6-link-local fe80::82f:598a:7574:62ec%en9 on en9]
    measured: ipv4 169.254.173.203 on en9: scoped=false remote=Optional(169.254.173.203:55604) local=Optional(169.254.173.203:61027)
    measured: ipv4 172.26.0.157 on en0: scoped=true remote=Optional(172.26.0.157:55604) local=Optional(172.26.0.157:61024)
    measured: ipv6-link-local fe80::1871:a0ff:fe99:78c6%en11 on en11: scoped=false remote=Optional(fe80::1871:a0ff:fe99:78c6%en11.55604) local=Optional(fe80::1871:a0ff:fe99:78c6%en11.61026)
    measured: ipv6-link-local fe80::1cb7:cf63:4020:a044%en0 on en0: scoped=true remote=Optional(fe80::1cb7:cf63:4020:a044%en0.55604) local=Optional(fe80::1cb7:cf63:4020:a044%en0.61025)
    measured: ipv6-link-local fe80::82f:598a:7574:62ec%en9 on en9: scoped=false remote=Optional(fe80::82f:598a:7574:62ec%en9.55604) local=Optional(fe80::82f:598a:7574:62ec%en9.61028)
    measured: reconnect ipv4 172.26.0.157 on en0 -> ipv6-link-local fe80::1871:a0ff:fe99:78c6%en11 on en11: retry acknowledged from memory, sink saw 1 then 2
    measured: resolve on en0#14 -> Optional((host: "<mac-hostname>.local", port: 55605))
    measured: resolve on en11#25 -> Optional((host: "<mac-hostname>.local", port: 55605))
    measured: resolve on en9#26 -> Optional((host: "<mac-hostname>.local", port: 55605))
    measured: wrong key on ipv4 169.254.173.203 on en9 -> Optional(-9820: bad MAC)
    measured: wrong key on ipv4 172.26.0.157 on en0 -> Optional(-9820: bad MAC)
    measured: wrong key on ipv6-link-local fe80::1871:a0ff:fe99:78c6%en11 on en11 -> Optional(-9820: bad MAC)
    measured: wrong key on ipv6-link-local fe80::1cb7:cf63:4020:a044%en0 on en0 -> Optional(-9820: bad MAC)
    measured: wrong key on ipv6-link-local fe80::82f:598a:7574:62ec%en9 on en9 -> Optional(-9820: bad MAC)

## Result

`NearbyLANInterfaceTests` 5/5, `NearbyAckPersistenceTests` 4/4. The whole `Nearby|Pairing|Capture`
filter: 146 tests, 0 failures, 8 env-gated skips without helpers; rerun with the real
`flashtex-bridge`/`flashtex-compiler`/`flashtex-preview-controller` binaries: 146 tests, 0 failures,
2 opt-in skips (`FLASHTEX_NEARBY_APP_EVIDENCE_DIR`, `FLASHTEX_NEARBY_SERVE_INFO`), 1-min load 14.9.
Frame deadline on the `en0`/`en9` paths: 0.505–0.512 s after the first partial byte at a 0.5 s
deadline (0.508–0.510 s in the first run). A `feth`/`bridge` pair was not needed: three real `en*`
links were active, so no privileged interface creation was attempted.
