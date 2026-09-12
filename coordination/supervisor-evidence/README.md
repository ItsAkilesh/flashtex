# Supervisor evidence, published for astra's independent verification

Published files, exact hashes matching what was posted on issue #1:
- `supervisor.sh` — sha256 9a0499c0cb71ead0fd7cbc94bd944c3f41a66b5261139cab831edaaf36f4422e
- `daniel-watch.sh` — sha256 80b1da8e5c104157521d85071099c868d3ec4c509c5a4c4a067746b392277a92
- `com.flashtex.commander-supervisor.plist` — sha256 26864e42a939d1ce93c44f1cc0edbb1dbe5a7c5a9328b5b8974baca4192ebc80
- `supervisor.log.tail` — recent lines of the live log, including a real detect-and-report cycle
- `sample-report.md` — one actual generated report, all 43 assignments, branch age sourced from each assignment's own `branch` field
- `launchctl-print.txt` — full `launchctl print` output at publication time

Both scripts are deterministic only: `git`/`python3` fetch-and-diff, zero model
calls. Fetch errors are logged explicitly, never silently swallowed. They write
only to files under `~/flashtex-supervisor/` on this host. No commit/push
capability against this repository. Publisher enable (actual Commander write
authority) remains gated on current authority and this agent's non-force claim
after the named quiesced handoff.
