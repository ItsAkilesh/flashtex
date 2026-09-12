# make-app.sh --install update-path check — 2026-09-12T05:30:20Z

Verifies that `--install` replaces a **currently running** installed copy
atomically, launches the new build, and cleans up the retained previous
bundle — not just a fresh install into an empty `~/Applications`.

## Setup

- Version A = commit `2967d0a` (tip after the FLASHTEX_LOG launch-check
  follow-up, before this doc-only change).
- Version B = commit `42eea04` (`mac: document --install/--dmg in the
  Packaging usage line` — a trivial, real README diff made specifically to
  produce a new `git rev-parse --short HEAD` for this check, per the
  Commander's instruction to "touch README, new short SHA").

## Steps and real output

1. Built and installed A:

   ```console
   $ apps/mac/scripts/make-app.sh --install
   ...
   ==> Installing to ~/Applications
       kept previous install at /Users/jay3332/Applications/FlashTeX-previous.app pending launch verification
       installed /Users/jay3332/Applications/FlashTeX.app
   ==> Verifying the installed app launches
       launch OK (FlashTeX process is running)
       removed /Users/jay3332/Applications/FlashTeX-previous.app (new install verified)
   ```

   `plutil -extract CFBundleVersion raw ~/Applications/FlashTeX.app/Contents/Info.plist`
   → `2967d0a`.

2. Re-launched A manually and kept it running (the script's own install flow
   quits the app once launch is verified, so the update-path scenario — an
   *already-running* previous version being replaced — needs an explicit
   relaunch):

   ```console
   $ open ~/Applications/FlashTeX.app
   $ pgrep -x FlashTeX
   58100
   $ kill -0 58100   # confirmed alive immediately before step 4
   ```

3. Committed version B (`42eea04`, README-only) and rebuilt.

4. Ran `--install` again while A (pid 58100) was still running:

   ```console
   $ apps/mac/scripts/make-app.sh --install
   ...
   ==> Installing to ~/Applications
       kept previous install at /Users/jay3332/Applications/FlashTeX-previous.app pending launch verification
       installed /Users/jay3332/Applications/FlashTeX.app
   ==> Verifying the installed app launches
       launch OK (FlashTeX process is running)
       removed /Users/jay3332/Applications/FlashTeX-previous.app (new install verified)
   ```

## Assertions (real, checked after step 4)

- **The running A is replaced**: `kill -0 58100` → no such process (A's pid
  is gone). The install's own `pkill -x FlashTeX` (run before launching the
  new bundle) is what terminated it; nothing about the replace step was
  skipped because an old instance was live.
- **B launches**: the script's own launch-check passed (`launch OK`), and
  `plutil -extract CFBundleVersion raw ~/Applications/FlashTeX.app/Contents/Info.plist`
  → `42eea04`, matching version B's commit, not A's `2967d0a` — confirming
  the *installed and launched* copy is actually the new build, not a stale
  A left in place.
- **`FlashTeX-previous.app` cleaned**: `test -d ~/Applications/FlashTeX-previous.app`
  → absent immediately after the run (the script's own "new install
  verified" cleanup ran, and no leftover was found on a follow-up check).
- Final process state: `pgrep -x FlashTeX` → none (the script quits the app
  once its own launch verification succeeds, matching its documented
  behavior — not a failure of this check).

## Result

Pass. `--install` correctly replaces a live, running previous install
(not only a quiescent one), verifies the new build actually launches
before discarding the fallback copy, and leaves no `FlashTeX-previous.app`
behind on success.
