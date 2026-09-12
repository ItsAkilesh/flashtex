# Native LaTeX workspace — product and engineering master plan

Status: proposed plan, based on the founders’ decisions. No implementation or benchmark results are claimed.

## 1. Product intent

Build an open-source, local Mac LaTeX IDE for undergraduate students that remains useful to advanced authors. Its defining experience is responsive source editing with a live document preview, plus handwriting and camera capture on iPad/iPhone that becomes editable LaTeX at a selected destination on Mac.

The compiler is a new implementation written from scratch in Rust. Existing TeX engines may inform design and serve as external test references, but must not execute compilation on behalf of the product. Swift implements the native interface. Application logic should generally live in Rust; small Apple framework adapters may be Swift.

The long-term compatibility requirement is a drop-in replacement for existing LaTeX projects. This requirement is not satisfied by a renderer that implements a handful of familiar commands.

## 2. Feasibility and commitments

The team has five hours for an initial working demonstration, little prior implementation experience, access to AI coding assistants, Apple development hardware, device signing, and a Grok API key.

A from-scratch Rust engine with universal LaTeX compatibility, arbitrary-error recovery, and sub-200 ms updates for every document cannot credibly be delivered or validated in five hours. AI assistance does not remove the need to implement and verify language semantics.

The recommended five-hour milestone is a functioning vertical slice with an explicitly declared supported subset, actual incremental reuse, native editing, PDF output, and both capture inputs. Even this is an aggressive target whose outcome depends on early build and device-integration success. This milestone is a proposal for sequencing; it does not weaken the founders’ full-compatibility requirement or imply that they have accepted partial compatibility as the finished product.

Use three labels in progress reports:

- Implemented and tested: demonstrated behavior with evidence.
- Implemented, unverified: code exists but its behavior is not established.
- Required, outstanding: part of the product specification that remains unfinished.

Do not describe the demo as a fully compatible replacement. Do not silently delegate unsupported input to another engine. Unsupported features must produce explicit diagnostics.

## 3. Confirmed product requirements

| Area | Requirement |
|---|---|
| Audience | Undergraduate students first; advanced workflows eventually supported |
| Desktop | macOS only; native Swift interface |
| Compiler | Original Rust implementation; no existing engine backend |
| Core logic | Rust wherever practical |
| Projects | Ordinary LaTeX project folders; no required proprietary source format |
| Compatibility | Drop-in replacement, including packages, diagrams, graphs, and complex projects as the long-term target |
| Preview | Automatic updates while typing; sub-200 ms target on the agreed benchmark |
| Recovery | Partially recovered output with explicit diagnostics and visible error indicators |
| Editing | Edit source; click preview to navigate to source |
| AI priority | Handwriting conversion and assistance, then error explanations, autocomplete, navigation, Git, citations/templates |
| Companion | Both Pencil/finger drawing and camera/photo capture |
| Transfer | Nearby devices; Mac awake and available; selected insertion destination |
| AI provider | Grok only initially; internet access acceptable |
| Collaboration | Deferred, with architecture that permits adding it |
| Dark mode | Preview only; exports follow the document’s specified colours |
| Distribution | Open source; app manages its own runtime |

## 4. Five-hour demonstration contract

Use an undergraduate-style document containing text, headings, equations, a figure, references, and a small TikZ diagram. Inspect its exact constructs before coding. Select honest, narrowly bounded examples; publish which constructs actually work.

The demo should:

1. Open a local `.tex` file and its related assets.
2. Edit text and supported equations in a native source editor.
3. Update a preview produced by the Rust compiler and show measured latency.
4. Introduce a recoverable syntax error and display the resulting diagnostic and recovered output.
5. Click a rendered item and navigate to its source range.
6. Pin an insertion destination; draw on iPad; convert through Grok on Mac; review and insert.
7. Repeat the conversion using a camera photograph.
8. Toggle dark preview and export a PDF whose colours still follow the source.

Each item requires a working end-to-end path. A mock preview, hardcoded output, or an existing engine hidden behind Rust does not count.

The team has accepted a representative document for performance evaluation. That does not establish broad compatibility. If references or TikZ cannot be implemented during the event, report those failures rather than substituting an image and claiming the language feature works.

## 5. Compiler design

### 5.1 Establish an honest language boundary

