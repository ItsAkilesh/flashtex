// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FlashTeXMac",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "FlashTeXMac", targets: ["FlashTeXMac"]),
        .library(name: "FlashTeXProtocol", targets: ["FlashTeXProtocol"]),
    ],
    targets: [
        // Codable models for docs/contracts/runtime-v1.md plus offset conversion.
        .target(name: "FlashTeXProtocol"),
        .executableTarget(
            name: "FlashTeXMac",
            dependencies: ["FlashTeXProtocol"]
        ),
        .testTarget(
            name: "FlashTeXProtocolTests",
            dependencies: ["FlashTeXProtocol"]
        ),
        .testTarget(
            name: "FlashTeXMacTests",
            dependencies: ["FlashTeXMac"]
        ),
    ]
)
