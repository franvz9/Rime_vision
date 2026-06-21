import Foundation
import RimeVisionCore

func testWriteIfChangedDoesNotRewriteIdenticalContent() throws {
    let directory = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: directory) }

    let file = directory.appendingPathComponent("default.custom.yaml")
    try "patch:\n  menu:\n    page_size: 6\n".write(to: file, atomically: true, encoding: .utf8)
    let before = try FileManager.default.attributesOfItem(atPath: file.path)[.modificationDate] as? Date

    let result = try ConfigBackup.writeIfChanged("patch:\n  menu:\n    page_size: 6\n", to: file)
    let after = try FileManager.default.attributesOfItem(atPath: file.path)[.modificationDate] as? Date

    try expectEqual(result, .unchanged, "identical content should not be rewritten")
    try expectEqual(before, after, "modification date should stay unchanged")
    let files = try FileManager.default.contentsOfDirectory(atPath: directory.path)
    try expectEqual(files, ["default.custom.yaml"], "no backup should be created for unchanged content")
}

func testWriteIfChangedCreatesTimestampedBackupBeforeOverwriting() throws {
    let directory = try temporaryDirectory()
    defer { try? FileManager.default.removeItem(at: directory) }

    let file = directory.appendingPathComponent("squirrel.custom.yaml")
    try "patch:\n  style:\n    color_scheme: old\n".write(to: file, atomically: true, encoding: .utf8)

    let result = try ConfigBackup.writeIfChanged("patch:\n  style:\n    color_scheme: new\n", to: file)

    try expectEqual(result, .written, "different content should be written")
    try expectEqual(try String(contentsOf: file, encoding: .utf8), "patch:\n  style:\n    color_scheme: new\n", "new content should be written")

    let files = try FileManager.default.contentsOfDirectory(atPath: directory.path)
    let backups = files.filter { $0.hasPrefix("squirrel.custom.yaml.") && $0.hasSuffix(".bak") }
    try expectEqual(backups.count, 1, "one timestamped backup should be created")
    let backupContent = try String(contentsOf: directory.appendingPathComponent(backups[0]), encoding: .utf8)
    try expectEqual(backupContent, "patch:\n  style:\n    color_scheme: old\n", "backup should contain old content")
}
