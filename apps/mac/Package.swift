// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FlashTeXMac",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "FlashTeXMac", targets: ["FlashTeXMac"]),
        .library(name: "FlashTeXProtocol", targets: ["FlashTeXProtocol"]),
        .library(name: "FlashTeXAccessibility", targets: ["FlashTeXAccessibility"]),
    ],
    dependencies: [
        // Test-only: the reference companion client (apps/mac/tools/nearby-client)
        // drives the real NearbyListener in NearbyReferenceClientTests. The
        // package has no dependency back on this one, so there is no cycle.
        .package(path: "tools/nearby-client"),
    ],
    targets: [
        // Codable models for docs/contracts/runtime-v1.md plus offset conversion.
        .target(name: "FlashTeXProtocol"),
        .executableTarget(
            name: "FlashTeXMac",
            dependencies: ["FlashTeXProtocol", "FlashTeXAccessibility"]
        ),
        .testTarget(
            name: "FlashTeXProtocolTests",
            dependencies: ["FlashTeXProtocol"]
        ),
        .testTarget(
            name: "FlashTeXMacTests",
            dependencies: ["FlashTeXMac", .product(name: "NearbyClient", package: "nearby-client")]
        ),
        // Pure accessibility models (reading sequence, editor navigation,
        // command table) plus the SwiftUI attachment views; depends only on
        // FlashTeXProtocol. Owner: mac-accessibility.
        .target(
            name: "FlashTeXAccessibility",
            dependencies: ["FlashTeXProtocol"]
        ),
        .testTarget(
            name: "FlashTeXAccessibilityTests",
            dependencies: ["FlashTeXAccessibility"]
        ),
    ]
)
