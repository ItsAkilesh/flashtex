import SwiftUI
import VisionKit

/// Scans the Mac's pairing QR (`flashtex-nearby://pair?…`) with VisionKit's
/// `DataScannerViewController` where the device supports it (camera + Neural
/// Engine; never in the simulator, where `isSupported` is false). The first
/// recognised barcode whose payload has our scheme is handed back; anything
/// else is ignored. The caller falls back to pasting the URL text.
struct PairingScannerView: UIViewControllerRepresentable {
    var onPayload: (String) -> Void

    static var isAvailable: Bool { DataScannerViewController.isSupported && DataScannerViewController.isAvailable }

    func makeUIViewController(context: Context) -> DataScannerViewController {
        let vc = DataScannerViewController(recognizedDataTypes: [.barcode(symbologies: [.qr])], qualityLevel: .balanced,
                                           recognizesMultipleItems: false, isHighFrameRateTrackingEnabled: false,
                                           isHighlightingEnabled: true)
        vc.delegate = context.coordinator
        try? vc.startScanning()
        return vc
    }

    func updateUIViewController(_ vc: DataScannerViewController, context: Context) {}

    static func dismantleUIViewController(_ vc: DataScannerViewController, coordinator: Coordinator) { vc.stopScanning() }

    func makeCoordinator() -> Coordinator { Coordinator(onPayload: onPayload) }

    final class Coordinator: NSObject, DataScannerViewControllerDelegate {
        let onPayload: (String) -> Void
        private var delivered = false
        init(onPayload: @escaping (String) -> Void) { self.onPayload = onPayload }

        func dataScanner(_ scanner: DataScannerViewController, didAdd addedItems: [RecognizedItem], allItems: [RecognizedItem]) {
            guard !delivered else { return }
            for item in addedItems {
                if case .barcode(let b) = item, let text = b.payloadStringValue, text.hasPrefix("flashtex-nearby://") {
                    delivered = true
                    scanner.stopScanning()
                    onPayload(text)
                    return
                }
            }
        }
    }
}
