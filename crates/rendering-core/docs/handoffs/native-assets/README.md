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