TeX is a programmable token-processing system, and LaTeX is built on top of it. A fixed command parser alone will not execute arbitrary LaTeX packages.

The long-term compiler requires:

- TeX-compatible tokenization, including mutable category codes.
- Expansion semantics, macro arguments, scoping, registers, and conditionals.
- Execution of primitives and the LaTeX format/package ecosystem.
- Horizontal and vertical lists, boxes, glue, penalties, and math layout.
- Font metrics, shaping, line breaking, page breaking, and output routines.
- File input/output, auxiliary files, cross-reference convergence, and bibliography workflows.
- Deliberately specified compatibility profiles for engine-specific primitives and behavior.

Supporting pdfTeX-, XeTeX-, and LuaTeX-dependent projects is a substantial compatibility programme. A filename ending in `.tex` does not supply enough information to select semantics. Engine selection and project configuration must be explicit or reproducibly inferred.

For the first milestone, document a finite grammar and supported primitive/command set. Treat any convenience parser as a prototype unless it implements the relevant TeX semantics. Do not assume a prototype parser can be extended into full TeX simply by adding command names.

### 5.2 Pipeline

Source revisions → tokenization → expansion/execution → layout structures → positioned display list → PDF and preview artifacts.

All semantic interpretation and layout belong in Rust. Swift displays compiler output and handles interaction. A Swift renderer must not independently interpret LaTeX and produce a visually similar substitute.

An assumed implementation policy: general-purpose libraries for fonts, images, PDF serialization, networking, and storage are allowed. The TeX/LaTeX semantics, typesetting decisions, and incremental compiler logic are original Rust work. Record dependencies and their roles. If “from scratch” also excludes these libraries, the implementation scope increases substantially.

### 5.3 Incrementality, explained operationally

After each edit, reuse only work whose inputs and relevant state are unchanged.

For the prototype:

1. Assign every document snapshot a revision number.
2. Track changed byte ranges and map editor coordinates correctly across Swift UTF-16 and Rust UTF-8.
3. Reuse tokenized/parsed units where the supported grammar permits it.
4. Cache layout by content, environment state, fonts, and layout constraints.
5. Invalidate dependent layout and pagination when a unit changes.
6. Render changed pages or regions and publish the result for the appropriate revision.

This gives real incremental work reuse for the implemented subset. Instrument cache hits and recomputed units so the claim can be demonstrated.

For full TeX, safe reuse must also account for mutable category codes, macro definitions, registers, assignments, auxiliary files, output routines, and external effects. Execution checkpoints are a candidate design, not a solved feature. Checkpoints must capture relevant interpreter state and either record or prevent replay of side effects. Restore before the affected execution point and recompute until reuse is proven safe.

Never assume that TeX can be independently compiled one paragraph or one page at a time. If correctness cannot be established, rebuild the affected suffix or the full document.

Use a fast in-memory preview path and an authoritative export path with the same semantics. Do not use an unrelated approximate renderer to claim immediate full-document compilation.

### 5.4 Performance contract

Target: less than 200 ms from a keystroke to visible matching output for ordinary warm edits in the representative document, measured on a named Mac.

Report median and p95 latency, document size, hardware, cold-start time, and edits tested. Include time spent scheduling, compiling, transferring artifacts, and drawing. Test a global macro or layout change separately from a local text edit.

Compile off the UI thread. Keep a bounded queue, coalesce pending edits, and avoid starting work for every obsolete intermediate revision. Do not cancel every build so aggressively that continuous typing prevents all preview progress. Publish only monotonically newer completed revisions, and label a preview that trails the source.

Sub-200 ms is a benchmark target, not a guarantee for arbitrary source: TeX programs and external computations can require unbounded work.

### 5.5 Error recovery and navigation

Diagnostics carry severity, source file, range, source revision, message, and optional suggested fix. Rendered elements carry source provenance through macro expansion and layout.

Recovery rules must be explicit. For a narrowly supported grammar, examples include a visible placeholder for an unsupported command and a temporary missing delimiter at a well-defined recovery boundary. These rules do not establish general TeX error recovery.

Show recovered output only when usable. Highlight affected regions where mapping is reliable; otherwise show a document-level warning. If recovery fails, retain the last valid output with an outdated-preview indicator. Never silently edit source to repair it.

