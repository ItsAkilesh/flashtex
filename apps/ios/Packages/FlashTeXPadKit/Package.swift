// swift-tools-version: 5.9
import PackageDescription

// FlashTeXPadKit: the iPad app's non-UI logic. It links the EXISTING contracts
// without a second protocol:
//   - `FlashTeXProtocol`  <- symlink to apps/mac/Sources/FlashTeXProtocol
//                            (runtime-v1 / transfer-v1 Codable models, UTF-8
//                            byte-offset discipline)
//   - `NearbyClient`      <- symlink to apps/mac/tools/nearby-client/Sources/NearbyClient
//                            (the reference companion client: TLS-PSK pairing,
//                            hello, destination_query, capture_submit)
// Nothing under the two symlinks is owned or edited by apps/ios.
let package = Package(
    name: "FlashTeXPadKit",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "FlashTeXPadKit", targets: ["FlashTeXPadKit"]),
        .library(name: "NearbyClient", targets: ["NearbyClient"]),
        .library(name: "FlashTeXProtocol", targets: ["FlashTeXProtocol"]),
    ],
    targets: [
        .target(name: "FlashTeXProtocol"),
        .target(name: "NearbyClient"),
        .target(name: "FlashTeXPadKit", dependencies: ["FlashTeXProtocol", "NearbyClient"]),
        .testTarget(name: "FlashTeXPadKitTests", dependencies: ["FlashTeXPadKit"]),
    ]
)
