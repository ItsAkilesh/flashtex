# Existing wrapping-paragraph acceptance gap

Untouched producer65dbe7d compiles existing02-wrapping-paragraph from Mac
oracle dda8200 with zero diagnostics and the pinned rooted official LM2.004
metrics/font/license. The immutable registry/exporter emits the original PDF;
no reference coordinates or reference PDF were re-emitted. Source preamble
stripping and reference engine metadata are preserved separately.

Poppler26.01 at144DPI1224×1584 yields **979 differing RGB pixels**. All13 lines
have identical word membership, all210 words correspond in sequence, and
extracted text bytes are equal. This closes neither raw/operator nor visual
parity. The historical Mac27814-pixel figure uses different renderer/build
conditions and is not this measurement.

Existing PDF reader verification identifies a horizontal difference inside the
first word: reference `h` starts at72+707.2×11.9552/1000 =80.45471744bp;
original GID63 starts at84362432/2^20 =80.45428466796875bp. Reference code104
is not that original GID. The first next word `quick` starts at97.11070208bp
from the reference widths/TJ(-414), versus97.11606884002685546875bp in the
original. These are exact decimal operator differences, not tolerant equality.
The producer tick origin reaches the PDF unchanged, so this evidence does not
show an extra consumer placement error. Font-size/width/space serialization
and Type1-versus-CFF resource differences remain; no per-pixel causal proof or
justified coordinate patch was established.

The saved six-decimal Poppler word boxes give maximum absolute xMin delta
0.006042bp. Their yMin values are font-descriptor bounds, **not baselines**:
Type1/CFF ascent/descent metadata differs. Do not misreport that difference as
a six-point line shift. Raw owner operators retain actual baselines.

`measurement.json` pins the exact evidence, and `wrapping_reference.rs` uses
existing owner PDF reader/types to guard reference widths and original tick-to-PDF
position. Run the full original route with:

```
PATH="$HOME/.cargo/bin:$PATH" python3 crates/rendering-core/tools/replay_original.py --metrics-root /path/to/pinned-lm-root --fixture wrapping --report /tmp/wrapping.json
```

Expected measured status is mismatch, exit3; never change fixture content or
coordinates to turn it green. Next bounded candidate is existing18-ligatures:
it reuses the same LM12 assets while probing original-GID/Unicode mapping and
kerning independently of these matching multiline breaks. No sweep was run.
