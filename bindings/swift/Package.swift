// swift-tools-version: 6.0
// The Swift package of eranos: the hand-written `Eranos` module, which apps
// import, over the code UniFFI generates from the library and the library
// itself, an XCFramework that `cargo xtask` builds.

import PackageDescription

let package = Package(
    name: "Eranos",
    platforms: [
        .iOS(.v17),
        .macCatalyst(.v17),
        .macOS(.v14),
        .tvOS(.v17),
        .visionOS(.v1),
        .watchOS(.v10)
    ],
    products: [
        .library(name: "Eranos", targets: ["Eranos"])
    ],
    dependencies: [
        // `swift package generate-documentation`, which builds the DocC archive.
        .package(url: "https://github.com/swiftlang/swift-docc-plugin", from: "1.5.0")
    ],
    targets: [
        .binaryTarget(name: "EranosFFI", path: "EranosFFI.xcframework"),
        .target(name: "EranosGenerated", dependencies: ["EranosFFI"]),
        .target(name: "Eranos", dependencies: ["EranosGenerated"]),
        // The tests build values of the generated code, to convert every one the library can
        // send and to reach the fallbacks for values it never sends.
        .testTarget(name: "EranosTests", dependencies: ["Eranos", "EranosGenerated"])
    ]
)
