use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static HEX8_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$").unwrap()
});

static HEX6_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^0x([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})([A-Fa-f0-9]{2})$").unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}
