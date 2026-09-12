# Grok handwriting-to-LaTeX corpus

Owner: bridge/grok-corpus. Created September 12, 2026.

## The gap this closes

The bridge (`crates/bridge`) converts a captured image (`CaptureImage`: a PNG
or JPEG, base64-encoded) into a `Proposal` (LaTeX + ambiguities +
dependencies) via Grok (`crates/bridge/src/grok.rs`). Before this corpus
existed, exactly one live call had been validated against a synthetic,
already-typeset math image — clean, not handwriting. **Nobody could say how
the pipeline behaves on realistic input, because no test corpus existed.**

This directory is that corpus: every image is machine-typeset from known
LaTeX using this machine's MacTeX install, then rasterized and (for most
cases) deliberately degraded to simulate a real phone-camera capture. Because
every image starts from LaTeX source, **every image has an exact,
non-eyeballed ground truth attached** in `manifest.json`.

**What this proves and what it doesn't:** this corpus makes Grok conversion
quality *measurable*. It does not make it *good*, and it is not evidence of
handwriting-recognition accuracy — nothing here is a photograph of actual
handwriting, and no live Grok call has been made against any of it. See
"What's still missing" at the bottom.

## Layout

```
tests/grok-corpus/
  cases/*.tex        LaTeX source for each typeset case, ground truth marked
                      inline between `% GT-BEGIN` / `% GT-END` comments.
  images/*.png|jpg    The corpus images (committed; ~1.3 MB total).
  manifest.json       One record per case: image path, mime type, dimensions,
                      sha256, ground_truth_latex, category, and (for degraded
                      cases) which base case and transform produced it.
  generate.sh         Regenerates images/ and manifest.json from scratch.
  build_manifest.py   Called by generate.sh; assembles manifest.json.
  score.py            Offline scoring harness (see below).
```

## Regenerating the corpus

```sh
tests/grok-corpus/generate.sh
```

