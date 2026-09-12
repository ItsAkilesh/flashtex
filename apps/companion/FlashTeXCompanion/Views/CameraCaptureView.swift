import SwiftUI
import PhotosUI

/// Camera and photo library capture view.
/// Produces the same capture_submit payload as PencilKit drawing.
struct CameraCaptureView: View {
    @Environment(CaptureStore.self) private var store

    @State private var showCamera = false
    @State private var selectedPhoto: PhotosPickerItem?
    @State private var contextText: String = ""

    // Review-before-send: show the sheet when a pending capture exists.
    private var showReview: Binding<Bool> {
        Binding(
            get: { store.pendingCapture != nil },
            set: { if !$0 { store.discardPending() } }
        )
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 24) {
                Spacer(minLength: 16)

                Image(systemName: "camera.viewfinder")
                    .font(.system(size: 80))
                    .foregroundStyle(.secondary)
                    .accessibilityLabel("Camera capture icon")
                    .accessibilityHidden(true)   // decorative

                Text("Capture handwriting or equations from paper")
                    .font(.title3)
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal)

                if let error = store.lastError {
                    Text(error)
                        .font(.caption)
                        .foregroundStyle(.red)
                        .padding(.horizontal)
                        .accessibilityLabel("Error: \(error)")
                }

                // MARK: Context input
                VStack(alignment: .leading, spacing: 6) {
                    Text("Context (optional)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    TextField(
                        "e.g. "Physics problem set 3, page 2"",
                        text: $contextText,
                        axis: .vertical
                    )
                    .lineLimit(2...4)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityLabel("Capture context")
                    .accessibilityHint("Optional note forwarded with the capture to help the runtime understand what it's looking at")
                }
                .padding(.horizontal)

                // MARK: Destination summary
                DestinationSummaryView()
                    .padding(.horizontal)

                // MARK: Action buttons
                VStack(spacing: 12) {
                    Button(action: { showCamera = true }) {
                        Label("Take Photo", systemImage: "camera.fill")
                            .frame(maxWidth: .infinity)
                            .font(.headline)
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)
                    .accessibilityLabel("Take Photo")
                    .accessibilityHint("Opens the camera to capture handwriting or an equation")

                    PhotosPicker(selection: $selectedPhoto, matching: .images) {
                        Label("Choose from Library", systemImage: "photo.on.rectangle")
                            .frame(maxWidth: .infinity)
                            .font(.headline)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                    .accessibilityLabel("Choose from Library")
                    .accessibilityHint("Opens the photo library to choose an existing image")
                }
                .padding(.horizontal, 40)

                Spacer(minLength: 16)
            }
        }
        .navigationTitle("Camera Capture")
        // Full-screen camera sheet.
        .fullScreenCover(isPresented: $showCamera) {
            CameraRepresentable { image in
                if let image {
                    let instructions = contextText.trimmingCharacters(in: .whitespaces)
                    store.stagePendingCapture(
                        source: .camera,
                        image: image,
                        instructions: instructions.isEmpty
                            ? "Faithfully transcribe this handwriting or equation; preserve all notation."
                            : instructions
                    )
                }
            }
            .ignoresSafeArea()
        }
        // Photo library selection.
        .onChange(of: selectedPhoto) { _, newValue in
            guard let item = newValue else { return }
            Task {
                if let data = try? await item.loadTransferable(type: Data.self),
                   let image = UIImage(data: data) {
                    let instructions = contextText.trimmingCharacters(in: .whitespaces)
                    store.stagePendingCapture(
                        source: .photoLibrary,
                        image: image,
                        instructions: instructions.isEmpty
                            ? "Faithfully transcribe this handwriting or equation; preserve all notation."
                            : instructions
                    )
                }
                selectedPhoto = nil
            }
        }
        // Review-before-send sheet.
        .sheet(isPresented: showReview) {
            if let pending = store.pendingCapture {
                CaptureReviewSheet(pending: pending) {
                    store.confirmAndSend()
                } onDiscard: {
                    store.discardPending()
                }
            }
        }
    }
}

// MARK: – UIKit camera wrapper

/// UIKit camera wrapper.
struct CameraRepresentable: UIViewControllerRepresentable {
    let completion: (UIImage?) -> Void

    func makeUIViewController(context: Context) -> UIImagePickerController {
        let picker = UIImagePickerController()
        picker.sourceType = .camera
        picker.delegate = context.coordinator
        return picker
    }

    func updateUIViewController(_ uiViewController: UIImagePickerController, context: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(completion: completion)
    }

    class Coordinator: NSObject, UIImagePickerControllerDelegate, UINavigationControllerDelegate {
        let completion: (UIImage?) -> Void
        init(completion: @escaping (UIImage?) -> Void) { self.completion = completion }

        func imagePickerController(
            _ picker: UIImagePickerController,
            didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]
        ) {
            let image = info[.originalImage] as? UIImage
            picker.dismiss(animated: true) { self.completion(image) }
        }

        func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
            picker.dismiss(animated: true) { self.completion(nil) }
        }
    }
}
