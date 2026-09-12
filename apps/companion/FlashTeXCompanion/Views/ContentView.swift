import SwiftUI

/// Root tab view for FlashTeX Companion.
struct ContentView: View {
    @State private var store = CaptureStore()

    var body: some View {
        TabView {
            NavigationStack {
                DrawingCanvasView()
            }
            .tabItem {
                Label("Draw", systemImage: "pencil.tip")
            }

            NavigationStack {
                CameraCaptureView()
            }
            .tabItem {
                Label("Camera", systemImage: "camera")
            }

            NavigationStack {
                CaptureHistoryView()
            }
            .tabItem {
                Label("History", systemImage: "clock")
            }

            NavigationStack {
                SettingsView()
            }
            .tabItem {
                Label("Settings", systemImage: "gear")
            }
        }
        .environment(store)
        .sheet(isPresented: Binding(
            get: { store.showPayloadPreview },
            set: { store.showPayloadPreview = $0 }
        )) {
            if let json = store.lastPayloadJSON {
                PayloadPreviewView(json: json)
            }
        }
    }
}
