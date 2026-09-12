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

## Explicit passive binary eexec inspection

Additive `eexec::inspect_binary_eexec(&Resource, max_plaintext_bytes)` implements
the byte recurrence in Adobe Type1 specification sections7.1–7.2 (printed pages
62–64): fixed seed55665, constants52845/22719, unsigned16bit modular state updated
from ciphertext. Exactly four decrypted prefix bytes are retained separately and
excluded from returned plaintext/hash. State continues across adjacent PFB binary
records, including a prefix split across records; ASCII header/trailer is excluded.
No existing decryption utility was present in the linked crates.

The API is an explicit caller assertion that these records contain binary eexec;
container framing cannot prove that semantic role. First ciphertext cannot be one
of the four specified whitespace characters and at least one of its first four
bytes must be non-hex; otherwise a typed unsupported-prefix error refuses ambiguous
ASCII-hex input. Fewer than four encrypted bytes and exceeded caller/hard output
caps fail before plaintext allocation. The immutable result borrows the full PFB
resource identity/license and retains original ciphertext ranges/digest, prefix,
and decrypted bytes/digest. No PostScript is evaluated or decrypted payload dumped.
Decryption is not authentication or proof of valid font dictionaries.

Pinned real lmr10 evidence: ciphertext range5730..118683,
SHA02262ab31d397263650f1ec77c7bef04d0720419f69aa9e6562a52b2ddc85c62;
decrypted prefix00000000;112949remaining bytes,
SHAbd88b12233faf829fbf86770638e4aec367847bcdb06e285b75e2381787af9a4.
The existing real PFB test now verifies these exact observations. Synthetic vector
`d9d73f4a50b9fa428d59a36bd8f46f9cb9c47adfb0f3297e` decodes to prefix00010203 then
`/Private 1 dict def` followed by newline; every possible binary-record split
produces the same result. This vector is a synthetic arithmetic check, not an
independent rendering oracle. Charstring seed4330/lenIV, encrypted dictionary
interpretation, glyph identity, shaping and outlines remain unsupported. The
existing `require_outlines` gate is unchanged.

## Strict passive CharStrings/Subrs extraction

`type1_records::extract(&Inspection)` accepts the literal private-dictionary
organization in Adobe Type1 section2.4 example2 and section7.3. It verifies exact
RD/ND/NP reader/writer procedure definitions before honoring any binary length.
Lengths are literal integers; RD consumes exactly one whitespace byte followed by
that exact byte count. Binary delimiter-looking bytes are never tokenized. Returned
record ranges are offsets in the eexec plaintext, linked through the retained
Inspection to original ciphertext ranges/full font/license identity. Original glyph
names and subroutine indices remain intact; no Unicode/GID mapping is inferred.

Supported metadata: bounded literal numeric/boolean values and flat numeric arrays
for the listed hint/private keys, exact MinFeature procedure, lenIV0..32 with
spec default4. Negative lenIV/unencrypted extensions are explicitly unsupported.
Subrs are literal indexed assignments; CharStrings are literal named records.
Exact supported closure tokens are recognized but never executed. Unknown forms,
redefined readers, duplicate metadata/names/indices, out-of-range indices, trailing
executable code and invalid lengths fail. Caps:8MiB input,65536tokens,256byte token,
4096subroutine/character slots,1MiB per binary record,128numeric-array elements.
No general nested procedure evaluator exists.

The real LM OtherSubrs declaration is accepted only as one opaque271byte block
with SHA2f4adc4e2d703495501ce0ee4cd3d955949139a5f8623779025d58a200e14c3a.
Its code is neither copied into the implementation nor executed; any different
OtherSubrs form is typed unsupported. `opaque_other_subrs_present` makes the
unimplemented procedure semantics explicit. This narrow recognition permits record
inspection only, not callothersubr or hint-replacement support.

Record decryption reuses the eexec byte recurrence with the explicit charstring
seed4330 and validated lenIV, returning bytes only. Adobe's published section7.3
ciphertext example matches its plaintext exactly. Actual pinned lmr10 extraction
finds822glyph records and882subroutine slots; all decrypt within bounds. A yields
8bytes SHA13883a7a5915c1d3874a112f50a9e269e7663daf1587f4edbd504e381e16cdec;
.notdef yields5bytes SHAeaa5a748d6652e8857daddbbb79acaa6359e20d1eb90a1b1f0c843b3538a00d8.
These are byte-stage observations, not interpreted outline/shape/render results.

## Opt-in Type1 charstring geometry subset

`type1_outline::interpret(&Records, glyph_name)` interprets only hsbw/sbw,
rmoveto/hmoveto/vmoveto, rlineto/hlineto/vlineto, rrcurveto/hvcurveto/vhcurveto,
closepath, bounded callsubr/return and endchar. Primary contract: Adobe Type1
section6.4 path/width operators and section8 subroutines. Numbers retain Type1's
signed32bit integer encoding (including255); Type2 fixedpoint255 semantics are not
reused. Existing exact Coordinate/CubicPoint/CubicCommand types are reused for
checked accumulated geometry. closepath deliberately leaves the current point
unchanged, as required for Type1, and does not behave like PostScript closepath.

Width is established exactly once, including through a subroutine; argument stack
and call chain remain shared. Unknown opcodes, hints, div/arithmetic, seac, flex,
OtherSubrs/pop/setcurrentpoint all return typed unsupported without being ignored.
The strict subset requires explicitly closed contours; implicit closure cases
return Path rather than guessing fill/stroke semantics. Path commands are raw
character-space data: neither FontMatrix nor PaintType is inferred or applied.

Bounds:24operand values,16nested subroutines with cycle refusal,65536byte/operator
steps,16384output commands and16MiB cumulative decoded input per glyph. Record
allocation remains capped by the extractor. Each emitted command carries its
original subroutine-index chain and byte offset; the result retains original glyph
name, decrypted charstring digest, full font/license identity and exact width/
sidebearing. No platform fallback, PostScript execution or renderer activation.

Actual pinned lmr10 inventory is in pfb-type1-outline-inventory.json:6accepted
(5nonmarking and fraction.alt with10commands);353stop at hstem,4at vstem,4at hstem3,
455at div. Categories identify the FIRST unsupported opcode, not all features a
font needs. Synthetic tests verify exact cumulative cubic points, closepath/current
point behavior, subroutine width/provenance, cycles, operand/output/step caps and
unsupported operators. Existing broad `Resource::require_outlines` still refuses:
this opt-in subset does not establish safe whole-font rendering or visual parity.
