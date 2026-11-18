// swift-tools-version: 6.2
// WebTiger's Apple shell: the module graph (docs/features/001-foundations.md §6).
//  - App targets contain almost nothing; everything testable lives here.
//  - The shell draws and hosts web views. It holds no browser logic: that is the Rust core's,
//    reached through WTCoreClient.
//  - Dependencies point downward only: WTUI → WTWebKit → WTCoreClient → WTContract.
import Foundation
import PackageDescription

let strict: [SwiftSetting] = [
    .enableUpcomingFeature("ExistentialAny"),
    .enableUpcomingFeature("InternalImportsByDefault"),
    .unsafeFlags(["-warnings-as-errors"], .when(configuration: .debug)),
]

/// The Rust core, which Tools/apple/xcframework.sh builds into Frameworks/. With it, WTCoreClient
/// has NativeCore (WEBTIGER_CORE) and the apps run. Without it the package still builds and its
/// tests run against ScriptedCore, so a screen can be worked on before the core is built.
let frameworkPath = Context.packageDirectory + "/Frameworks/WebTigerCore.xcframework"
let hasCore = FileManager.default.fileExists(atPath: frameworkPath)
// The platforms the framework has slices for: a Mac-only build (--mac) leaves the iOS builds
// without NativeCore rather than failing them.
let coreHasIOS = FileManager.default.fileExists(atPath: frameworkPath + "/ios-arm64")
let corePlatforms: [Platform] = coreHasIOS ? [.macOS, .iOS] : [.macOS]
let coreDependency: [Target.Dependency] = hasCore
    ? [.target(name: "WebTigerCore", condition: .when(platforms: corePlatforms))] : []
let coreSettings: [SwiftSetting] = hasCore ? [.define("WEBTIGER_CORE", .when(platforms: corePlatforms))] : []

let package = Package(
    name: "WebTiger",
    defaultLocalization: "en",
    platforms: [.macOS(.v26), .iOS(.v26)],
    products: [
        .library(name: "WTContract", targets: ["WTContract"]),
        .library(name: "WTCoreClient", targets: ["WTCoreClient"]),
        .library(name: "WTDesign", targets: ["WTDesign"]),
        .library(name: "WTWebKit", targets: ["WTWebKit"]),
        .library(name: "WTUI", targets: ["WTUI"]),
    ],
    dependencies: [
        // The contract's runtime. Tools/proto/generate.sh builds its generator from the same
        // release, read from Package.resolved.
        .package(url: "https://github.com/apple/swift-protobuf", exact: "1.41.0"),
    ],
    targets: [
        // The contract (Core/proto/webtiger/core/v1), generated and checked in.
        .target(
            name: "WTContract",
            dependencies: [.product(name: "SwiftProtobuf", package: "swift-protobuf")],
            swiftSettings: [.enableUpcomingFeature("ExistentialAny")]
        ),
        // The core as Swift sees it: requests, snapshots, host services.
        .target(
            name: "WTCoreClient",
            dependencies: ["WTContract"] + coreDependency,
            swiftSettings: strict + coreSettings
        ),
        // Design tokens, generated from Design/tokens.json.
        .target(name: "WTDesign", swiftSettings: strict),
        // The WebKit engine adapter: WKWebView behind the WebViews host service.
        .target(
            name: "WTWebKit",
            dependencies: ["WTContract", "WTCoreClient"],
            swiftSettings: strict + coreSettings
        ),
        // The windows and their views.
        .target(
            name: "WTUI",
            dependencies: ["WTContract", "WTCoreClient", "WTDesign", "WTWebKit"],
            resources: [.process("Resources")],
            swiftSettings: strict + coreSettings
        ),
        .testTarget(
            name: "WTCoreClientTests",
            dependencies: ["WTCoreClient", "WTContract"],
            swiftSettings: strict + coreSettings
        ),
        .testTarget(
            name: "WTWebKitTests",
            dependencies: ["WTWebKit", "WTCoreClient", "WTContract"],
            swiftSettings: strict + coreSettings
        ),
        .testTarget(
            name: "WTUITests",
            dependencies: ["WTUI", "WTCoreClient", "WTContract"],
            swiftSettings: strict + coreSettings
        ),
    ]
)

if hasCore {
    package.targets.append(.binaryTarget(name: "WebTigerCore", path: "Frameworks/WebTigerCore.xcframework"))
}
