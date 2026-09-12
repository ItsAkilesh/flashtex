// swift-tools-version: 5.9
import PackageDescription

// NOTE: This Package.swift is provided for reference/testing of individual
// source file compilation. The canonical build is through the Xcode project
// (FlashTeXCompanion.xcodeproj) which targets iOS/iPadOS with PencilKit.
// `swift build` on this package will fail because it uses UIKit/PencilKit
// which require the iOS SDK. Use xcodebuild instead:
//
//   xcodebuild -project FlashTeXCompanion.xcodeproj \
//     -target FlashTeXCompanion \
//     -sdk iphonesimulator CODE_SIGNING_ALLOWED=NO build

let package = Package(
    name: "FlashTeXCompanion",
    platforms: [.iOS(.v17)],
    targets: [
        .executableTarget(
            name: "FlashTeXCompanion",
            path: "FlashTeXCompanion"
        )
    ]
)
