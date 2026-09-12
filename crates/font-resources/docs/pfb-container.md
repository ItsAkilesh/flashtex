# Passive Type1/PFB resource container

Reviewed linked font-engine/PDF source at ancestor4b0fbd0: PDF writes Type1 standard
font names and handles OpenType CFF embedding, but neither crate parses PFB records.
`pfb::inspect` is an original bounded framing reader, not a PostScript interpreter.

Primary references: Adobe's [Type1 specification](https://www.adobe.com/content/dam/acom/en/devnet/font/pdfs/T1_SPEC.pdf)
describes clear/encrypted font programs but explicitly excludes platform disk
container details (printed page4). For the PFB record framing, the primary Skia
[Type1 PDF implementation](https://skia.googlesource.com/skia/+/main/src/pdf/SkPDFType1Font.cpp)
documents the two/six-byte headers, marker128, types1/2/3 and little-endian lengths.
No implementation code was copied. Container framing does not establish that the
payload is valid Type1 code.

Supported strict profile: nonempty ASCII records, nonempty binary records,
nonempty ASCII trailer records, then exactly128,3 at EOF. Adjacent records of the
same type are supported. Other orderings/empty segments are explicitly outside
this profile, not claimed universally invalid fonts. Headers and payload ranges
refer to original immutable bytes; marker bytes inside payloads are not scanned.
Every truncation, unknown type, missing end marker, trailing data and exceeded
limit fails. Limits:64MiB whole font,1024segments,1MiB license; metadata is bounded.
No buffer proportional to declared untrusted segment length is allocated.

`Resource::from_bytes` binds exact whole-font SHA/length, operator resource name,
license text/hash and declared license metadata, retaining immutable copies.
It neither proves legal embedding permission nor invents a PostScript font name,
face, GID or character map. `require_outlines` always returns the typed
`EncryptedOutlinesUnsupported`. No eexec decryption, shaping, rendering or PDF
export is activated by successful container validation.

Real evidence: existing official lm2.004bas.zip (archive97a725ea...) member
`fonts/type1/public/lm/lmr10.pfb`,119235bytes,
SHA84eb01245abb17c0530ca3909427256d73df8bed2d7243b9f2717ca08c010ac8.
The actually executed test found original payload ranges ASCII6..5724,
binary5730..118683, ASCII118689..119233; final marker occupies119233..119235.
License identity is the exact archive LICENSE from lm-required-metrics-provenance.
Asset stays in `/tmp/flashtex-lm-tfm-0or97xdj/v2.004`, not installed or committed.
Reproduce with `FLASHTEX_LM_TFM_DIR` pointing there and run
`cargo test --offline --manifest-path crates/font-resources/Cargo.toml --test pfb_inventory -- --ignored`.
Synthetic tests cover every truncation boundary, encoded little-endian lengths,
wrong phase order, payload marker bytes, caps, immutable copies and hash/license
mismatch. Neither synthetic nor real container acceptance is an outline claim.