Preview clicks use compiler-generated source mappings. Macro-generated output may map to its invocation, with definition navigation offered separately. Account for differing source revisions before moving the caret.

## 6. App architecture and extensibility

Suggested modules:

- `compiler`: tokens, execution state, layout, PDF/display-list output, source maps.
- `project`: ordinary files, revisions, configuration, assets, autosave.
- `build`: scheduling, cache management, diagnostics, artifact publication.
- `ai`: Grok client, context collection, response validation, proposed edits.
- `capture`: transfer protocol, capture queue, acknowledgements, destination tracking.
- `mac-app`: Swift source editor, preview, commands, panels, native adapters.
- `companion-app`: Swift drawing, camera, crop, connection, send/review status.

For the hackathon, the Mac can launch a Rust worker and exchange versioned structured messages over standard input/output. Keep logs on standard error, define framing, and recover from worker crashes. This avoids making embedded-language bindings a prerequisite. Large binary artifacts should use an explicit file or binary transport contract rather than being mixed into logs.

Extensibility means clear interfaces and compatibility tests. Do not build a general plugin runtime during the five-hour milestone.

Keep project sources independent of app metadata. Store insertion anchors and cache state in app storage or optional sidecar metadata. External edits must be detected without overwriting them.

Future collaboration needs revision-aware operations, stable identifiers, conflict handling, and shared document state. Record insert/replace operations against a base revision now. This helps a later design but is not equivalent to implementing a CRDT or collaborative editor.

## 7. Native interface

Mac layout: project sidebar, source editor, preview, and a compact contextual assistance panel. Make keyboard editing, undo/redo, and preview stability the polish priorities.

Use native source editing facilities through Swift/AppKit where needed; avoid trying to recreate a mature text editor in five hours. Add basic highlighting and completion for the compiler’s actual supported constructs.

The preview should preserve zoom and reading position between updates. Dark mode is a display transformation. Provide an original-colour toggle because inversion can distort photos and charts. Never bake preview-only colours into exported PDF content.

iPad: a large PencilKit canvas, photo capture/import, crop, context note, destination label, and Send. iPhone uses the same capture flow with emphasis on the camera. Camera testing must happen on a real device.

## 8. Nearby transfer

Initial transport: same-Wi-Fi connection with Bonjour discovery and explicit pairing. Network framework is a suitable Apple adapter. Keep application messages and state handling in Rust on Mac. A manual connection fallback is useful if discovery fails on the venue network.

Transfer completed captures instead of synchronizing individual Pencil strokes. Mac must be awake; queue locally if the connection drops. Do not add CloudKit or a relay service to this milestone.

Pairing should authenticate the chosen Mac; use an encrypted connection. Do not accept anonymous insertion requests from every device on the venue network.

Capture messages include protocol version, unique capture ID, project ID, destination ID, base revision, image metadata, and user instructions. Acknowledge receipt and completion separately. Retries must not duplicate insertion.

The Mac owns a pinned insertion anchor. Subsequent edits update that anchor. If its target is deleted or cannot be rebased safely, require destination reselection instead of inserting at an arbitrary current caret.

## 9. Grok conversion and assistance

Keep the API key on Mac in native credential storage. The companion sends captures to Mac, which sends the conversion request to Grok.

Context selection:

1. Include user instructions and the requested operation.
2. Include the destination environment and nearby source.
3. Include relevant macro definitions and package declarations within a bounded budget.
4. Include source diagnostics when explaining or fixing an error.
5. Offer an expandable view of what will be sent.

Default to faithful transcription. Do not silently correct mathematics or invent missing steps. Ask the model to mark ambiguities separately from LaTeX output. Model-reported confidence is not a calibrated probability.

Request structured output containing proposed LaTeX, ambiguities, and required dependencies. Validate the response and compile the proposal in the destination context before presenting it. Compilation proves syntax/layout acceptance, not mathematical correctness.

Review shows original capture, generated source, and rendered output, followed by one undoable insertion. If new packages or macros are required, propose those changes separately.

For the demo, give Grok the implemented feature boundary so it can prefer supported constructs. If faithful representation requires an unsupported feature, report that limitation rather than changing the meaning to make compilation pass.

Distinguish three capture outcomes:

