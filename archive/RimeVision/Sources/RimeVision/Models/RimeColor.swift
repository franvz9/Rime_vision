import AppKit

struct RimeColor: Equatable {
    var r: Int
    var g: Int
    var b: Int
    var a: Int

    private static let pattern8 = try! NSRegularExpression(pattern: "^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$")
    private static let pattern6 = try! NSRegularExpression(pattern: "^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$")

    init(r: Int, g: Int, b: Int, a: Int = 255) {
        self.r = min(255, max(0, r))
        self.g = min(255, max(0, g))
        self.b = min(255, max(0, b))
        self.a = min(255, max(0, a))
    }

    var nsColor: NSColor {
        NSColor(red: CGFloat(r) / 255, green: CGFloat(g) / 255, blue: CGFloat(b) / 255, alpha: CGFloat(a) / 255)
    }

    var hexString: String {
        if a < 255 {
            return String(format: "0x%02X%02X%02X%02X", a, b, g, r)
        } else {
            return String(format: "0x%02X%02X%02X", b, g, r)
        }
    }

    static func from(hex: String) -> RimeColor? {
        let cleaned = hex.replacingOccurrences(of: " ", with: "")

        if let match = pattern8.firstMatch(in: cleaned, range: NSRange(cleaned.startIndex..., in: cleaned)),
           match.numberOfRanges == 5 {
            let alpha = String(cleaned[Range(match.range(at: 1), in: cleaned)!])
            let blue = String(cleaned[Range(match.range(at: 2), in: cleaned)!])
            let green = String(cleaned[Range(match.range(at: 3), in: cleaned)!])
            let red = String(cleaned[Range(match.range(at: 4), in: cleaned)!])
            return RimeColor(
                r: Int(red, radix: 16) ?? 0,
                g: Int(green, radix: 16) ?? 0,
                b: Int(blue, radix: 16) ?? 0,
                a: Int(alpha, radix: 16) ?? 255
            )
        } else if let match = pattern6.firstMatch(in: cleaned, range: NSRange(cleaned.startIndex..., in: cleaned)),
                  match.numberOfRanges == 4 {
            let blue = String(cleaned[Range(match.range(at: 1), in: cleaned)!])
            let green = String(cleaned[Range(match.range(at: 2), in: cleaned)!])
            let red = String(cleaned[Range(match.range(at: 3), in: cleaned)!])
            return RimeColor(
                r: Int(red, radix: 16) ?? 0,
                g: Int(green, radix: 16) ?? 0,
                b: Int(blue, radix: 16) ?? 0,
                a: 255
            )
        }
        return nil
    }

    static let white = RimeColor(r: 255, g: 255, b: 255)
    static let black = RimeColor(r: 0, g: 0, b: 0)
    static let clear = RimeColor(r: 0, g: 0, b: 0, a: 0)
}
