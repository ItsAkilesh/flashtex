import FlashTeXPadKit
import NearbyClient
import PencilKit
import PhotosUI
import SwiftUI

/// Primary screen: draw with Apple Pencil (or a finger in the simulator) or
/// pick a photo, add an instruction, and send it to the Mac as a transfer-v1
/// `capture_submit`. The Mac's bridge converts it (the configured conversion provider when a key exists,
/// else the deterministic local provider) into a reviewed LaTeX/TikZ proposal
/// and the Mac user approves insertion; the iPad only receives the receipt.
struct CaptureView: View {
    @EnvironmentObject var model: PadModel
    @State private var drawing = PKDrawing()
    @State private var canvasSize = CGSize(width: 1024, height: 640)
    @State private var photo: PhotosPickerItem?
    @State private var picked: UIImage?
    @State private var pickedSource: CaptureRecord.Source = .photo
    @State private var instructions = "Convert this drawing to TikZ"
    @State private var draft: CaptureRecord?
    @State private var problem: String?
    @State private var toolsVisible = true

    var body: some View {
        VStack(spacing: 8) {
            HStack {
                Text(model.link.isConnected ? "Connected to \(model.pairedMac?.macName ?? "Mac")"
                     : "Not connected — \(model.linkStatus)\(model.linkError.map { ": \($0)" } ?? "") — pair in Mac link")
                    .font(.footnote).foregroundStyle(model.link.isConnected ? .green : .secondary)
                    .accessibilityIdentifier("capture.connection")
                Spacer()
                if let d = model.destination {
                    Text("destination \(d.destinationId) @ \(d.path) rev \(d.baseRevision)").font(.footnote.monospaced()).foregroundStyle(.secondary)
                } else {
                    Text("no insertion point pinned on the Mac").font(.footnote).foregroundStyle(.secondary)
                }
            }.padding(.horizontal)

            if let img = picked {
                Image(uiImage: img).resizable().scaledToFit().frame(maxHeight: 360)
                    .overlay(alignment: .topTrailing) {
                        Button("Back to canvas") { picked = nil }.buttonStyle(.bordered).padding(6)
                    }
                    .accessibilityIdentifier("capture.pickedImage")
            } else {
                PencilCanvas(drawing: $drawing, size: $canvasSize, toolsVisible: $toolsVisible)
                    .frame(minHeight: model.captures.isEmpty ? 320 : 200, maxHeight: model.captures.isEmpty ? .infinity : 200)
                    .background(Color.white)
                    .overlay(RoundedRectangle(cornerRadius: 6).stroke(.gray.opacity(0.4)))
                    .padding(.horizontal)
                    .accessibilityIdentifier("capture.canvas")
            }

            HStack {
                Button { drawing = PKDrawing(); picked = nil; toolsVisible = true } label: { Label("Clear", systemImage: "trash") }
                    .accessibilityIdentifier("capture.clear")
                PhotosPicker(selection: $photo, matching: .images) { Label("Photo…", systemImage: "photo") }
                    .accessibilityIdentifier("capture.photo")
                Button { loadSample() } label: { Label("Sample image", systemImage: "photo.on.rectangle") }
                    .accessibilityIdentifier("capture.sample")
                Spacer()
                Text("\(drawing.strokes.count) stroke\(drawing.strokes.count == 1 ? "" : "s")").font(.footnote).foregroundStyle(.secondary)
                    .accessibilityIdentifier("capture.strokes")
            }.padding(.horizontal)

            TextField("Instruction for the Mac (≤ 4096 bytes), e.g. “convert this to TikZ”, “this is a matrix”", text: $instructions, axis: .vertical)
                .textFieldStyle(.roundedBorder).padding(.horizontal)
                .accessibilityIdentifier("capture.instructions")

            HStack {
                Button { prepare() } label: { Label("Prepare capture", systemImage: "square.and.arrow.up") }
                    .buttonStyle(.bordered).accessibilityIdentifier("capture.prepare")
                if let d = draft {
                    Text("\(d.id.prefix(12))… \(d.png.count) PNG bytes" + (d.pixelSize.map { " \($0.width)×\($0.height)" } ?? ""))
                        .font(.footnote.monospaced()).accessibilityIdentifier("capture.draft")
                    Button("Discard") { model.discard(d.id); draft = nil }
                        .buttonStyle(.bordered).accessibilityIdentifier("capture.discard")
                    Button { let id = d.id; draft = nil; Task { await model.send(id) } } label: { Label("Send to Mac", systemImage: "paperplane.fill") }
                        .buttonStyle(.borderedProminent).disabled(!model.link.isConnected)
                        .accessibilityIdentifier("capture.send")
                }
                Spacer()
            }.padding(.horizontal)
            if let p = problem { Text(p).foregroundStyle(.red).font(.footnote).padding(.horizontal).accessibilityIdentifier("capture.problem") }

            Divider()
            CapturesList()
        }
        .padding(.vertical, 8)
        .navigationTitle("Capture")
        .onChange(of: photo) { _, item in
            guard let item else { return }
            Task {
                if let data = try? await item.loadTransferable(type: Data.self), let img = UIImage(data: data) {
                    picked = img; pickedSource = .photo
                }
            }
        }
    }

