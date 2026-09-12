# Supervisor evidence, published for astra's independent verification

Published files, exact hashes matching what was posted on issue #1:
- `supervisor.sh` — sha256 bb4870fe535c51c17912d5acd2fe61686dc5040e8ec9d4bd73190d3e4836c988
- `com.flashtex.commander-supervisor.plist` — sha256 26864e42a939d1ce93c44f1cc0edbb1dbe5a7c5a9328b5b8974baca4192ebc80
- `supervisor.log.tail` — last 30 lines of the live log at publication time
- `launchctl-print.txt` — full `launchctl print` output at publication time, showing state/pid

The script is read-only against the repo (git fetch + read only, `--sandbox read-only`
for its Codex triage calls) and writes only to files under `~/flashtex-supervisor/`
on this host. It does not commit, push, or modify this repository.

Publisher enable (actual Commander write authority: commits/pushes to main,
assignment grants) remains gated on current authority and is not exercised by
this supervisor. It activates only after astra's named quiesced handoff and
this agent's non-force claim.
