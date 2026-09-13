# On-device capture conversion (design, for owner review)

Status: proposal, FT-066 (kabir-claude, mac-m5pro-kabir), 2026-09-13. Nothing
in this document is implemented except the provider seam it plugs into.
Reviewer: the owner (jay3332). Mac-side counterpart: `ConversionProvider` in
`apps/mac/Sources/FlashTeXMac/ConversionCredential.swift` (mac-claude-a).

## Why

Capture conversion (iPad drawing or photo, then LaTeX for review) is the only
model-backed feature FlashTeX ships (`docs/extensibility.md`). Today it needs a
network provider: xAI, or since FT-066 any OpenAI-compatible endpoint. That
means a key, a per-call cost, a round trip of 3–60 s (measured in
`docs/evidence/grok-live-20260912T191209Z`), and the capture leaving the
machine. An on-device model fixes all four for the common case (one equation or
a short derivation) and can fall back to a network provider only if the user
picks one.

## The seam it plugs into (implemented)

`crates/bridge` now separates *which* provider converts from *how* a capture
becomes a reviewed edit:

- `trait Converter { convert(..) -> Proposal; convert_with_evidence(..) -> Converted }`
  (`crates/bridge/src/lib.rs`). Everything after it stays the same for every
  provider: `Proposal::validate` and the LaTeX safety scan, the `UNSUPPORTED:`
  and empty-argument insertion block, context staleness checks, the durable
  journal, and the explicit approve → prepare → apply gate.
- `ProviderKind` plus `ProviderConfig::resolve/build` (`src/provider.rs`).
  Only an explicit `--conversion-provider <name>` flag enables one.
- `ProviderEvidence {provider, model, response_id, usage}` travels in
  `capture_proposal` and `capture_status` and is saved in the journal.

An on-device provider is one more `ProviderKind` arm, `on-device`. It needs no
key and no base URL. Its `model` is the bundled model's version string, and
its `response_id` is a local inference id (for example
`ondevice-<sha256(image)[..12]>-<timestamp>`). `usage` is `null`, or a count of
decoder tokens if that turns out to be useful.

## Where inference runs

The bridge is a Rust child process. Core ML and Vision are Apple frameworks
reached from Swift. There are three options:

| Option | How | For | Against |
|---|---|---|---|
| A. Swift helper process (recommended) | `flashtex-convert-ondevice`, a small Swift CLI bundled in `FlashTeX.app/Contents/MacOS/`, speaking one JSON line in and one out. The bridge's `OnDeviceClient` spawns it per request. | Keeps the bridge Rust-only and cross-platform. Uses the existing helper-process contract (`docs/extensibility.md` §1). Core ML crashes and memory spikes stay out of the journal process. The same Swift code can run on the iPad directly. | Process spawn plus model load cost about 150–400 ms on a cold start. Mitigate with a `--serve` mode the bridge keeps alive. |
| B. Loopback OpenAI-compatible server | Run the model behind `http://127.0.0.1:PORT/v1/chat/completions`, then `--conversion-provider openai-compatible` with no key. | Works **today** with no bridge change (see "What exists now"). | A generic server stack (llama.cpp, MLX) rather than Core ML/ANE. A port to manage. Weaker sandboxing story. |
| C. Core ML through Rust FFI | Call Core ML from the bridge via `objc2`. | No extra process. | macOS-only code in a cross-platform crate. Unsafe FFI in the process that owns the journal. |

Recommendation: prototype with B (zero integration cost), ship A.

Contract for A (proposed, `ondevice-v1`):

```json
{"request_id":"…","image":{"mime_type":"image/png","data_base64":"…"},
 "instructions":"…","supported_features":["\\frac{}{}", "…"]}
```
```json
{"request_id":"…","status":"completed","proposal":{"latex":"…","ambiguities":["AMBIGUOUS: …"],"required_dependencies":[]},
 "model":"flashtex-ink2tex-0.1.0","inference_ms":212}
```

The helper never writes files and never opens a socket. The app runs it
sandboxed with network denied. The bridge applies the same 512 KiB reply cap
and `Proposal::validate` as for network providers.

## Model

### Task framing

The input is a PencilKit drawing rendered at 2× (`apps/ios/FlashTeXPad/CaptureView.swift`)
or a photo, up to 8 MiB. The output is the same structured proposal. The model
has two stages:

