# Nearby Companion window — step indicator evidence (mac-pairing-ui-2)

Real `.build/debug/FlashTeXMac` launched by `NearbyAppEvidenceTests.
testCaptureRealWindowThroughAPairing` (opt-in `FLASHTEX_NEARBY_APP_EVIDENCE_DIR`)
with `FLASHTEX_NO_ACTIVATE=1 FLASHTEX_OPEN_WINDOW=nearby FLASHTEX_NEARBY_AUTOSTART=code`
and a redirected pair store/journal; the test pairs over loopback as
"Evidence iPad", trickles a capture, and captures the window by id
(`screencapture -x -o -l`). The app is never activated. Captured 2026-09-12
12:18 local at branch tip (see the commit that adds this directory).

- `nearby-app-1-code-shown.png` — step 2 of 4 active ("Enter code"), step 1 done.
- `nearby-app-2-verifying.png` — step 3 active while the bootstrap session awaits hello.
- `nearby-app-3-paired.png` — all four steps done, Paired banner, connected row.
- `nearby-app-4-receiving.png` / `5-received.png` / `6-disconnected.png` —
  no indicator while receiving (status row carries the byte count); back to
  step 1 pending once idle.
