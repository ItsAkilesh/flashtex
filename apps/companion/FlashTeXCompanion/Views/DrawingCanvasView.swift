import SwiftUI
import PencilKit

/// PencilKit-based drawing canvas for Apple Pencil input.
/// Exports the drawing as a PNG capture_submit payload.
struct DrawingCanvasView: View {
    @Environment(CaptureStore.self) private var store
    @State private var canvasView = PKCanvasView()
    @State private var toolPickerVisible = true

    var body: some View {
        VStack(spacing: 0) {
            PencilCanvasRepresentable(canvasView: $canvasView,
                                      toolPickerVisible: $toolPickerVisible)
                .background(Color.white)
                .clipShape(RoundedRectangle(cornerRadius: 12))
                .padding()

            if let error = store.lastError {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .padding(.horizontal)
            }

            HStack(spacing: 16) {
                Button(action: clearCanvas) {
                    Label("Clear", systemImage: "trash")
                        .font(.headline)
                }
                .buttonStyle(.bordered)

                Button(action: undoStroke) {
                    Label("Undo", systemImage: "arrow.uturn.backward")
                        .font(.headline)
                }
                .buttonStyle(.bordered)
                .disabled(canvasView.drawing.strokes.isEmpty)

                Button(action: captureDrawing) {
                    Label("Capture", systemImage: "arrow.up.circle.fill")
                        .font(.headline)
                }
                .buttonStyle(.borderedProminent)
                .disabled(canvasView.drawing.strokes.isEmpty)
            }
            .padding(.bottom)
        }
        .navigationTitle("Pencil Drawing")
    }

    private func clearCanvas() {
        canvasView.drawing = PKDrawing()
    }

    private func undoStroke() {
        var strokes = canvasView.drawing.strokes
        if !strokes.isEmpty {
            strokes.removeLast()
            canvasView.drawing = PKDrawing(strokes: Array(strokes))
        }
    }

    private func captureDrawing() {
        let bounds = canvasView.drawing.bounds
        guard !bounds.isEmpty else { return }

        let padding: CGFloat = 20
        let exportRect = bounds.insetBy(dx: -padding, dy: -padding)
        let image = canvasView.drawing.image(from: exportRect, scale: 2.0)
        store.addCapture(source: .pencil, image: image)
    }
}

/// UIKit wrapper for PKCanvasView with tool picker integration.
struct PencilCanvasRepresentable: UIViewRepresentable {
    @Binding var canvasView: PKCanvasView
    @Binding var toolPickerVisible: Bool

    func makeUIView(context: Context) -> PKCanvasView {
        canvasView.drawingPolicy = .anyInput
        canvasView.tool = PKInkingTool(.pen, color: .black, width: 3)
        canvasView.backgroundColor = .white
        canvasView.isOpaque = true

        // Show tool picker
        let toolPicker = PKToolPicker()
        toolPicker.setVisible(true, forFirstResponder: canvasView)
        toolPicker.addObserver(canvasView)
        canvasView.becomeFirstResponder()
        context.coordinator.toolPicker = toolPicker

        return canvasView
    }

    func updateUIView(_ uiView: PKCanvasView, context: Context) {}

    func makeCoordinator() -> Coordinator { Coordinator() }

    class Coordinator {
        var toolPicker: PKToolPicker?
    }
}
