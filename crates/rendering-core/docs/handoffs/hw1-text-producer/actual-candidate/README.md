# Actual isolated Text producer acceptance

Producer base `9aaec57a019c6a0073419eeb3ec90f922f5b367c`, compiler cumulative
`f464c6d6` followed by separate comment delta `61bd63f7`, and reviewer adapter
`9ce5e9e0`. This is a MODIFIED candidate, not unchanged producer evidence or an
authoritative vendor repin. The offline locked release build passed. Binary,
all 383 source-file hashes, request/output hashes and exact resource hashes are
recorded alongside. No downloads or native execution.

The compiler patch applied cleanly to the existing vendor. Against compiler base
1c02, vendor json.rs/protocol.rs retain older serialization code; these two files
were preserved unchanged. No unrelated compatibility fixes were necessary.

Six source probes preserve all raw output. Five original labels `(a)` through
`(d)` and `and`, plus comment-before-argument joining, reach Roman12 original GIDs
with no diagnostics. Scripts are emitted but correctly retain lmr8/lmmi8 optical
resource warnings in this explicit three-font asset stage. All document hashes
match input; all math source spans remain whole-expression spans. Compiler's
more precise Text AST spans do not upgrade producer navigation automatically.

Broader Text fidelity is NOT accepted:

- `a b` returns status ok but the next origin advances by rm-lmr12 character
  slot32 width (285213/2^20 em), not SPACE fontdimen (342239/2^20 em). The emitted
  space GID/advance comes from the actual OTF, while placement follows that TFM
  character slot. Text-space glue requires an explicit adapter policy.
- Escaped braces preserve correct Unicode/cmap GIDs but placement uses ASCII
  slots123/125 (each 513365/2^20 em), not a proven text encoding binding. Correct
  extracted text alone does not prove corresponding metric geometry.
- `ffi` emits original GIDs55,55,66 in separate clusters; the existing TFM has
  f+f->slot11 and f+i->slot12 ligature actions. The per-character Text loop does
  not execute those actions. No shaping/ligature parity is claimed.

Initial `cases-missing-license` intentionally remains preserved: pointing at the
original flat asset directory failed its required `GUST-FONT-LICENSE.TXT` basename
and all six results recovered with required_metrics_unavailable. A separate
owned stage copied the unchanged official LICENSE bytes under the required name,
then reran unchanged requests/binary into `cases`. This setup correction is not a
producer fix or evidence that host fallback was used successfully.

`run.py` is the exact scratch replay driver; run it from the prepared scratch
root `/home/natkarri/flashtex-rendering-core/target/text-producer-9aa`, which contains
`producer`, staged `fonts`/`metrics`, and the binary. It is not a standalone asset
downloader/build system. Source audit pins all inputs. fontTools only inspects
existing TFM/cmap data for evidence; it is not part of production rendering.

Acceptance is bounded transport/type/source/GID evidence, not a LaTeX visual
oracle, PDF comparison, general Text support, or native performance measurement.
Owner next step: implement explicit supported text spacing/encoding/ligature
semantics, or emit scoped limitations for unverified cases before producer adoption.
