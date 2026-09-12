import SwiftUI
import PencilKit

/// Full-screen PencilKit drawing canvas with context note and destination label.
/// Matches product spec: "large PencilKit canvas, context note, destination label, and Send."
struct DrawingCanvasView: View {
    @Environment(CaptureStore.self) private var store
    @State private var canvasView = PKCanvasView()
    @State private var instructions: String = ""
    @State private var showInstructionsField = false

    var body: some View {
        VStack(spacing: 0) {
            // Destination strip
            DestinationStrip()
                .padding(.horizontal)
                .padding(.top, 4)

            // Canvas — white, full-width, prominent
            ZStack(alignment: .topTrailing) {
                PencilCanvasRepresentable(canvasView: $canvasView)
                    .background(Color.white)
                    .clipShape(RoundedRectangle(cornerRadius: 12))
                    .shadow(color: .black.opacity(0.08), radius: 4, x: 0, y: 2)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)

                // Stroke count badge
                if !canvasView.drawing.strokes.isEmpty {
                    Text("\(canvasView.drawing.strokes.count)")
                        .font(.caption2.monospacedDigit())
                        .foregroundStyle(.secondary)
                        .padding(6)
                }
            }
            .frame(maxHeight: .infinity)

            // Error
            if let error = store.lastError {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundStyle(.red)
                    Text(error).font(.caption).foregroundStyle(.red)
                }
                .padding(.horizontal)
            }

            // Instructions field (collapsible)
            if showInstructionsField {
                HStack(alignment: .top, spacing: 8) {
                    Image(systemName: "text.bubble")
                        .foregroundStyle(.secondary)
                        .padding(.top, 4)
                    TextField("Context note (e.g. 'preserve matrix notation')", text: $instructions, axis: .vertical)
                        .font(.callout)
                        .lineLimit(2...4)
                        .textFieldStyle(.plain)
                }
                .padding(10)
                .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 10))
                .padding(.horizontal, 12)
            }

            // Action bar
            HStack(spacing: 12) {
                Button(action: clearCanvas) {
                    Image(systemName: "trash")
                }
                .buttonStyle(.bordered)
                .disabled(canvasView.drawing.strokes.isEmpty)

                Button(action: undoStroke) {
                    Image(systemName: "arrow.uturn.backward")
                }
                .buttonStyle(.bordered)
                .disabled(canvasView.drawing.strokes.isEmpty)

                // Instructions toggle
                Button(action: { withAnimation { showInstructionsField.toggle() } }) {
                    Image(systemName: showInstructionsField ? "text.bubble.fill" : "text.bubble")
                }
                .buttonStyle(.bordered)
                .tint(showInstructionsField ? .accentColor : nil)

                Spacer()

                Button(action: captureDrawing) {
                    Label("Send", systemImage: "arrow.up.circle.fill")
                        .font(.headline)
                }
                .buttonStyle(.borderedProminent)
                .disabled(canvasView.drawing.strokes.isEmpty)
            }
            .padding(.horizontal, 16)
            .padding(.bottom, 8)
        }
        .navigationTitle("Pencil Drawing")
        .navigationBarTitleDisplayMode(.inline)
    }

    private func clearCanvas() {
        canvasView.drawing = PKDrawing()
    }

    private func undoStroke() {
        var strokes = canvasView.drawing.strokes
        guard !strokes.isEmpty else { return }
        strokes.removeLast()
        canvasView.drawing = PKDrawing(strokes: Array(strokes))
    }

    private func captureDrawing() {
        let bounds = canvasView.drawing.bounds
        guard !bounds.isEmpty else { return }
        let exportRect = bounds.insetBy(dx: -24, dy: -24)
        let image = canvasView.drawing.image(from: exportRect, scale: 2.0)
        let note = instructions.trimmingCharacters(in: .whitespacesAndNewlines)
        let effectiveInstructions = note.isEmpty
            ? "Faithfully transcribe this handwriting or equation; preserve all notation."
            : note
        store.addCapture(source: .pencil, image: image, instructions: effectiveInstructions)
    }
}

/// Shows the currently pinned destination ID and base revision.
/// Tapping opens Settings to change it.
struct DestinationStrip: View {
    @Environment(CaptureStore.self) private var store

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: "mappin.circle.fill")
                .foregroundStyle(.accentColor)
                .font(.caption)
            Text(store.currentDestinationID)
                .font(.caption.weight(.medium))
                .lineLimit(1)
                .foregroundStyle(.secondary)
            Text("rev \(store.currentBaseRevision)")
                .font(.caption2)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .padding(.vertical, 4)
    }
}

/// UIKit wrapper for PKCanvasView with persistent PKToolPicker.
struct PencilCanvasRepresentable: UIViewRepresentable {
    @Binding var canvasView: PKCanvasView

    func makeUIView(context: Context) -> PKCanvasView {
        canvasView.drawingPolicy = .anyInput
        canvasView.tool = PKInkingTool(.pen, color: .black, width: 3)
        canvasView.backgroundColor = .white
        canvasView.isOpaque = true

        let picker = PKToolPicker()
        picker.setVisible(true, forFirstResponder: canvasView)
        picker.addObserver(canvasView)
        canvasView.becomeFirstResponder()
        context.coordinator.toolPicker = picker

        return canvasView
    }

    func updateUIView(_ uiView: PKCanvasView, context: Context) {}
    func makeCoordinator() -> Coordinator { Coordinator() }

    class Coordinator {
        var toolPicker: PKToolPicker?
    }
}
