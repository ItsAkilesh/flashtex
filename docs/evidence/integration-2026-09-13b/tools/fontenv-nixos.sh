# Gate font environment for the NixOS PC.
#
# The rooted TFM loader refuses symlinked path components, and every NixOS
# texmf-dist directory is a symlink farm, so pointing at the system tree gives
# `required_metrics_unavailable` and a silent fall back to OpenType advances.
# With no override at all the renderer finds none of its default directories
# and substitutes TIMES metrics with `font_unavailable` -- and still exits
# "recovered, N pages, 0 overfull", so an overfull-only harness scores wrong
# geometry as a pass.
#
# Outlines: the repo's bundled flat Fonts dir (symlink-free).
# TFM: bundled lm + amsfonts symbols, plus the complete `ec` from the nix
# store, because the bundled jknappen/ec has 70 files and NO ectt* (T1
# typewriter), which makes every \texttt fixture emit ec_metrics_unavailable.
# The nix-store ec path has no symlinked component, so the rooted loader
# accepts it.
F=/home/kubar/code/flashtex/apps/mac/Fonts
EC=/nix/store/ixjcn8fbciq57zlrrlas3bxych7dd4fm-ec-1.0-tex/fonts/tfm/jknappen/ec
export FLASHTEX_FONT_DIRS="$F"
export FLASHTEX_TFM_DIRS="$F/texmf/fonts/tfm/public/lm:$EC:$F/texmf/fonts/tfm/public/amsfonts/symbols"
