// swift-tools-version: 5.9
import PackageDescription

// Reference companion client for the FlashTeX nearby transport
// (apps/mac/docs/nearby-v1-proposal.md). No dependency on the Mac app or on
// FlashTeXProtocol so the companion (FT-004) can copy `Sources/NearbyClient`
// verbatim and apps/mac can use it as a test-only path dependency without a
// package cycle.
let package = Package(
    name: "nearby-client",
    platforms: [.macOS(.v13), .iOS(.v15)],
    products: [
        .library(name: "NearbyClient", targets: ["NearbyClient"]),
        .executable(name: "nearby-client", targets: ["nearby-client"]),
    ],
    targets: [
        // Discovery, pairing-code derivation, TLS-PSK parameters, the
        // JSON Lines session, a small pair store, and the CLI command logic.
        .target(name: "NearbyClient"),
        // Thin executable: `nearby-client pair|send|status|browse|forget`.
        .executableTarget(name: "nearby-client", dependencies: ["NearbyClient"]),
        .testTarget(name: "NearbyClientTests", dependencies: ["NearbyClient"]),
    ]
)
