# Pinned 602-pixel inline-math mismatch classification

Status: measured classification and owner handoff. No producer patch is justified
by the current geometry evidence. Inputs are the unchanged published65dbe7d
original and committed pdfTeX reference in `tests/fixtures/math-reference`.
Both used the recorded correct assets; original diagnostics are empty. All
602 differences below use the same Poppler26.01,144DPI,1224×1584 RGB arrays.
No source or font was replaced for this analysis.

The existing PDF owner's `flashtex-pdf-exact dump` produced the retained raw
operator/font traces. Its exact binary SHA is recorded in classification.json.
No PDF parser was added. Source font IDs and original GIDs in the candidate were
checked against the actual display; reference Type1 character codes are not
comparable original GIDs, and reference source provenance stays unknown.

| Spatial window | Differing pixels | Candidate-only ink | Reference-only ink |
| --- | ---: | ---: | ---: |
| text colon |20|0|4|
| numerator/denominator |101|9|13|
| alpha |103|17|14|
| plus |61|0|0|
| beta |134|29|27|
| radical |102|12|10|
| radicand x |81|6|3|

The windows are disjoint x ranges derived from displayed positions, not claimed
reference source-map assignments. All602 differences fall in them:582 are
math-associated and20 surround the text colon. Ink means any nonwhite RGB
pixel; the raw pixel-difference count has no tolerance. The fraction-rule window
has zero differences. The radical-rule interior excluding its shared left join
also has zero differences; this does not assert equality at the glyph/rule join.

## Geometry and consumer observations

Candidate original GIDs1296/1297 (a/b),4459/4460 (alpha/beta),12 (plus),3077
(radical),1319 (x) reach the CID PDF unchanged. Its exact decimal origins and
sizes are the integer ticks divided by2^20 with the existing page y flip.
No consumer-side additional advance was found in the retained operator trace.

For reference a, the text-line operators give x=72+69.153=141.153 and
y=708.045+4.707=712.752 PDF points. Candidate x/y differ by
-0.000217559814453125/-0.000043060302734375. For b, reference141.590/703.922
versus candidate141.59049510955810546875/703.922199249267578125. The checked
radical/x origins likewise differ by less than0.001bp. All exact differences
are retained in classification.json; they are not rounded away or called equal.

Reference rules are horizontal stroked paths with butt caps. Their exact filled
rectangle is centerline ± half the declared stroke width. Comparing those with
the candidate rectangles also gives nonzero differences below0.001bp. This
change of representation is documented, not byte/operator equality. No measured
large rule or baseline displacement explains this fixture's pixel pattern.

## Concrete producer resource distinction

The reference embeds LMMathItalic8-Regular for fraction letters,
LMMathItalic12-Regular for alpha/beta/x, LMRoman12-Regular for plus, and
LMMathSymbols10-Regular for the radical. The candidate uses a single
LatinModernMath-Regular CFF resource for every math glyph.

Published `mathtex.rs::TexMathMetrics::otf_gid` checks the TFM font name for
extension chains, then maps ordinary characters with `base(ch)` into that one
OpenType face. `mathfont.rs::math_char` maps letters/Greek to mathematical italic
Unicode. Ordinary TFM family/optical-size distinctions do not select a different
outline resource at this boundary. This is an observed producer design
constraint, and a likely contributor to outline/optical differences. The same
reference versus candidate font-design distinction is not a consumer placement
failure. CFF versus Type1 hinting/rasterization effects have not been isolated.
The20 colon pixels also show that not every changed pixel belongs to math.

## Requested owner follow-up

Preserve exact TFM font/size/code → actual font resource/original-GID binding
traces during a future debug run. Investigate an explicit, licensed,
optical-style-aware mapping or verified OpenType style selection with the font
owner. Reference Type1 code97 must not be cast to candidate GID97. A general fix
requires actual outline/style evidence; adjusting these near-equal coordinates
to this one reference would not establish one.

No minimal geometry patch is proposed: the measured positions do not identify
an incorrect general transformation. Keep producer resource selection,
consumer exact placement, and rasterizer/hinting attribution separate. The next
existing math fixture can test whether the resource pattern repeats, without
turning the current inference into a parity claim.

Reproduce raw traces with the existing owner executable's `dump` command on both
pinned PDFs; generate both PNGs with `pdftoppm -r 144 -singlefile -png`. Compare
RGB tuples directly using Pillow12.1. The disjoint x windows and exact arithmetic
values are recorded in classification.json; source/reference hashes and RGB
hashes remain in the fixture measurement.json.
