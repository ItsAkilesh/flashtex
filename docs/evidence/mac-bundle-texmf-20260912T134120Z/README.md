# texmf-acceptance.sh — 2026-09-12T13:42:00Z

Host TeX excluded by env -i (PATH=/usr/bin:/bin, empty HOME, no FLASHTEX_*/TEXMF*) and a sandbox-exec profile denying reads under /usr/local/texlive, /Library/TeX, /usr/share/texmf, /usr/share/texlive.

- app: /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ac12dd0004019340d/apps/mac/build/FlashTeX.app
- producer sha256: 59ca1ad82067eaf3f212976ce77ac910ad5be9a3d8a34b59091c27f6a1bb73f6
- uptime:  9:41  up 1 day,  8:34, 2 users, load averages: 5.59 23.53 23.22
- resource sha256 cd13479f463b9a575d053dd7bf0884daa46bfdeffe4b7f537c193861652ac9e5 Contents/Resources/texmf/fonts/tfm/public/lm/ec-lmr10.tfm
- resource sha256 299021120f0a29ef61278a2363903bd8defbb8faaade458eb79067342aecb56f Contents/Resources/texmf/fonts/tfm/public/lm/ec-lmr12.tfm
- resource sha256 9d4e3d8e39a41b93d91f79c1c47d2297efb7b1af220b94860693c08361f227aa Contents/Resources/texmf/fonts/tfm/public/lm/rm-lmr12.tfm
- resource sha256 eb0bfdf8db3ae1409639fac9c88f84923872500d882d9ff8dc37aff445c723fe Contents/Resources/texmf/fonts/tfm/public/lm/rm-lmr6.tfm
- resource sha256 80bcbfd844d2310ac1d3bead45aee25e91b1a4a0a60ff1771959b9a1e90ec1a2 Contents/Resources/texmf/fonts/tfm/public/lm/rm-lmr8.tfm
- resource sha256 49ea6cb9257bbee0a3979c48a774cd221550ac1c20c95549efe45fc99cc18050 Contents/Resources/texmf/doc/fonts/lm/GUST-FONT-LICENSE.TXT
- info: control: 4 results, 9 missing-metric diagnostics (recorded, not required; 0 only once the producer discovers the bundle itself)
- ok: env-direct: 4/4 compile results, 0 missing-metric diagnostics
- ok: env-user: 4/4 compile results, 0 missing-metric diagnostics
- ok: env-helper: preview update received, 0 missing-metric diagnostics
- ok: removed: 3 explicit missing-metric diagnostics naming ec-lmr10.tfm
- ok: removed: verifier refuses the copy (exit 1, ec-lmr10.tfm missing)
- ok: verify_bundle_resources.py /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ac12dd0004019340d/apps/mac/build/FlashTeX.app/Contents/Resources: exit 0 (9 pinned resources verified)
- ok: components.json carries the 9 verified resource hashes

Result: 7 ok, 0 failed
