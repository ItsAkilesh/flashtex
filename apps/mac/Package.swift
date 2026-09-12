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
            dependencies: ["FlashTeXMac"]
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
