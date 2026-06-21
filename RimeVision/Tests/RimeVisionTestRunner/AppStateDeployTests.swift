import Foundation
import RimeVisionCore

@MainActor
func testDeployWithoutUnsavedChangesDoesNotRewriteCustomFiles() throws {
    let directory = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: directory) }

    let squirrelCustom = directory.appendingPathComponent("squirrel.custom.yaml")
    let defaultCustom = directory.appendingPathComponent("default.custom.yaml")
    let squirrelContent = """
    patch:
      style:
        color_scheme: blue_reverie
    """
    let defaultContent = """
    patch:
      menu:
        page_size: 9
    """
    try squirrelContent.write(to: squirrelCustom, atomically: true, encoding: .utf8)
    try defaultContent.write(to: defaultCustom, atomically: true, encoding: .utf8)

    let manager = ConfigManager(rimeUserDir: directory)
    let appState = AppState(configManager: manager)
    appState.hasUnsavedChanges = false

    appState.deploy()

    try expectEqual(try String(contentsOf: squirrelCustom, encoding: .utf8), squirrelContent, "deploy without changes should not rewrite squirrel.custom.yaml")
    try expectEqual(try String(contentsOf: defaultCustom, encoding: .utf8), defaultContent, "deploy without changes should not rewrite default.custom.yaml")
}
