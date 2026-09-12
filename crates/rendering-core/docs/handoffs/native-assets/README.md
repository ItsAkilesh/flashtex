# No-TeX bundle resource gap: owner handoff

This is a read-only audit of the pinned component sources in `audit.json`, not an
executed macOS app or a claim that a specific released bundle uses those exact
binaries. The packaging and shell trees share the same make-app.sh bytes. Its
250–254 copy block includes OTF/license/README only; there are zero TFM files under
published apps/mac. All three existing regular10pt/regular12pt/math OTF hashes
match the immutable assets in our accepted fixtures.

The current producer's fonts.rs120–159 discovers Fonts beside the executable and
in ../Resources, then host TeX directories; `FLASHTEX_TFM_DIRS` precedes inferred
TFM directories. Its398–434 required-metric loader derives roots only by stripping
`/fonts/tfm/public/lm` and requires the whole four-file12pt set plus digest-bound
license. A flat Resources/Fonts path cannot satisfy that root shape, even if TFM
files are copied there. The separately searched ec-lmr10 file is also absent.
Existing MacTeX installations or inherited overrides can conceal both omissions.

`manifest.json` proposes the smallest rooted asset setup for the already measured
regular10pt/12pt and12pt roman-math fixtures. It pins five existing official2.004
TFMs, the exact rooted license, and the three existing OTFs. It does not promise
bold/italic/7pt/17pt coverage: those additional packaged faces have distinct TFM
names and require a separate inventory before broad no-TeX compatibility claims.

Reviewable owner changes, before signing the app:

1. Stage the five verified metrics into
   `Contents/Resources/texmf/fonts/tfm/public/lm` and the verified license into
   `Contents/Resources/texmf/doc/fonts/lm/GUST-FONT-LICENSE.TXT`. Use the existing
   official2.004 archive/member hashes, not a new release or generated metrics.
2. For both direct-producer and helper-spawned producer routes, prepend that
   absolute bundled TFM directory to `FLASHTEX_TFM_DIRS`, preserving explicit user
   entries. Alternatively add the equivalent bundle discovery in the producer's
   existing `default_tfm_dirs`; that is an owner implementation choice. Copying
   files alone is insufficient with the audited discovery code.
3. Include these exact resource hashes in the existing component/resource report
   before signing. Refuse missing or mismatched resources during packaging rather
   than relying on the build machine's TeX tree.

Acceptance belongs to the existing Mac owners: install the generated bundle in an
app-only environment with no host TeX/resources/overrides available. Run the same
regular10pt corrected multi-document fixture and existing12pt text/math fixtures;
retain source/binary/resource hashes and require zero missing-metric diagnostics.
Check both native direct and helper child process routes. Deliberately omit/change
one asset and retain explicit failure evidence. Native launching, signing, and
render/reference parity remain unverified here. GH34 remains limited to its fixed
Linux fixture; this is a separate packaging issue.

## Read-only verifier for the Mac owner

From the repository root, run:

```sh
python3 crates/rendering-core/tools/verify_bundle_resources.py /path/FlashTeX.app/Contents/Resources
```

Exit0 means the nine pinned resources (three fonts, five metrics, one license)
match the proposed layout, byte lengths and SHA256. Exit1 reports resource refusal;
exit2 reports setup/pinned-manifest refusal. It does not search host TeX, create
files, install fonts, or claim the application actually discovers the verified
resources. Extra files are explicitly not scanned. The bundled manifest itself
is SHA-pinned so deleting an entry cannot silently lower the required coverage.

Descriptor-relative no-follow traversal refuses symlinks and nonregular assets.
Manifest paths reject absolute/traversal/empty components and duplicates; file,
entry-count and total byte budgets bound the work. Hashing reads bounded chunks,
checks length and metadata stability, and never prints font bytes. A successful
read checks that observed snapshot only; it does not seal later native file use.

`verifier-evidence.json` records an isolated assembled resource stage using existing
pinned files: all nine verified. Missing10pt, same-length corrupted10pt,
misrooted12pt and changed license stages each refused. The temporary stages were
removed afterward; this is no native installation/execution evidence. Nine focused
Python tests additionally cover unsafe paths, duplicate/changed manifests,
symlink files/directories and a FIFO without blocking.

## Candidate patches (not owner-applied)

`packaging.patch` targets exactly af155e59f218cc3e88e19d15b1d042223d0ee298
`apps/mac/scripts/make-app.sh`. It requires the packaging-time
`FLASHTEX_BUNDLE_TEXMF_ROOT` to name an explicitly supplied official2.004 root,
stages only the existing manifest's rooted metric/license entries, and calls the
same pinned verifier before signing. It does not download anything or infer files
from host TeX. The existing OTF copy stays in place and is checked by the verifier.
The patch requires verifier product8e6d2787 and its pinned manifest in the owner
checkout; the packaging source by itself predates that dependency.

`producer-discovery.patch` targets exactly
f762f82a7307dc3d6522078364af79e09a51ad06 `crates/render-pipeline/src/fonts.rs`.
It adds `../Resources/texmf/fonts/tfm/public/lm` relative to the actual producer
executable, which the packaging script places in Contents/MacOS. Explicit
`FLASHTEX_TFM_DIRS` entries remain first and are never replaced. The existing font
and host discovery paths remain afterward. Both direct and helper-launched bundled
producer paths use the same executable location; no Swift environment mutation
or separate resource loader is introduced. Owners must rebuild the producer from
the patched source and verify its packaged binary identity.

`patch-manifest.json` pins both patch bytes and bases. `patch-check-evidence.json`
records `git apply --check` against both exact files in disposable scratch,
`bash -n` of the resulting packaging script, structural override ordering, and
execution of only the new staging block using existing official assets. Nine
resources verified; missing source10pt metrics refused before signing. No Rust
producer or native application was executed by these checks.

The bounded check can be repeated from the repository root with:

```sh
python3 crates/rendering-core/docs/handoffs/native-assets/check_candidate_patches.py
```

It needs the previously acquired official assets at the recorded local path;
missing assets are a setup failure, not a download trigger. These patches are
reviewable candidates only. The Mac owners still own applying/rebasing them and
the app-only/nohostTeX acceptance gate above.
