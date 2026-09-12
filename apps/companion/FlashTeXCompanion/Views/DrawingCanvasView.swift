import SwiftUI
import PencilKit

/// Full-screen PencilKit drawing canvas with context note and destination label.
/// Matches product spec: "large PencilKit canvas, context note, destination label, and Send."
///
/// Rev 3 changes:
/// - `Send` now calls `stagePendingCapture` so the user reviews before dispatch.
/// - `DestinationStrip` retained; a `DestinationSummaryView` sheet is accessible from Settings.
/// - Accessibility labels and hints on all interactive controls.
struct DrawingCanvasView: View {
    @Environment(CaptureStore.self) private var store
    @State private var canvasView = PKCanvasView()
    @State private var instructions: String = ""
    @State private var showInstructionsField = false

    // Review-before-send: sheet appears when a pending capture exists.
    private var showReview: Binding<Bool> {
        Binding(
            get: { store.pendingCapture != nil },
            set: { if !$0 { store.discardPending() } }
        )
    }

    var body: some View {
        VStack(spacing: 0) {
            // Destination strip
            DestinationStrip()
                .padding(.horizontal)
                .padding(.top, 4)
                .accessibilityElement(children: .combine)
                .accessibilityLabel("Destination: \(store.currentDestinationID), revision \(store.currentBaseRevision)")

            // Canvas — white, full-width, prominent
            ZStack(alignment: .topTrailing) {
                PencilCanvasRepresentable(canvasView: $canvasView)
                    .background(Color.white)
                    .clipShape(RoundedRectangle(cornerRadius: 12))
                    .shadow(color: .black.opacity(0.08), radius: 4, x: 0, y: 2)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .accessibilityLabel("Drawing canvas")
                    .accessibilityHint("Draw handwriting or equations here using Apple Pencil or your finger")

                // Stroke count badge
                if !canvasView.drawing.strokes.isEmpty {
                    Text("\(canvasView.drawing.strokes.count)")
                        .font(.caption2.monospacedDigit())
                        .foregroundStyle(.secondary)
                        .padding(6)
                        .accessibilityLabel("\(canvasView.drawing.strokes.count) strokes")
                }
            }
            .frame(maxHeight: .infinity)

            // Error
            if let error = store.lastError {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundStyle(.red)
                        .accessibilityHidden(true)
                    Text(error).font(.caption).foregroundStyle(.red)
                }
                .padding(.horizontal)
                .accessibilityLabel("Error: \(error)")
            }

            // Instructions field (collapsible)
            if showInstructionsField {
                HStack(alignment: .top, spacing: 8) {
                    Image(systemName: "text.bubble")
                        .foregroundStyle(.secondary)
                        .padding(.top, 4)
                        .accessibilityHidden(true)
                    TextField(
                        "Context note (e.g. 'preserve matrix notation')",
                        text: $instructions,
                        axis: .vertical
                    )
                    .font(.callout)
                    .lineLimit(2...4)
                    .textFieldStyle(.plain)
                    .accessibilityLabel("Context note")
                    .accessibilityHint("Optional description forwarded with the capture; helps the runtime interpret the content")
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
                .accessibilityLabel("Clear canvas")
                .accessibilityHint("Removes all strokes from the drawing")

                Button(action: undoStroke) {
                    Image(systemName: "arrow.uturn.backward")
                }
                .buttonStyle(.bordered)
                .disabled(canvasView.drawing.strokes.isEmpty)
                .accessibilityLabel("Undo last stroke")
                .accessibilityHint("Removes the most recent stroke")

                // Instructions toggle
                Button(action: { withAnimation { showInstructionsField.toggle() } }) {
                    Image(systemName: showInstructionsField ? "text.bubble.fill" : "text.bubble")
                }
                .buttonStyle(.bordered)
                .tint(showInstructionsField ? .accentColor : nil)
                .accessibilityLabel(showInstructionsField ? "Hide context field" : "Add context note")
                .accessibilityHint("Toggles a text field for describing what is drawn")

                Spacer()

                Button(action: stageDrawing) {
                    Label("Review & Send", systemImage: "arrow.up.circle.fill")
                        .font(.headline)
                }
                .buttonStyle(.borderedProminent)
                .disabled(canvasView.drawing.strokes.isEmpty)
                .accessibilityLabel("Review and Send")
                .accessibilityHint("Opens a preview of the capture payload so you can confirm before sending")
            }
            .padding(.horizontal, 16)
            .padding(.bottom, 8)
        }
        .navigationTitle("Pencil Drawing")
        .navigationBarTitleDisplayMode(.inline)
        // Review-before-send sheet.
        .sheet(isPresented: showReview) {
            if let pending = store.pendingCapture {
                CaptureReviewSheet(pending: pending) {
                    if store.confirmAndSend() {
                        // Clear canvas on successful send.
                        canvasView.drawing = PKDrawing()
                        instructions = ""
                        showInstructionsField = false
                    }
                } onDiscard: {
                    store.discardPending()
                }
            }
        }
    }

