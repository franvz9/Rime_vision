import Foundation
@testable import RimeVisionCore

// ── MARK: Grammar Model Tests ──

@MainActor
func testScanGramFilesFindsModels() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }

    // Create test .gram files
    let data = "test".data(using: .utf8)!
    try data.write(to: dir.appendingPathComponent("model-a.gram"))
    try data.write(to: dir.appendingPathComponent("model-b.gram"))
    try data.write(to: dir.appendingPathComponent("not-a-model.txt"))

    let manager = GrammarModelManager(rimeUserDir: dir)
    manager.scanGramFiles()

    try expectEqual(manager.models.count, 2, "should find exactly 2 .gram files")
    try expect(manager.models.contains(where: { $0.filename == "model-a" }), "should find model-a")
    try expect(manager.models.contains(where: { $0.filename == "model-b" }), "should find model-b")
}

@MainActor
func testMountWritesGrammarKeys() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }

    let manager = GrammarModelManager(rimeUserDir: dir)
    let model = GrammarModel(
        filename: "test-model",
        fileURL: dir.appendingPathComponent("test-model.gram"),
        fileSize: 1000,
        modificationDate: Date()
    )
    var config = SchemaGrammarConfig.defaultConfig(for: "test_schema")
    config.mountedModel = model.filename

    try manager.mount(model: model, to: "test_schema", config: config)

    // Read back the custom yaml
    let customURL = dir.appendingPathComponent("test_schema.custom.yaml")
    let dict = try RimeConfigStore.loadYAML(from: customURL)
    let patch = try require(dict["patch"] as? [String: Any], "patch section should exist")

    let expanded = RimePatch.expandedPatch(patch)
    let grammar = try require(expanded["grammar"] as? [String: Any], "grammar section should exist")

    try expectEqual(grammar["language"] as? String, "test-model", "grammar/language")
    try expectEqual(grammar["collocation_max_length"] as? Int, 5, "collocation_max_length")
    try expectEqual(grammar["collocation_min_length"] as? Int, 2, "collocation_min_length")

    let translator = try require(expanded["translator"] as? [String: Any], "translator section should exist")
    try expectEqual(translator["contextual_suggestions"] as? Bool, true, "contextual_suggestions")
    try expectEqual(translator["max_homophones"] as? Int, 7, "max_homophones")
    try expectEqual(translator["max_homographs"] as? Int, 7, "max_homographs")
}

@MainActor
func testUnmountRemovesOnlyGrammarKeys() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }

    // First, create a custom yaml with grammar + other translator keys
    let customURL = dir.appendingPathComponent("test_schema.custom.yaml")
    let yamlContent = """
    patch:
      grammar/language: test-model
      grammar/collocation_max_length: 5
      grammar/collocation_min_length: 2
      translator/contextual_suggestions: true
      translator/max_homophones: 7
      translator/max_homographs: 7
      translator/dictionary: test_schema
      translator/enable_user_dict: true
      "melt_eng/enable_completion": true
    """
    try yamlContent.write(to: customURL, atomically: true, encoding: .utf8)

    let manager = GrammarModelManager(rimeUserDir: dir)
    try manager.unmount(from: "test_schema")

    // Read back
    let dict = try RimeConfigStore.loadYAML(from: customURL)
    let patch = try require(dict["patch"] as? [String: Any], "patch section should exist")
    let expanded = RimePatch.expandedPatch(patch)

    // grammar should be gone
    try expect(expanded["grammar"] == nil, "grammar section should be removed")

    // translator should still have non-grammar keys
    if let translator = expanded["translator"] as? [String: Any] {
        try expect(translator["contextual_suggestions"] == nil, "contextual_suggestions should be removed")
        try expect(translator["max_homophones"] == nil, "max_homophones should be removed")
        try expect(translator["max_homographs"] == nil, "max_homographs should be removed")
        try expectEqual(translator["dictionary"] as? String, "test_schema", "dictionary should be preserved")
        try expectEqual(translator["enable_user_dict"] as? Bool, true, "enable_user_dict should be preserved")
    }

    // melt_eng should be preserved
    try expect(expanded["melt_eng"] != nil, "melt_eng should be preserved")
}

@MainActor
func testMountPreservesOtherPatchKeys() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }

    // Create existing custom yaml with unrelated keys
    let customURL = dir.appendingPathComponent("test_schema.custom.yaml")
    let yamlContent = """
    patch:
      "melt_eng/enable_completion": true
      page_size: 6
    """
    try yamlContent.write(to: customURL, atomically: true, encoding: .utf8)

    let manager = GrammarModelManager(rimeUserDir: dir)
    let model = GrammarModel(
        filename: "test-model",
        fileURL: dir.appendingPathComponent("test-model.gram"),
        fileSize: 1000,
        modificationDate: Date()
    )
    let config = SchemaGrammarConfig.defaultConfig(for: "test_schema").withModel(model)

    try manager.mount(model: model, to: "test_schema", config: config)

    // Read back
    let dict = try RimeConfigStore.loadYAML(from: customURL)
    let patch = try require(dict["patch"] as? [String: Any], "patch section should exist")
    let expanded = RimePatch.expandedPatch(patch)

    // Original keys should be preserved
    try expect(expanded["melt_eng"] != nil, "melt_eng should be preserved")
    try expectEqual(expanded["page_size"] as? Int, 6, "page_size should be preserved")

    // Grammar should be added
    let grammar = try require(expanded["grammar"] as? [String: Any], "grammar should be added")
    try expectEqual(grammar["language"] as? String, "test-model", "grammar/language")
}

@MainActor
func testLoadConfigReadsExistingCustomYAML() async throws {
    let dir = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: dir) }

    // Create a custom yaml matching real-world format
    let customURL = dir.appendingPathComponent("rime_mint.custom.yaml")
    let yamlContent = """
    patch:
      grammar/language: wanxiang-lts-zh-hans
      grammar/collocation_max_length: 8
      grammar/collocation_min_length: 2
      grammar/collocation_penalty: -16
      grammar/non_collocation_penalty: -8
      grammar/weak_collocation_penalty: -100
      grammar/rear_penalty: -20
      translator/contextual_suggestions: true
      translator/max_homophones: 7
      translator/max_homographs: 7
      "melt_eng/enable_completion": true
    """
    try yamlContent.write(to: customURL, atomically: true, encoding: .utf8)

    let manager = GrammarModelManager(rimeUserDir: dir)
    manager.loadAll(schemaIds: ["rime_mint"])

    let config = try require(manager.mountConfigs["rime_mint"], "rime_mint config should be loaded")
    try expectEqual(config.mountedModel, "wanxiang-lts-zh-hans", "mounted model")
    try expectEqual(config.collocationMaxLength, 8, "collocation_max_length")
    try expectEqual(config.collocationMinLength, 2, "collocation_min_length")
    try expectEqual(config.collocationPenalty, -16, "collocation_penalty")
    try expectEqual(config.nonCollocationPenalty, -8, "non_collocation_penalty")
    try expectEqual(config.weakCollocationPenalty, -100, "weak_collocation_penalty")
    try expectEqual(config.rearPenalty, -20, "rear_penalty")
    try expectEqual(config.contextualSuggestions, true, "contextual_suggestions")
    try expectEqual(config.maxHomophones, 7, "max_homophones")
    try expectEqual(config.maxHomographs, 7, "max_homographs")
    try expect(config.isMounted, "should be mounted")
}
