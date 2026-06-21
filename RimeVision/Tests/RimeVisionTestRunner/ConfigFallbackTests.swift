import Foundation
import RimeVisionCore

func testInvalidCustomYAMLFallsBackToBaseConfig() throws {
    let directory = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: directory) }

    let base = directory.appendingPathComponent("default.yaml")
    let custom = directory.appendingPathComponent("default.custom.yaml")

    try """
    schema_list:
      - schema: rime_mint
    menu:
      page_size: 6
    """.write(to: base, atomically: true, encoding: .utf8)

    try """
    patch:
      punctuator:
        full_shape:
          ': { commit: "' : { commit: ： }" }
    """.write(to: custom, atomically: true, encoding: .utf8)

    let effective = try RimeConfigStore.loadEffective(baseURL: base, customURL: custom)
    let menu = try require(effective["menu"] as? [String: Any], "base menu should still load when custom is invalid")
    try expectEqual(menu["page_size"] as? Int, 6, "page_size should fall back to base")

    let schemas = try require(effective["schema_list"] as? [[String: Any]], "base schema list should still load when custom is invalid")
    try expectEqual(schemas.first?["schema"] as? String, "rime_mint", "schema list should fall back to base")
}
