import Foundation
import RimeVisionCore

func testSchemeCopyNameUsesCopySuffixAndSkipsExistingNames() throws {
    let result = SchemeCopyNaming.uniqueCopyName(for: "blue_reverie", existingNames: [
        "blue_reverie",
        "blue_reverie_copy",
        "blue_reverie_copy_2"
    ])

    try expectEqual(result, "blue_reverie_copy_3", "copy name should use next available suffix")
}

func testSchemeCopyNameUsesPlainCopyWhenAvailable() throws {
    let result = SchemeCopyNaming.uniqueCopyName(for: "blue_reverie", existingNames: ["blue_reverie"])

    try expectEqual(result, "blue_reverie_copy", "copy name should use _copy when it is available")
}