    func loadSample() {
        guard let url = Bundle.main.url(forResource: "sample-capture", withExtension: "png"), let data = try? Data(contentsOf: url),
              let img = UIImage(data: data) else { problem = "sample-capture.png missing from bundle"; return }
        picked = img; pickedSource = .sample
    }

    /// Renders the canvas (or the picked image) to PNG and drafts a record;
    /// nothing is sent until "Send to Mac".
    func prepare() {
        problem = nil
        let png: Data
        let source: CaptureRecord.Source
        let size: (Int, Int)
        if let img = picked {
            guard let d = img.pngData() else { problem = "could not encode the image as PNG"; return }
            png = d; source = pickedSource; size = (Int(img.size.width * img.scale), Int(img.size.height * img.scale))
        } else {
            guard !drawing.strokes.isEmpty else { problem = "draw something first (or pick a photo)"; return }
            let bounds = CGRect(origin: .zero, size: canvasSize)
            let img = drawing.image(from: bounds, scale: 2)
            guard let d = img.pngData() else { problem = "could not encode the drawing as PNG"; return }
            png = d; source = .pencil; size = (Int(img.size.width * img.scale), Int(img.size.height * img.scale))
        }
        if let why = CaptureQueue.validate(png: png, instructions: instructions) { problem = why; return }
        toolsVisible = false // hide the PencilKit tool picker so the status list is readable
        draft = model.draft(CaptureRecord(source: source, png: png, instructions: instructions, pixelSize: (width: size.0, height: size.1)))
    }
}

struct PencilCanvas: UIViewRepresentable {
    @Binding var drawing: PKDrawing
    @Binding var size: CGSize
    @Binding var toolsVisible: Bool

    func makeUIView(context: Context) -> PKCanvasView {
        let v = PKCanvasView()
        v.drawingPolicy = .anyInput // finger works in the simulator; Pencil on hardware
        v.tool = PKInkingTool(.pen, color: .black, width: 4)
        v.backgroundColor = .white
        v.delegate = context.coordinator
        v.isAccessibilityElement = true
        v.accessibilityIdentifier = "capture.canvasView"
        v.drawing = drawing
        let picker = PKToolPicker()
        picker.setVisible(true, forFirstResponder: v)
        picker.addObserver(v)
        context.coordinator.picker = picker
        DispatchQueue.main.async { v.becomeFirstResponder() }
        return v
    }

    func updateUIView(_ v: PKCanvasView, context: Context) {
        if v.drawing != drawing { v.drawing = drawing }
        if let picker = context.coordinator.picker, picker.isVisible != toolsVisible {
            picker.setVisible(toolsVisible, forFirstResponder: v)
            if toolsVisible { DispatchQueue.main.async { v.becomeFirstResponder() } } else { DispatchQueue.main.async { v.resignFirstResponder() } }
        }
        DispatchQueue.main.async { if v.bounds.size != .zero, size != v.bounds.size { size = v.bounds.size } }
    }

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    final class Coordinator: NSObject, PKCanvasViewDelegate {
        var parent: PencilCanvas
        var picker: PKToolPicker?
        init(_ p: PencilCanvas) { parent = p }
        func canvasViewDrawingDidChange(_ v: PKCanvasView) { parent.drawing = v.drawing }
    }
}

struct CapturesList: View {
    @EnvironmentObject var model: PadModel

