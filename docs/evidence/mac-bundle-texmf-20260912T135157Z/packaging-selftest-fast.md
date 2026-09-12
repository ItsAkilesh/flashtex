# packaging-selftest.sh — 2026-09-12T13:54:06Z

Mode: fast; make-app.sh args: (none)

## Pre-flight failure paths (no build)

- ok: make-app.sh --sign Developer ID Application: FlashTeX Selftest (SELFTEST00) -> exit 1, pre-build, message: make-app.sh: --sign: no valid codesigning identity matching "Developer ID Application: FlashTeX Selftest (SELFTEST00)" i…
- ok: make-app.sh --notarize flashtex-selftest-profile -> exit 1, pre-build, message: make-app.sh: --notarize requires --sign <Developer ID Application identity>: Apple only notarizes Developer ID-signed, h…
- ok: make-app.sh --sign - --notarize flashtex-selftest-profile -> exit 1, pre-build, message: ==> Signing identity: ad-hoc (-) with hardened runtime and entitlements (local test of the distribution path) make-app.s…
- ok: make-app.sh --sign - --entitlements /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-selftest.Xpq1nE/missing.entitlements -> exit 1, pre-build, message: make-app.sh: --sign: entitlements file not found: /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-selftest.Xp…
- ok: make-app.sh --sign - --entitlements /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-selftest.Xpq1nE/bad.entitlements -> exit 1, pre-build, message: make-app.sh: --sign: entitlements file is not a valid plist: /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-…
- ok: no Developer ID on this machine: identity check fires before the profile check; direct probe 'security find-generic-password -s com.apple.gke.notary.tool -a flashtex-selftest-profile' exits 44 (non-zero = absent)
- ok: unknown argument -> exit 1

## Pinned rooted TFM metrics (GH36; no build, no download, no host TeX)

- ok: bundle-texmf.py check apps/mac/Fonts/texmf: 29 entries verified (Commander manifest + SUPPLEMENTARY-METRICS.json pin)
- ok: make-app.sh --debug -> exit 1, pre-build, message:       "status": "refused",           "path": "fonts/tfm/public/lm/ec-lmr10.tfm",           "status": "verified",        …
- ok: make-app.sh --debug -> exit 1, pre-build, message:       "status": "refused",           "path": "fonts/tfm/public/lm/ec-lmr10.tfm",           "status": "missing",         …
- ok: make-app.sh --debug -> exit 1, pre-build, message:       "status": "refused",           "path": "fonts/tfm/public/lm/ec-lmr10.tfm",           "status": "verified",        …
- ok: make-app.sh --debug -> exit 1, pre-build, message: make-app.sh: pinned bundle metrics root not found: /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-selftest.X…

## Resources

- ok: plutil -lint Resources/Info.plist.template
- ok: plutil -lint Resources/FlashTeX.entitlements
- ok: Info.plist.template carries @VERSION@ and @GIT_SHA@ placeholders
- ok: FlashTeX.entitlements grants no entitlements (empty dict)

## --help

- ok: make-app.sh --help
- ok: launch-check.sh --help
- ok: repro-check.sh --help
- ok: texmf-acceptance.sh --help
- ok: make-app.sh parses under /bin/bash 3.2
- ok: launch-check.sh parses under /bin/bash 3.2
- ok: repro-check.sh parses under /bin/bash 3.2
- ok: packaging-selftest.sh parses under /bin/bash 3.2
- ok: texmf-acceptance.sh parses under /bin/bash 3.2

Result: 25 ok, 0 failed