- Equations and prose → editable LaTeX.
- Sketch → embedded image preserving the original.
- Sketch → proposed editable TikZ, requiring explicit review and actual compiler support.

Do not claim that a graph photograph yields its original underlying dataset.

Other initial AI actions: explain selected error and propose an edit to selected source. Apply all AI changes through the same revision-aware undoable edit mechanism.

## 10. Suggested five-hour execution budget

This is a high-risk allocation, not a delivery guarantee. Team size remains unspecified; it does not assume concurrent agents or developers.

| Elapsed time | Focus | Gate |
|---|---|---|
| 0:00–0:25 | Run Mac and companion shells; connect a Rust worker; freeze the sample and supported subset | Both Apple apps launch; Mac receives a Rust response |
| 0:25–1:40 | Minimal Rust compilation, layout, artifact generation, and source mappings | Actual source produces visible output and an exported PDF |
| 1:40–2:25 | Incremental cache, scheduling, diagnostics, preview interaction | An edit reuses measured work; one error recovers visibly |
| 2:25–3:35 | Pencil and camera capture, pairing, transfer, Grok request | Both real capture types reach Mac and produce a proposal |
| 3:35–4:15 | Review/insertion, anchor handling, dark preview, export verification | End-to-end capture → source → preview works |
| 4:15–5:00 | Benchmark, fix failures, rehearse, record limitations | Evidence accompanies every demonstrated claim |

If a gate fails, immediately reduce the prototype’s supported subset or polish scope and record what remains unfinished. Do not hide a fallback engine or claim the full product was completed. References, TikZ, and general font/layout behavior may exceed this budget even independently.

## 11. Acceptance evidence

- Open and save ordinary files without unwanted source transformations.
- Demonstrate original Rust compilation; record dependencies and execution path.
- List supported constructs and explicitly diagnose unsupported constructs.
- Compare warm edits against clean builds: incremental output must match clean output for the same revision.
- Include local edits, earlier-document edits, and state-changing edits in invalidation tests.
- Verify recoverable and unrecoverable errors, plus source navigation after edits.
- Measure end-to-end preview latency rather than compiler time alone.
- Test real Pencil and camera captures, a lost connection, a retried message, and a deleted destination.
- Verify that generated LaTeX preserves representative symbols, signs, indices, and matrix structure; review alongside the image.
- Verify dark preview does not change exported colours.

Long-term compatibility testing needs a public corpus, per-engine profiles, package/version coverage, and comparison against reference engines. Reference engines run in the development test harness only. Compare layout, text, fonts, references, diagnostics, and supported behavior; PDF byte equality is not a sufficient or universally appropriate criterion.

## 12. Research sources and their role

- [TeXpresso](https://github.com/let-def/texpresso): architectural inspiration for incremental live TeX, rollback, and recovery. Not an allowed engine backend under the founders’ decision.
- [Tectonic](https://github.com/tectonic-typesetting/tectonic): reference for Rust-facing TeX integration and dependency handling. Not a from-scratch pure Rust engine.
- [Typst](https://github.com/typst/typst): inspiration for incremental compiler design. Its language semantics do not provide LaTeX compatibility.
- [TeX Live guide](https://tug.org/texlive/doc/texlive-en/texlive-en.html): engine distinctions and the surrounding compatibility ecosystem.
- [Apple networking guidance](https://developer.apple.com/documentation/technotes/tn3151-choosing-the-right-networking-api): Bonjour and nearby transport options.
- [PencilKit](https://developer.apple.com/documentation/PencilKit): native drawing input.
- [Grok image understanding](https://docs.x.ai/developers/model-capabilities/images/understanding): image-plus-text request capability; not evidence of handwriting accuracy.

## 13. Decisions still unresolved

- Product name and team size; use neutral names and avoid assuming parallel staffing.
- Exact supported grammar and benchmark source; freeze before implementation.
- Exact meaning of from-scratch with respect to general-purpose dependencies; this plan permits infrastructure libraries but excludes existing TeX execution backends.
- Full compatibility profiles and the sequence in which they will be implemented.
- Minimum macOS/iOS versions, based on available test devices.
- Repository license, chosen with the actual dependency inventory.

None of these justify claiming unavailable functionality. The immediate engineering success is a truthful working slice of the eventual product, with the universal-compatibility work still visible.
