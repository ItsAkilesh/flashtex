# Actual helper route and resource scope review

Pinned parent `3162b89a4db0e80953e7bbc0836d7ef98090ff19`, route integration `43c8094e7d2b5615623513a2b58fb233f8fa77af`. `review.json` hashes exact published exchange/source artifacts. Read-only Linux review: native execution and measurement remain the owner's evidence.

The original exchange contains 22 input and 35 output frames, **five display candidates and no error frames**. `configure_layout` requests only rules/font hints; subsequent `configure_display_candidates` requests `display-candidates-v1` and receives enabled=true, preview_error=null. All five candidates retain untrusted=true and source_actions_enabled=false. Their compile generations and editor versions remain distinct. This supersedes the earlier packaging test's failed candidate configuration as evidence that an actual native helper candidate exchange now exists. It is the acknowledged Value route, not proof of experimental raw-prototype adoption. The app log records one old-request paint rejection and later successful paint decisions; these are published owner observations, not local measurements.

GH31 commit `a9b55af746a829ae8158dac3fcfc9c09e811150b` is an ancestor of the pinned parent. Actual `V2FontStore` source reads file bytes once at initial resolution, checks discovered raw SHA/length and constructs CGDataProvider/CGFont from those same bytes. Cached immutable CGFonts remain keyed by raw SHA. The native store still accepts historical face-salted lookup spelling in addition to raw spelling; that compatibility lookup does not mean the strict CFF binder accepts a face-salted resource digest. The stale comments saying the producer currently emits salted SHA should not define the corrected full-snapshot contract.

## Metrics-only D4 and diagnostic D5 scope

Producer `9aaec57a` has an explicit `core14-afm` path with byte_length=0, no file path and identity from Core14Face. The native model decodes this metrics-only declaration; its current guard permits nonnegative length, not exclusively zero. `V2FontStore.resolve` refuses a referenced nonpaintable font before file lookup. Therefore this is **not authenticated font-program data and not a paintable CFF resource**. Scope raw complete-font-byte authentication to supported file-backed resources. Keep metrics-only declaration identity separate; neither the generic strict core profile nor `PipelineCff` accepts this extension. Do not add an implicit Times substitution or claim all native-decoded declarations are renderable.

Recovery remains route-specific: strict Rust helper pairing gates v1 diagnostic codes/severity, while searchable CFF export checks v2 error severity. Native V2Loader decodes/prepares frames and the view displays their diagnostics; that source path is not the strict Rust diagnostic policy. Do not standardize the policies merely by documenting them as equivalent. Owner-native recovery acceptance requires its exact route and fixtures.

The exchange does not prove bundle-only font discovery or override precedence, large-frame recovery, broad reference fidelity, GUI export safety or sub-200ms responsiveness. Those remain separate evidence scopes. The parent's reported 8 MiB versus 16 MiB frame failure and larger-frame paint rejection are retained limitations, not resolved by this small successful exchange.

## GH31 closure recommendation

The original same-path mutation before first resolve is addressed. Freshly fetched native parent `23596ffb8d967ced6c09e4a9aa5e0e8c0eb78b2e` contains fix `a9b55af746a829ae8158dac3fcfc9c09e811150b`; its `GlyphRunRenderer.swift` and `V2FontStoreIdentityTests.swift` are unchanged from the reviewed `3162b89`.

Exact tests in `V2FontStoreIdentityTests`:

- `testChangedBytesAfterDiscoveryAreRefusedBeforeCGFontConstruction`: appending a zero byte still produces a CoreGraphics-loadable font with unchanged glyph count/UPEM/PostScript name; resolution refuses both lookup hash spellings, then restoring original bytes resolves, proving refused bytes were not cached.
- `testVerifiedCGFontStaysImmutableAfterTheFileChanges`: verified cached CGFont remains the same object after mutation, including the historical lookup alias.
- `testUnknownHashAndLengthMismatchStayRefused`: retains explicit unavailable/mismatch failures.

Owner issue 31 comment `5646532909` and pinned `coordination/mac-helper-display.md` report these three tests within the filtered **52/52, zero skips** Mac run on the applied route tree (load 6–8). These execution results are owner-reported; no standalone raw XCTest transcript was located in this bounded review and no test was rerun here. Test source SHA-256 is `2a3d5a853f47fa00ca80892bde96eb8a42c67cc61ecb21c68b1bedc68579435e`; actual vendored LM10 fixture SHA is `1aa18cfefa58132c52ce5de70db1fd1154201c19cd2b2cdaffba4906a33e6852`. Both were independently checked from the current parent Git tree. The test can skip if its asset is absent, but the recorded run reports zero skips and that exact asset is present.

Recommend Commander closure for the original mutation bug. This does not close unrelated performance or raw-versus-salted protocol policy work, certify all native acceptance, or claim that the native store removed its historical alias. The store's remaining alias does not bypass authentication of the actual bytes used to construct CGFont.
