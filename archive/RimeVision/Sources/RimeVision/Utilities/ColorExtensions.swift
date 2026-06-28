import SwiftUI

extension Color {
    init(rimeColor: RimeColor) {
        self.init(
            red: Double(rimeColor.r) / 255,
            green: Double(rimeColor.g) / 255,
            blue: Double(rimeColor.b) / 255,
            opacity: Double(rimeColor.a) / 255
        )
    }
}

extension RimeColor {
    var swiftUIColor: Color {
        Color(rimeColor: self)
    }
}
