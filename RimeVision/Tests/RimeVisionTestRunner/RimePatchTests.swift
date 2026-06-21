import Foundation
import RimeVisionCore

func testSlashPathPatchMergesIntoNestedDictionary() throws {
    let base: [String: Any] = [
        "menu": ["page_size": 6],
        "switcher": ["caption": "〔方案切换〕"]
    ]
    let patch: [String: Any] = [
        "menu/page_size": 9,
        "switcher": ["hotkeys": ["Control+grave", "Control+Shift+grave"]]
    ]

    let merged = RimePatch.merge(base: base, patch: patch)

    let menu = try require(merged["menu"] as? [String: Any], "menu should be a dictionary")
    try expectEqual(menu["page_size"] as? Int, 9, "page_size should come from slash-path patch")

    let switcher = try require(merged["switcher"] as? [String: Any], "switcher should be a dictionary")
    try expectEqual(switcher["caption"] as? String, "〔方案切换〕", "nested merge should preserve existing keys")
    try expectEqual(switcher["hotkeys"] as? [String], ["Control+grave", "Control+Shift+grave"], "flow sequence should stay an array")
}

func testExpandedPatchTreatsNestedAndSlashPathAsEquivalent() throws {
    let patch: [String: Any] = [
        "menu/page_size": 8,
        "ascii_composer": [
            "switch_key/Shift_L": "commit_code"
        ]
    ]

    let expanded = RimePatch.expandedPatch(patch)

    let menu = try require(expanded["menu"] as? [String: Any], "menu should be expanded")
    try expectEqual(menu["page_size"] as? Int, 8, "slash-path page size should expand")

    let composer = try require(expanded["ascii_composer"] as? [String: Any], "ascii_composer should exist")
    let switchKey = try require(composer["switch_key"] as? [String: Any], "nested slash path should expand")
    try expectEqual(switchKey["Shift_L"] as? String, "commit_code", "nested slash-path value should be set")
}

func testYAMLFlowSequenceLoadsAsArray() throws {
    let yaml = """
    patch:
      switcher:
        hotkeys: [Control+grave, Control+Shift+grave]
    """

    let parsed = try RimeConfigStore.parseYAML(yaml)
    let patch = try require(parsed["patch"] as? [String: Any], "patch should parse")
    let expanded = RimePatch.expandedPatch(patch)
    let switcher = try require(expanded["switcher"] as? [String: Any], "switcher should exist")

    try expectEqual(switcher["hotkeys"] as? [String], ["Control+grave", "Control+Shift+grave"], "hotkeys should parse as [String]")
}
