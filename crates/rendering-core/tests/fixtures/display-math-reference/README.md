# Existing display-math reference investigation

This is unchanged producer `65dbe7da7a182e99322070e2c9763cc3b69a342b`
output for existing `07-math-display`, using the same explicitly rooted official
LM2.004 metrics and immutable text/math fonts as `../math-reference`. The recorded
request strips the fixture preamble using the existing oracle harness policy;
reference engine metadata records its own LM preamble. No reference geometry was
substituted into the original. Diagnostics are empty.

At Poppler26.01.0, 144 DPI, 1224×1584 RGB, **1244 pixels differ**. Exact raw RGB,
PDF, source and output hashes are in `measurement.json`. Raw PDF bytes and parsed
operators also differ. This is gap evidence, not visual or general TeX parity.

The extracted texts differ solely at the summation: original `∑`, reference `X`.
Using the existing PDF reader/CMap parser, reference `/F53` shows code88 with
`LMMathExtension10-Regular`, but its declared ToUnicode CMap has no code88 entry.
The saved decoded CMap proves this absence; `pdffonts` reporting a ToUnicode table
exists does not prove coverage. The original explicitly maps original GID3060 to
`∑`. Do not change correct original text to `X` to satisfy this incomplete reference
mapping. Poppler's observed fallback is recorded; its internal fallback algorithm
was not investigated. Reference code88 is not an original-font GID.

Raw owner dumps show reference math uses LMMathItalic8/12, LMRoman8/12 and
LMMathExtension10; the original uses LatinModernMath for every math item. This
repeats the first fixture's resource/design difference. The sum origin is
(265.06750774383544921875,697.30734539031982421875) bp versus reference
(265.068,697.307) bp, from exact relative text operators. These are unequal;
this small delta alone does not explain every differing pixel. No verified
consumer-position error or justified producer patch follows from this evidence.

`math_reference.rs` reproduces original bytes through the immutable registry and
existing searchable PDF backend, checks reference hash, and validates both the
absent reference and present original Unicode mapping. It reuses licensed fonts
in sibling fixtures. Run `cargo test --manifest-path crates/rendering-core/Cargo.toml
--test math_reference`. For raster evidence use `pdftoppm -r 144 -png -singlefile`
on each saved PDF and compare decoded RGB without tolerance. Extraction uses
`pdftotext PDF OUTPUT`. Owner dumps use the existing flashtex-pdf-exact executable
SHA90ac4be6d4d4cf2f714b4b80e562cb4a6d55267daac332c2d18cbcc2064651a2.
