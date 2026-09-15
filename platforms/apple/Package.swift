// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "LiveForgeApple",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "LiveForgeApple", targets: ["LiveForgeApple"]),
    ],
    targets: [
        .target(name: "LiveForgeApple"),
        .testTarget(name: "LiveForgeAppleTests", dependencies: ["LiveForgeApple"]),
    ],
    swiftLanguageVersions: [.v5]
)
