# Exact font and rendering consumer integration

Merged exact renderer528ef7e with its published font1fc71f3 ancestry. No peer
font-engine or native activation changes were introduced by this merge.

Independent integration gates:108 ordinary font-resources tests,135 ordinary
rendering-core tests, strict all-target Clippy for both. Two explicitly invoked
pinned installed-font tests passed: mixed STIX/Liberation registry replay and
STIXMath metrics/assemblies/kern-boundary replay. Font/license hashes are checked
by the tests; no font bytes were copied into the repository.

Includes exact MATH constants, variants/fitting, assemblies and kern metrics,
rooted registry leases, mixed TrueType/CFF virtual-font consumers, and bounded
exact PDF operator stream. PDF container integration, native painting and reference
engine equality are separate gates. Device correction work after1fc71f3 is not
included in this checkpoint.

Resource timing caveat: fresh integration compilation around08:51–08:54Z overlapped
root FT047 whole-pipeline benchmark attempts. Both baseline and candidate timed
out under contention; this is not a measured candidate regression. Heavy integration
builds were stopped after completing these gates while root moved to isolated
same-input validation comparison. No new workers or paid calls were used.
