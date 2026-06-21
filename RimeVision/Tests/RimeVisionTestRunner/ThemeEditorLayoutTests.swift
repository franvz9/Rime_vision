import Foundation
import RimeVisionCore

func testThemeEditorSheetUsesRoomyLayoutMetrics() throws {
    try expect(SchemeDetailLayout.sheetMinWidth >= 680, "theme editor sheet should be wide enough for labels and color editor")
    try expect(SchemeDetailLayout.sheetMinHeight >= 720, "theme editor sheet should be tall enough for color rows")
    try expect(SchemeDetailLayout.labelWidth >= 90, "theme editor labels should have stable width")
}
