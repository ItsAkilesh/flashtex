import SwiftUI
import PencilKit

/// PencilKit-based drawing canvas for Apple Pencil input.
/// Exports the drawing as a PNG capture_submit payload.
struct DrawingCanvasView: View {
    @Environment(CaptureStore.self) private var store
    @State private var canvasView = PKCanvasView()
    @State private var showingExport = false

    var body: some View {
        VStack(spacing: 0) {
            PencilCanvasRepresentable(canvasView: $canvasView)
                .background(Color.white)
                .clipShape(RoundedRectangle(cornerRadius: 12))
                .padding()

            HStack(spacing: 16) {
                Button(action: clearCanvas) {
                    Label("Clear", systemImage: "trash")
                        .font(.headline)
                }
                .buttonStyle(.bordered)

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

    private func captureDrawing() {
        let bounds = canvasView.drawing.bounds
        guard !bounds.isEmpty else { return }

        let padding: CGFloat = 20
        let exportRect = bounds.insetBy(dx: -padding, dy: -padding)
        let image = canvasView.drawing.image(from: exportRect, scale: 2.0)
        store.addCapture(source: .pencil, image: image)
    }
}

/// UIKit wrapper for PKCanvasView.
struct PencilCanvasRepresentable: UIViewRepresentable {
    @Binding var canvasView: PKCanvasView

    func makeUIView(context: Context) -> PKCanvasView {
        canvasView.drawingPolicy = .anyInput
        canvasView.tool = PKInkingTool(.pen, color: .black, width: 3)
        canvasView.backgroundColor = .white
        canvasView.isOpaque = true
        return canvasView
    }

    func updateUIView(_ uiView: PKCanvasView, context: Context) {}
}
