import SwiftUI

/// Root view. Shows connection status banner at top, then tab content.
struct ContentView: View {
    @State private var store = CaptureStore()
    @State private var transport = BonjourTransport.shared

    var body: some View {
        VStack(spacing: 0) {
            ConnectionBannerView()

            TabView {
                NavigationStack {
                    DrawingCanvasView()
                }
                .tabItem { Label("Draw", systemImage: "pencil.tip") }

                NavigationStack {
                    CameraCaptureView()
                }
                .tabItem { Label("Camera", systemImage: "camera") }

                NavigationStack {
                    CaptureHistoryView()
                }
                .tabItem { Label("History", systemImage: "clock") }

                NavigationStack {
                    SettingsView()
                }
                .tabItem { Label("Settings", systemImage: "gear") }
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
        .onAppear {
            BonjourTransport.shared.startBrowsing()
        }
    }
}

/// Slim status bar showing Wi-Fi connection state to the Mac.
struct ConnectionBannerView: View {
    @State private var transport = BonjourTransport.shared

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(dotColor)
                .frame(width: 7, height: 7)
            Text(label)
                .font(.caption.weight(.medium))
                .foregroundStyle(.secondary)
            Spacer()
            if case .failed = transport.state {
                Button("Retry") { BonjourTransport.shared.startBrowsing() }
                    .font(.caption.weight(.medium))
                    .buttonStyle(.borderless)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 6)
        .background(.bar)
    }

    private var dotColor: Color {
        switch transport.state {
        case .connected:    return .green
        case .connecting:   return .yellow
        case .browsing:     return .orange
        case .idle:         return .gray
        case .failed:       return .red
        }
    }

    private var label: String {
        switch transport.state {
        case .idle:                return "Not connected"
        case .browsing:            return "Looking for Mac…"
        case .connecting(let h):   return "Connecting to \(h)…"
        case .connected(let h):    return "Connected to \(h)"
        case .failed(let msg):     return "Error: \(msg)"
        }
    }
}