    // MARK: Private helpers

    private func clearCanvas() {
        canvasView.drawing = PKDrawing()
    }

    private func undoStroke() {
        var strokes = canvasView.drawing.strokes
        guard !strokes.isEmpty else { return }
        strokes.removeLast()
        canvasView.drawing = PKDrawing(strokes: Array(strokes))
    }

    private func stageDrawing() {
        let bounds = canvasView.drawing.bounds
        guard !bounds.isEmpty else { return }
        let exportRect = bounds.insetBy(dx: -24, dy: -24)
        let image = canvasView.drawing.image(from: exportRect, scale: 2.0)
        let note = instructions.trimmingCharacters(in: .whitespacesAndNewlines)
        let effectiveInstructions = note.isEmpty
            ? "Faithfully transcribe this handwriting or equation; preserve all notation."
            : note
        store.stagePendingCapture(source: .pencil, image: image, instructions: effectiveInstructions)
    }
}

// MARK: – DestinationStrip

/// Shows the currently pinned destination ID and base revision.
struct DestinationStrip: View {
    @Environment(CaptureStore.self) private var store

    var body: some View {
        HStack(spacing: 6) {
            Image(systemName: "mappin.circle.fill")
                .foregroundStyle(.accentColor)
                .font(.caption)
                .accessibilityHidden(true)
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

// MARK: – DestinationSummaryView

/// Compact destination card used in CameraCaptureView.
struct DestinationSummaryView: View {
    @Environment(CaptureStore.self) private var store

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "mappin.circle.fill")
                .foregroundStyle(.accentColor)
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 2) {
                Text("Sending to:")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
                Text(store.currentDestinationID)
                    .font(.caption.weight(.medium))
                    .lineLimit(1)
            }
            Spacer()
            Text("rev \(store.currentBaseRevision)")
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
        .padding(10)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 10))
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Sending to \(store.currentDestinationID), revision \(store.currentBaseRevision). Change in Settings.")
    }
}

// MARK: – CaptureReviewSheet

/// Modal sheet shown between staging and sending.
/// The user can inspect the payload JSON, then confirm or discard.
struct CaptureReviewSheet: View {
    let pending: CaptureStore.PendingCapture
    let onConfirm: () -> Void
    let onDiscard: () -> Void

    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    // Thumbnail
                    HStack {
                        Image(uiImage: pending.thumbnail)
                            .resizable()
                            .scaledToFill()
                            .frame(width: 72, height: 72)
                            .clipShape(RoundedRectangle(cornerRadius: 8))
                            .accessibilityLabel("Capture thumbnail")

                        VStack(alignment: .leading, spacing: 4) {
                            Text(pending.source.rawValue)
                                .font(.headline)
                            Text("→ \(pending.destinationID)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            Text("rev \(pending.baseRevision)")
                                .font(.caption2)
                                .foregroundStyle(.tertiary)
                        }
                        Spacer()
                    }
                    .padding()
                    .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 12))

                    // Payload JSON preview
                    GroupBox("Payload preview") {
                        ScrollView(.horizontal, showsIndicators: false) {
                            Text(pending.payloadJSON)
                                .font(.system(.caption2, design: .monospaced))
                                .foregroundStyle(.secondary)
                                .textSelection(.enabled)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .frame(maxHeight: 160)
                    }
                    .accessibilityLabel("Payload JSON preview")

                    // Capture ID
                    Label("Capture ID: \(pending.captureID)", systemImage: "number")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .padding()
            }
            .navigationTitle("Review Capture")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Discard", role: .destructive) {
                        onDiscard()
                        dismiss()
                    }
                    .accessibilityLabel("Discard capture")
                    .accessibilityHint("Cancels this capture without sending anything")
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Send") {
                        onConfirm()
                        dismiss()
                    }
                    .fontWeight(.semibold)
                    .accessibilityLabel("Send capture")
                    .accessibilityHint("Sends the capture payload to \(pending.destinationID)")
                }
            }
        }
    }
}

// MARK: – PencilCanvasRepresentable

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
