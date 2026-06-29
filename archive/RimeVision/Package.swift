// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "RimeVision",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "RimeVision", targets: ["RimeVision"]),
        .executable(name: "RimeVisionTestRunner", targets: ["RimeVisionTestRunner"])
    ],
    dependencies: [
        .package(url: "https://github.com/jpsim/Yams.git", from: "5.1.3")
    ],
    targets: [
        .target(
            name: "RimeVisionCore",
            dependencies: ["Yams"],
            path: "Sources/RimeVision",
            exclude: ["RimeVisionApp.swift"]
        ),
        .executableTarget(
            name: "RimeVision",
            dependencies: ["RimeVisionCore"],
            path: "Sources/RimeVisionApp"
        ),
        .executableTarget(
            name: "RimeVisionTestRunner",
            dependencies: ["RimeVisionCore"],
            path: "Tests/RimeVisionTestRunner"
        )
    ]
)
