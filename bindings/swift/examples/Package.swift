// swift-tools-version: 6.0
// The examples of eranos in Swift: each target ports the Rust example of its name and prints the same report,
// which `cargo xtask gates` compares byte for byte on a build of the library whose identifiers count up.

import PackageDescription

let package = Package(
    name: "EranosExamples",
    platforms: [.macOS(.v14)],
    dependencies: [.package(path: "..")],
    targets: [
        .target(name: "Report", dependencies: [.product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "grand_tour", dependencies: ["Report", .product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "members_and_accounts", dependencies: ["Report", .product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "multi_currency", dependencies: ["Report", .product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "persistence", dependencies: ["Report", .product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "quickstart", dependencies: ["Report", .product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "restaurant_bill", dependencies: ["Report", .product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "settling_up", dependencies: ["Report", .product(name: "Eranos", package: "swift")]),
        .executableTarget(name: "split_methods", dependencies: ["Report", .product(name: "Eranos", package: "swift")])
    ]
)
