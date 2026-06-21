import Foundation
import RimeVisionCore

@MainActor
func testSaveKeyBindingsWritesBindingsWithoutTouchingOtherDefaultPatchSections() throws {
    let directory = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: directory) }

    try """
    patch:
      switcher:
        caption: original
    """.write(to: directory.appendingPathComponent("default.custom.yaml"), atomically: true, encoding: .utf8)

    let manager = ConfigManager(rimeUserDir: directory)
    let bindings = [
        ConfigManager.KeyBindingItem(when: "paging", accept: "Control+p", send: "Page_Up", toggle: "", select: "")
    ]

    try manager.saveKeyBindings(bindings)

    let saved = try RimeConfigStore.parseYAML(String(contentsOf: directory.appendingPathComponent("default.custom.yaml"), encoding: .utf8))
    let patch = try require(saved["patch"] as? [String: Any], "patch should exist")
    let switcher = try require(patch["switcher"] as? [String: Any], "unrelated switcher patch should be preserved")
    try expectEqual(switcher["caption"] as? String, "original", "switcher caption should be preserved")

    let keyBinder = try require(patch["key_binder"] as? [String: Any], "key_binder should be saved")
    let savedBindings = try require(keyBinder["bindings"] as? [[String: Any]], "bindings should be an array of dictionaries")
    try expectEqual(savedBindings.count, 1, "one binding should be saved")
    try expectEqual(savedBindings[0]["when"] as? String, "paging", "when should be saved")
    try expectEqual(savedBindings[0]["accept"] as? String, "Control+p", "accept should be saved")
    try expectEqual(savedBindings[0]["send"] as? String, "Page_Up", "send should be saved")
}

@MainActor
func testLoadDefaultConfigUsesCustomPatchForPageSizeKeyBindingsAndPunctuation() throws {
    let directory = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: directory) }

    try """
    menu:
      page_size: 6
    key_binder:
      bindings:
        - { when: paging, accept: Control+n, send: Page_Down }
    punctuator:
      half_shape:
        ",": { commit: ， }
    """.write(to: directory.appendingPathComponent("default.yaml"), atomically: true, encoding: .utf8)

    try """
    patch:
      "menu/page_size": 9
      key_binder:
        bindings:
          - { when: paging, accept: Control+p, send: Page_Up }
      punctuator:
        half_shape:
          ".": { commit: 。 }
    """.write(to: directory.appendingPathComponent("default.custom.yaml"), atomically: true, encoding: .utf8)

    let manager = ConfigManager(rimeUserDir: directory)
    manager.loadDefaultConfig()
    manager.loadKeyBindings()
    manager.loadPunctuation()

    try expectEqual(manager.pageSize, 9, "page size should come from slash-path custom patch")
    try expectEqual(manager.keyBindings.map(\.accept), ["Control+p"], "key bindings should come from custom patch")
    try expect(manager.halfShapePunct.contains { $0.key == "." && $0.commit == "。" }, "punctuation should come from custom patch")
}
