# Byte and pixel parity: unresolved, and separate from compiler latency

Status: **unresolved**. Nothing in this document is a parity claim.

FT-002 revision 8 requires unresolved byte/pixel parity to be published
separately from compiler latency, because they are different questions with
different evidence and conflating them would overstate what is proven.

## What IS measured, and where

Compiler latency lives in `README.md` and is produced by
`cargo run --release --bin scaling_bench`, which prints the SHA-256 of every
generated input and of the binary. It measures one thing: source text in,
laid-out result out, inside this crate.

Output stability lives in `tests/pinned/*.jsonl`, compared byte for byte with no
normalisation, in both the unnegotiated and negotiated capability modes.

Neither of those is parity with anything. They prove this compiler is
self-consistent and fast, not that its output matches any other renderer.

## What is NOT established

**Raw PDF byte equality.** `crates/compiler` no longer writes PDFs; `crates/pdf`
(FT-009) does. No test in this crate compares produced PDF bytes against a
reference, and none can, because a reference does not exist here. Issue #9
closed on the producer side: fraction rules are now real `rule` primitives under
`rules-v1` and Greek and operator glyphs resolve through the Symbol face. That
removed known corruption; it did not establish byte equality with anything.

**Pixel equality / native paint parity.** Nothing in this crate rasterises. The
native shell (FT-003) paints, and `tests/visual-corpus` plus `tools/raster-compare`
exist for that comparison. This crate contributes positioned items and exact
source ranges; whether they paint identically to a reference is measured
elsewhere, by owners who are not this agent.

**Keystroke-to-visible-output latency.** The product requires under 200 ms from
keystroke to matching visible output. The compiler's warm-edit p95 is well inside
that, but UI paint, scheduling, IPC transport and PDF writing are all outside
this crate. The product target REMAINS UNPROVEN and can only be established by
measuring the real application. The benchmark says so in its own output so a
compiler number can never be mistaken for the product number.

## Known fidelity gaps that would affect any future parity comparison

- ~~Ligature substitution is not applied.~~ **Corrected 2026-09-12: it is.**
  This document previously claimed ligatures were missing. They are applied:
  `fi` shapes to a single ligature cluster whose source range still covers both
  input bytes, so advances reflect the ligature and click-to-source survives it.
  A test in `src/layout.rs` now pins that behaviour so the claim cannot drift
  from the code again. The README was already correct; this file was not.
- Paragraph breaking now delegates to `flashtex-paragraph-layout`'s TeX-style
  total-fit algorithm over font-engine-shaped boxes, finite interword glue and
  discretionary penalties. This closes the former greedy-breaking gap, but it
  is not by itself a claim that every TeX paragraph parameter is modelled.
- Explicit `\-` discretionaries can now split a word. Both fragments retain
  their literal UTF-8 slices and the generated hyphen is attributed to the
  complete source word. Automatic pattern hyphenation remains missing because
  `flashtex-paragraph-layout` currently ships only the `Hyphenator` interface,
  `NoHyphenation`, and `ExplicitDiscretionary`; it explicitly does not ship a
  pattern implementation. The compiler does not duplicate one locally.
- Characters outside the base-14 repertoire are reported, not rendered, in the
  export path. `crates/compiler/src/export.rs` names each one and the reason.
- The features in `UNSUPPORTED.md` are absent by design at this milestone, and
  any document using them cannot be compared against a full engine.

Each of these must be closed before a parity comparison would mean anything. A
parity number produced today would be measuring these gaps, not fidelity.

## Where responsibility sits

This crate owns positioned output, source identity and compiler latency. PDF
bytes belong to FT-009, native paint to FT-003, and end-to-end demo verification
to FT-008. This document exists so those remain separately owned and separately
evidenced rather than folded into one number.