1. **Layout and region detection** with Vision
   (`VNDetectRectanglesRequest`/document segmentation for photos, a crop to the
   ink bounds for drawings). Deskew and binarize. A photo of a page may hold
   several expressions, so emit one region per line or block.
2. **Image-to-markup**: an encoder-decoder over each region.
   - Encoder: a small ViT or ConvNeXt-T at 384×384 (or a variable-width
     aspect-preserving 128×768 strip), about 20–30 M parameters.
   - Decoder: a 4–6 layer transformer with a LaTeX tokenizer. About 600
     tokens: control sequences as single tokens, taken from
     `features::supported_features()` plus the TikZ subset. About 20–40 M
     parameters.
   - Constrained decoding: tokens outside `supported_features` are masked
     unless the model emits an `UNSUPPORTED` marker. That makes the
     report-don't-substitute rule structural rather than a prompt request.
     Brace balance is enforced by a stack in the decoder loop.
   - Ambiguities: where the top-2 token margin at a position falls below a
     calibrated threshold, emit `AMBIGUOUS: <token a> or <token b> at …`.

**Strokes**: PencilKit captures carry timing and pressure, which online
handwriting recognition exploits (stroke order separates `1`/`l`, `x`/`×`).
The capture protocol sends only a PNG today. A later protocol revision could
add an optional `strokes` array. The model should be image-first so that
photos keep working, with strokes as an optional second input. This is a
protocol change (runtime-v1 owner) and is not required for v1.

**Sketch to TikZ** is a separate decoder head, or a separate small model,
trained on (rendered TikZ, source) pairs restricted to the compiler's TikZ
subset (FT-062). Ship it after math. Figures are rarer and harder to review.

### Size and latency budget

| | Budget | Rationale |
|---|---|---|
| Download (app bundle delta) | ≤ 60 MB (8-bit palettized weights) | Comparable to a font pack; no separate download flow in v1 |
| Parameters | ≤ 80 M total | Fits the ANE comfortably; ~40 MB at 4-bit, ~80 MB at 8-bit |
| Peak memory | ≤ 400 MB | Runs next to the compiler/preview on an 8 GB Mac and on an M-series iPad |
| Latency, one equation | p50 ≤ 300 ms, p95 ≤ 1 s warm on M1 | 10–100× faster than the measured network path |
| Latency, one page photo | ≤ 3 s | Regions decoded in parallel |

Conversion runs `.mlpackage` with `computeUnits = .all` and falls back to the
CPU automatically. Weights are quantized with `coremltools` (palettization
plus linear 8-bit), and the test gate requires ≤ 1 point of exact-match loss
against float16.

## Data: reviewed captures, with consent

The best training data is what users already produce: a capture, the proposal,
and the LaTeX they actually **approved** (possibly edited) in the review sheet.
That triple is exactly supervised data. It is also personal data (their notes
and handwriting), so collection is opt-in and minimal.

- **Opt-in only.** A separate, off-by-default Preferences switch: "Contribute
  approved captures to improve on-device conversion". Its text states exactly
  what is sent. Changing the conversion provider never changes this switch.
  Without consent nothing leaves the machine. The journal stays private
  (`crates/bridge/README.md`).
- **What is kept** per contributed capture: the image (PNG/JPEG as submitted),
  the approved LaTeX as inserted (from the `AppliedEdit`/`PreparedEdit`
  replacement), the provider proposal and `ambiguities`, `ProviderEvidence`
  (which model produced the draft, to detect teacher bias), and
  `supported_features` hash. **Not kept**: the surrounding document, project
  names or paths, `source_before/after`, the destination, the user's
  identity, device identifiers. Photos are cropped to the ink region on
  device before upload. EXIF and location metadata are stripped.
- **Only approved-and-applied captures** are eligible. Rejected captures and
  proposals blocked for unsupported constructs are excluded. The reviewer's
  edits are the label.
- **Local first.** Contributions queue in a separate directory in the app
  container, which the user can see and delete from Preferences before upload.
  Upload is batched, over HTTPS, to a project-controlled bucket. Consent
  withdrawal deletes the queue and requests server-side deletion by a random
  contribution id kept only on the device.
- **Retention and access.** Raw images are kept for 12 months, then only
  derived training shards. Access is limited to the model training job. A
  datasheet documents composition. Nothing is shared with a network provider.
