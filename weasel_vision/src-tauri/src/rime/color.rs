use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use ts_rs::TS;

static HEX8_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$").unwrap()
});

static HEX6_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct RimeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RimeColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let cleaned = hex.replace(' ', "");

        if let Some(caps) = HEX8_PATTERN.captures(&cleaned) {
            let alpha = u8::from_str_radix(&caps[1], 16).ok()?;
            let blue = u8::from_str_radix(&caps[2], 16).ok()?;
            let green = u8::from_str_radix(&caps[3], 16).ok()?;
            let red = u8::from_str_radix(&caps[4], 16).ok()?;
            return Some(Self::new(red, green, blue, alpha));
        }

        if let Some(caps) = HEX6_PATTERN.captures(&cleaned) {
            let blue = u8::from_str_radix(&caps[1], 16).ok()?;
            let green = u8::from_str_radix(&caps[2], 16).ok()?;
            let red = u8::from_str_radix(&caps[3], 16).ok()?;
            return Some(Self::new(red, green, blue, 255));
        }

        None
    }

    pub fn to_hex(self) -> String {
        if self.a < 255 {
            format!(
                "0x{:02X}{:02X}{:02X}{:02X}",
                self.a, self.b, self.g, self.r
            )
        } else {
            format!("0x{:02X}{:02X}{:02X}", self.b, self.g, self.r)
        }
    }

    /// Convert RGBA color to CSS rgba() string format
    /// TODO(v0.3.0): Use this for CSS color output in theme export
    #[allow(dead_code)]
    pub fn to_css(self) -> String {
        if self.a < 255 {
            format!(
                "rgba({}, {}, {}, {:.2})",
                self.r,
                self.g,
                self.b,
                self.a as f64 / 255.0
            )
        } else {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        }
    }

    /// Pure white color (255, 255, 255, 255)
    /// TODO(v0.3.0): Use for default initializers and fallback values
    #[allow(dead_code)]
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Pure black color (0, 0, 0, 255)
    /// TODO(v0.3.0): Use for default initializers and fallback values
    #[allow(dead_code)]
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Fully transparent color (0, 0, 0, 0)
    /// TODO(v0.3.0): Use for default initializers and fallback values
    #[allow(dead_code)]
    pub const CLEAR: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_6() {
        let c = RimeColor::from_hex("0xFF0000").unwrap();
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_parse_hex_8() {
        let c = RimeColor::from_hex("0x80FF0000").unwrap();
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_to_hex_no_alpha() {
        let c = RimeColor::new(255, 128, 0, 255);
        assert_eq!(c.to_hex(), "0x0080FF");
    }

    #[test]
    fn test_to_hex_with_alpha() {
        let c = RimeColor::new(255, 128, 0, 128);
        assert_eq!(c.to_hex(), "0x800080FF");
    }

    #[test]
    fn test_parse_invalid_hex() {
        assert!(RimeColor::from_hex("0xGGGGGG").is_none());
        assert!(RimeColor::from_hex("not_a_color").is_none());
        assert!(RimeColor::from_hex("").is_none());
        assert!(RimeColor::from_hex("0x12345").is_none()); // 5 digits: too short for 6-digit
        assert!(RimeColor::from_hex("0x1234567").is_none()); // 7 digits: invalid
    }

    #[test]
    fn test_parse_hex_case_insensitive() {
        let lower = RimeColor::from_hex("0xff0000").unwrap();
        let upper = RimeColor::from_hex("0xFF0000").unwrap();
        assert_eq!(lower, upper);

        let lower8 = RimeColor::from_hex("0x80ff0000").unwrap();
        let upper8 = RimeColor::from_hex("0x80FF0000").unwrap();
        assert_eq!(lower8, upper8);
    }

    #[test]
    fn test_black_white_transparent() {
        // Black: 0x000000 → B=0, G=0, R=0
        let black = RimeColor::from_hex("0x000000").unwrap();
        assert_eq!(black, RimeColor::BLACK);

        // White: 0xFFFFFF → B=255, G=255, R=255
        let white = RimeColor::from_hex("0xFFFFFF").unwrap();
        assert_eq!(white, RimeColor::WHITE);

        // Fully transparent: 0x00000000 → A=0, B=0, G=0, R=0
        let transparent = RimeColor::from_hex("0x00000000").unwrap();
        assert_eq!(transparent, RimeColor::CLEAR);
    }

    #[test]
    fn test_roundtrip_hex_6() {
        let c = RimeColor::new(0x12, 0x34, 0x56, 255);
        let hex = c.to_hex();
        let parsed = RimeColor::from_hex(&hex).unwrap();
        assert_eq!(c, parsed);
    }

    #[test]
    fn test_roundtrip_hex_8() {
        let c = RimeColor::new(0x12, 0x34, 0x56, 0x78);
        let hex = c.to_hex();
        let parsed = RimeColor::from_hex(&hex).unwrap();
        assert_eq!(c, parsed);
    }

    #[test]
    fn test_to_css_opaque() {
        let c = RimeColor::new(255, 128, 0, 255);
        assert_eq!(c.to_css(), "rgb(255, 128, 0)");
    }

    #[test]
    fn test_to_css_translucent() {
        let c = RimeColor::new(255, 128, 0, 128);
        assert_eq!(c.to_css(), "rgba(255, 128, 0, 0.50)");
    }

    #[test]
    fn test_parse_hex_with_spaces() {
        // from_hex strips spaces
        let c = RimeColor::from_hex("0xFF 00 00").unwrap();
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 255);
    }
}
