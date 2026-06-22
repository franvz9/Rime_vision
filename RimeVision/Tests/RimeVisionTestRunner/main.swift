import Foundation
import RimeVisionCore

Task {
    var runner = TestRunner()
    await runner.run("slash-path patch merges into nested dictionary", testSlashPathPatchMergesIntoNestedDictionary)
    await runner.run("nested and slash-path patches are equivalent", testExpandedPatchTreatsNestedAndSlashPathAsEquivalent)
    await runner.run("YAML flow sequence loads as string array", testYAMLFlowSequenceLoadsAsArray)
    await runner.run("writeIfChanged skips identical content", testWriteIfChangedDoesNotRewriteIdenticalContent)
    await runner.run("writeIfChanged creates timestamped backup", testWriteIfChangedCreatesTimestampedBackupBeforeOverwriting)
    await runner.run("key bindings save without touching unrelated sections", testSaveKeyBindingsWritesBindingsWithoutTouchingOtherDefaultPatchSections)
    await runner.run("default config loads custom patch values", testLoadDefaultConfigUsesCustomPatchForPageSizeKeyBindingsAndPunctuation)
    await runner.run("deploy without unsaved changes does not rewrite custom files", testDeployWithoutUnsavedChangesDoesNotRewriteCustomFiles)
    await runner.run("invalid custom YAML falls back to base config", testInvalidCustomYAMLFallsBackToBaseConfig)
    await runner.run("theme editor sheet uses roomy layout metrics", testThemeEditorSheetUsesRoomyLayoutMetrics)
    await runner.run("scheme copy name uses next available suffix", testSchemeCopyNameUsesCopySuffixAndSkipsExistingNames)
    await runner.run("scheme copy name uses plain copy when available", testSchemeCopyNameUsesPlainCopyWhenAvailable)
    await runner.run("scan gram files finds models", testScanGramFilesFindsModels)
    await runner.run("mount writes grammar keys to custom yaml", testMountWritesGrammarKeys)
    await runner.run("unmount removes only grammar keys", testUnmountRemovesOnlyGrammarKeys)
    await runner.run("mount preserves other patch keys", testMountPreservesOtherPatchKeys)
    await runner.run("load config reads existing custom yaml", testLoadConfigReadsExistingCustomYAML)
    runner.finish()
}

RunLoop.main.run()

struct TestRunner {
    private var failures: [String] = []

    mutating func run(_ name: String, _ test: () async throws -> Void) async {
        do {
            try await test()
            print("✓ \(name)")
        } catch {
            failures.append("✗ \(name): \(error)")
            print("✗ \(name): \(error)")
        }
    }

    func finish() -> Never {
        if failures.isEmpty {
            print("All 17 tests passed")
            exit(0)
        }
        print("\nFailures:")
        failures.forEach { print($0) }
        exit(1)
    }
}

enum TestFailure: Error, CustomStringConvertible {
    case message(String)

    var description: String {
        switch self {
        case .message(let message): return message
        }
    }
}

func expect(_ condition: @autoclosure () -> Bool, _ message: String) throws {
    if !condition() { throw TestFailure.message(message) }
}

func expectEqual<T: Equatable>(_ actual: T, _ expected: T, _ message: String) throws {
    if actual != expected {
        throw TestFailure.message("\(message). Expected \(expected), got \(String(describing: actual))")
    }
}

func require<T>(_ value: T?, _ message: String) throws -> T {
    guard let value else { throw TestFailure.message(message) }
    return value
}

func temporaryDirectory() throws -> URL {
    let directory = FileManager.default.temporaryDirectory
        .appendingPathComponent("RimeVisionTests-")
        .appendingPathComponent(UUID().uuidString)
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    return directory
}