- **Legal/ethics review** of the consent text and retention before any
  collection ships. This design does not authorize collection or any spend.

Other sources that need no user data:

- **Synthetic, exact ground truth**: the existing `tests/grok-corpus` pipeline
  (typeset with MacTeX offline, rasterize, degrade) scaled from tens to
  millions of samples. Render with many fonts, then apply handwriting-style
  augmentation (elastic distortion, stroke-width jitter, per-glyph affine
  transforms), paper texture, lighting, and JPEG artifacts.
- **Synthetic handwriting**: render LaTeX token sequences with handwriting
  glyph sets drawn by contributors (a glyph-per-token set collected under
  explicit consent, far less sensitive than notes).
- **Public datasets with licences that allow training** (for example CROHME
  handwritten math and im2latex-style rendered formulas). Each dataset's
  licence is checked before use and recorded in the datasheet.
- **Teacher distillation**: label unlabeled synthetic or consented images with
  a network provider through the same bridge seam, keeping only labels that
  compile with zero diagnostics and round-trip (re-render, then compare
  rasters). This is a paid operation and needs its own explicit budget. It
  is not part of this design's authorization.

## Training plan

1. **Baseline (week 1–2)**: generate 1–2 M synthetic pairs from the corpus
   generator. Train encoder-decoder at 384 px on one GPU node. Evaluate on
   `tests/grok-corpus` (held out) with `score.py`.
2. **Handwriting (week 3–4)**: add public handwritten math plus augmentation.
   Fine-tune. Add constrained decoding and ambiguity calibration.
3. **Core ML (week 4)**: convert with `coremltools`, quantize, build the Swift
   helper (option A), and wire `--conversion-provider on-device`.
4. **Consented data (after legal review)**: monthly fine-tunes on approved
   captures. Each release is gated on the evaluation below.

Evaluation (a release gate, run offline in CI on held-out sets):

- exact match after LaTeX normalization, and token edit distance;
- **compile-and-compare**: the proposal compiles with the FlashTeX compiler
  and zero diagnostics, and its raster matches the ground-truth raster (MacTeX
  as oracle, as FT-060 does, never in the product path);
- **honesty**: on the adversarial cases (unsupported constructs, prose-only
  images) the model must emit `UNSUPPORTED:` or refuse, never a hollow
  `\sqrt{}`. This check is non-negotiable;
- latency and peak memory on the reference Macs and iPads.

Ship criterion for v1: on held-out real handwriting, compile-and-compare is no
worse than 10 points below the network provider, honesty is ≥ the network
provider, and p95 latency ≤ 1 s. Below that bar, on-device ships as an
"offline draft" choice rather than the default.

## Mac and bridge changes when this is built

- Bridge: `ProviderKind::OnDevice`, `OnDeviceClient` (spawn/serve the helper,
  bounded reply, same `Converted`), and `--conversion-provider on-device`.
  Resolve the helper through `FLASHTEX_CONVERSION_ONDEVICE_HELPER`, then next
  to the bridge binary.
- Mac: `ConversionProvider.onDevice` with `needsCredential = false` and no
  Keychain service. `bridgeFlag` stays `--conversion-provider on-device`.
  Preferences shows model version and size.
- iPad (later): run the same Core ML model locally for instant previews
  before sending. The Mac bridge remains the authority for the reviewed
  proposal.

## What exists now

- The seam, evidence and flag: `crates/bridge/src/{provider,openai,grok}.rs`,
  tested against checked-in reply fixtures through the real binary
  (`crates/bridge/tests/conversion_provider.rs`).
- Option B works today with no model integration: any local vision model
  served as OpenAI-compatible Chat Completions on loopback, with
  `--conversion-provider openai-compatible`,
  `FLASHTEX_CONVERSION_BASE_URL=http://127.0.0.1:PORT/v1` and
  `FLASHTEX_CONVERSION_MODEL=<id>`. No key is needed on loopback. Quality
  depends entirely on that model and has not been measured.

## Open questions for the owner

1. Approve opt-in contribution of approved captures at all, and who owns the
   bucket, retention and the legal review?
2. Is a ≤ 60 MB bundle increase acceptable, or should the model be an
   optional download?
3. Ship on-device as the default provider once it meets the bar, or keep it
   as an explicit choice?
4. Should the capture protocol grow an optional `strokes` payload (runtime-v1
   owner), or stay image-only?