Requires, all already present on this machine and used offline (no network,
no API keys): `pdflatex` and `xelatex` (MacTeX, with amsmath, unicode-math,
fontspec, multicol, geometry — standard in a full TeX Live install),
`pdfcrop`, Ghostscript (`gs`), `cargo`, and `python3` (stdlib only, no pip
packages). The script is deterministic — fixed DPI, fixed crop margins,
seeded noise — so re-running it reproduces the same corpus (byte-identical
except for whatever the exact toolchain build/version happens to change,
e.g. Ghostscript's own antialiasing between versions).

Pipeline per typeset case: `pdflatex`/`xelatex` &rarr; `pdfcrop --margins N`
(tight-crop to ink, avoids committing mostly-blank letter-size pages) &rarr;
`gs -sDEVICE=png16m -r200` (raster at 200 DPI) &rarr; optionally re-encoded to
JPEG. Degraded variants and the two synthetic adversarial images are produced
by `corpus_augment`, a small binary added to the bridge crate
(`crates/bridge/src/bin/corpus_augment.rs`) specifically for this corpus. It
reuses the crate's existing `image` dependency instead of requiring
ImageMagick or Python/PIL (neither is installed on this machine; only `sips`
and `gs` were available for raster work, and `image` was already a bridge
dependency, so no new crate was added). All of its transforms are
deterministic (seeded PRNG, no wall-clock randomness):

| op | simulates |
|---|---|
| `rotate <deg>` | page tilted 6-15 degrees in the photo |
| `perspective <strength>` | camera held at an angle (true 4-point homography, not a shear approximation) |
| `contrast <delta>` | washed-out lighting / faint pencil |
| `lighting-gradient <strength>` | uneven light across the page (diagonal shadow) |
| `jpeg <quality>` | aggressive phone/messaging-app recompression |
| `downscale-blur <factor>` | low-resolution or out-of-focus photo |
| `noise <amount> <seed>` | sensor/ISO noise |
| `pure-noise` / `solid` | the two synthetic adversarial cases below |

Note on the perspective transform: its visible strength scales with how much
vertical extent the content has. A single line of math (most of this corpus)
shows only a mild keystone even at high `strength`, because there is little
vertical range for the projection to converge over — verified separately
against a full-page checkerboard test image, where the same code produces an
obvious, strong keystone. That is a property of the source content, not a
bug in the transform.

## What's in the corpus (27 cases)

- **baseline-clean (2):** a single clean equation via `pdflatex` (matches the
  already-validated scenario), and one via `xelatex`+`unicode-math` typed
  with literal Unicode math symbols, exercising the unicode-math path this
  machine's MacTeX confirmed working.
- **structural-hard (6):** multi-line `align*` derivation, a matrix
  determinant expansion, doubly-nested fractions plus stacked
  sub/superscripts, a prose paragraph with inline math, a two-column page
  with one equation per column, and a page with two independently labeled
  problems.
- **adversarial (3):** a page of prose with zero math, a blank white image,
  and a pure-noise image.
- **degraded-photo (16):** the 8 transforms above applied to two of the
  baseline cases (`clean-quadratic-formula`, `clean-multiline-derivation`),
  covering both a trivial and a structurally harder source.

Every image was checked against the *actual* `CaptureImage`/
`CaptureSubmit::validate()` logic in `crates/bridge/src/lib.rs` (max 8192px,
max 8 MiB decoded, PNG/JPEG only, must actually decode) — see
`crates/bridge/tests/grok_corpus.rs`, which loads every entry in
`manifest.json`, base64-encodes it, and asserts it validates. This is what
ties the corpus to the real interface instead of a guess at it.

### A finding worth flagging: the adversarial cases and the Proposal schema

`Proposal::validate()` requires 1-65536 non-empty bytes of `latex`. For the
blank-page and pure-noise cases, there is no faithful non-empty transcription
— the schema-honest outcomes are a `content:"refusal"` response (handled in
`grok::parse_response` as `provider_refusal`) or a proposal whose `latex`
carries some placeholder/ambiguous text with the real uncertainty pushed into
`ambiguities`. A model that instead emits confident, unrelated LaTeX for a
blank or noise image is hallucinating, and catching that is the entire point
of these two cases. This can't be checked automatically without a live call;
flag it for whoever runs the corpus against a real key.

## Scoring harness (`score.py`)

No network access, no API keys — it only ever diffs two LaTeX strings you
already have.

```sh
# one-off pair
python3 tests/grok-corpus/score.py --pair '\frac{1}{2}' '\frac12'

# once you have real Grok outputs saved as <case-id>.tex files in a directory
python3 tests/grok-corpus/score.py \
  --manifest tests/grok-corpus/manifest.json \
  --outputs-dir /path/to/captured-outputs \
  --json /tmp/report.json
```

Batch mode reports, per case: `match` (identical after normalization),
`close` (ratio >= 0.95), `partial` (>= 0.75), `divergent` (< 0.75),
`not_applicable` (adversarial case — human judgment required, see above), or
`not_run` (no output file found for that case id). It prints a summary and
can optionally fail (`--min-ratio`) for a CI-style gate.

### What the normalizer does and does not catch (read this before trusting a score)

`normalize_latex()` is a set of regex substitutions, not a LaTeX parser. It
deliberately collapses:

- `\frac{1}{2}` and `\frac12` (the motivating example from the task) — bare
  single-token `\frac` arguments are expanded to braced form.
- `\dfrac`/`\tfrac`/`\cfrac` vs `\frac`.
- `\left(`/`\right)` sizing vs plain `(`/`)`.
- `\,` `\;` `\!` `\quad` `\qquad` `~` spacing vs literal whitespace.
- Which display-math delimiter was used (`\[...\]`, `$$...$$`,
  `equation*`/`align*`/... environment wrappers) vs none at all.
- Unicode normalization form (NFC).

It deliberately does **not** catch:

- Mathematical equivalence that isn't a spelling variant: `a/b` vs
  `\frac{a}{b}`, `x^2` vs `x \cdot x`, commuted operands, etc. These score as
  genuinely different text, which is correct for a text-diff tool but wrong
  if you wanted "is this the same equation."
  - A close corollary: a single wrong character that changes meaning (e.g.
  `x^2+1` vs `x^3+1`) can still land a `partial` or even `close` verdict
  under character-level similarity, because most of the string is identical.
  **The ratio measures textual similarity, not correctness — always look at
  the diff, don't trust the verdict number alone**, especially near a
  decision boundary.
- Macro/package choices the corpus itself doesn't exercise (e.g.
  `\varnothing` vs `\emptyset`, `array` vs `pmatrix` variants beyond what's
  in this corpus).
- Anything requiring actual typesetting to compare (this only diffs source
  text, never renders and compares pixels).

When in doubt the normalizer under-normalizes: two equivalent strings are
more likely to be scored "different" than two different strings are to be
scored "the same." Treat every verdict below `match` as a prompt for a human
to look at the diff, not as a final answer.

## What's still missing

- No live Grok call has been made against this corpus (deliberately — the
  hard constraint for this task was no API keys, no network calls).
  Conversion quality is unproven either way; this corpus only makes that
  question *answerable* in one command once a key is available.
- No case here is a photograph of real handwriting. The degraded-photo
  cases simulate camera/lighting artifacts on top of *typeset* math, which
  stresses robustness to image degradation but not to actual handwriting
  variation (stroke style, pen pressure, cross-outs, ambiguous digits).
- The scoring harness normalizes LaTeX *source text*; it has no notion of
  rendered/visual or mathematical equivalence beyond the specific spelling
  variants listed above.