    var body: some View {
        List {
            Section("Captures (\(model.captures.count))") {
                if model.captures.isEmpty { Text("none yet").foregroundStyle(.secondary) }
                ForEach(model.captures) { c in
                    HStack(alignment: .top) {
                        if let img = UIImage(data: c.png) {
                            Image(uiImage: img).resizable().scaledToFit().frame(width: 72, height: 54).background(Color.white).border(.gray.opacity(0.3))
                        }
                        VStack(alignment: .leading, spacing: 2) {
                            Text(c.id).font(.caption.monospaced())
                            Text("\(c.source.rawValue) · \(c.png.count) bytes · “\(c.instructions)”").font(.caption).foregroundStyle(.secondary)
                            Text(c.status.label).font(.footnote).accessibilityIdentifier("capture.status.\(c.id)")
                            if case .received(let r) = c.status {
                                Text(verbatim: "capture_received capture_id=\(r.captureId) durable=\(r.durable) has_proposal=\(r.hasProposal) applied=\(r.applied)")
                                    .font(.caption2.monospaced()).foregroundStyle(.secondary)
                                if let d = c.destinationId, let rev = c.baseRevision {
                                    Text("sent for destination \(d) base_revision \(rev)").font(.caption2.monospaced()).foregroundStyle(.secondary)
                                }
                                if let label = c.outcomeLabel {
                                    Text("Mac: \(label)").font(.footnote).foregroundStyle(c.outcomeIsFinal ? .primary : .secondary)
                                        .accessibilityIdentifier("capture.outcome.\(c.id)")
                                    if let o = c.outcome {
                                        Text(verbatim: "capture_status_ack state=\(o.state) durable=\(o.durable)" + (o.note.map { " note=“\($0)”" } ?? ""))
                                            .font(.caption2.monospaced()).foregroundStyle(.secondary)
                                    }
                                } else {
                                    Text("Mac: waiting for the first capture_status reply…").font(.footnote).foregroundStyle(.secondary)
                                        .accessibilityIdentifier("capture.outcome.\(c.id)")
                                }
                                if let latex = c.outcome?.latex {
                                    Text("Returned LaTeX/TikZ (read-only; approve or reject on the Mac):").font(.caption2).foregroundStyle(.secondary)
                                    Text(latex).font(.caption.monospaced()).padding(6)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .background(Color.gray.opacity(0.12)).cornerRadius(4)
                                        .accessibilityIdentifier("capture.latex.\(c.id)")
                                        .accessibilityLabel("returned latex \(latex)")
                                }
                                if !c.outcomeIsFinal {
                                    Button("Refresh status") { Task { await model.refreshOutcome(c.id) } }.buttonStyle(.bordered).font(.caption)
                                        .disabled(!model.link.isConnected).accessibilityIdentifier("capture.refresh.\(c.id)")
                                }
                            }
                            if case .refused(_, let msg) = c.status { Text(msg).font(.caption2).foregroundStyle(.red) }
                            if case .disconnected = c.status {
                                Button("Retry (same capture_id)") { Task { await model.send(c.id) } }.buttonStyle(.bordered).font(.caption)
                                    .accessibilityIdentifier("capture.retry.\(c.id)")
                            }
                            if case .drafted = c.status {
                                HStack {
                                    Button("Send") { Task { await model.send(c.id) } }.buttonStyle(.borderedProminent).font(.caption).disabled(!model.link.isConnected)
                                    Button("Discard") { model.discard(c.id) }.buttonStyle(.bordered).font(.caption)
                                }
                            }
                        }
                    }
                    .accessibilityIdentifier("capture.row.\(c.id)")
                }
            }
            Section {
                Text("Status is what nearby-v1 returns to a companion: the capture_received receipt (durable / has_proposal / applied at receipt time) or an error code, then the Mac-side outcome from capture_status (journaled → converting → proposal ready → inserted / rejected / failed) polled every 2 s until final. The LaTeX/TikZ is shown read-only: approval and insertion stay on the Mac. Retry after a disconnect re-sends the same capture_id; the Mac de-duplicates. Drafts, receipts and outcomes are kept on this iPad across relaunches.")
                    .font(.caption).foregroundStyle(.secondary)
            }
        }
        .accessibilityIdentifier("captures.list")
    }
}
