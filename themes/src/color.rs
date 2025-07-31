//! Color definitions and manipulation for themes

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new RGBA color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create an RGB color with full alpha
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Create a color from hex string (e.g., "#FF0000" or "FF0000")
    pub fn from_hex(hex: &str) -> Result<Self, crate::ThemeError> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(crate::ThemeError::InvalidColor(format!("Invalid hex color: {}", hex)));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| crate::ThemeError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| crate::ThemeError::InvalidColor(format!("Invalid hex color: {}", hex)))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| crate::ThemeError::InvalidColor(format!("Invalid hex color: {}", hex)))?;

        Ok(Self::rgb(r, g, b))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert to HSL
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;

        let lightness = (max + min) / 2.0;

        let (hue, saturation) = if delta == 0.0 {
            (0.0, 0.0)
        } else {
            let saturation = if lightness < 0.5 {
                delta / (max + min)
            } else {
                delta / (2.0 - max - min)
            };

            let hue = if max == r {
                ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
            } else if max == g {
                ((b - r) / delta + 2.0) / 6.0
            } else {
                ((r - g) / delta + 4.0) / 6.0
            };

            (hue * 360.0, saturation)
        };

        (hue, saturation, lightness)
    }

    /// Check if color is considered "dark"
    pub fn is_dark(&self) -> bool {
        let luminance = 0.299 * (self.r as f64) + 0.587 * (self.g as f64) + 0.114 * (self.b as f64);
        luminance < 128.0
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for Color {
    type Err = crate::ThemeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Color::from_hex(s)
    }
}

/// Terminal color palette with normal and bright variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    pub normal: ColorPalette,
    pub bright: ColorPalette,
}

impl TerminalColors {
    /// Provides a default dark terminal color scheme
    pub fn default_dark() -> Self {
        Self {
            normal: ColorPalette {
                black: Color::rgb(0, 0, 0),
                red: Color::rgb(205, 49, 49),
                green: Color::rgb(13, 188, 121),
                yellow: Color::rgb(229, 229, 16),
                blue: Color::rgb(36, 114, 200),
                magenta: Color::rgb(188, 63, 188),
                cyan: Color::rgb(17, 168, 205),
                white: Color::rgb(229, 229, 229),
            },
            bright: ColorPalette {
                black: Color::rgb(102, 102, 102),
                red: Color::rgb(240, 113, 113),
                green: Color::rgb(120, 240, 120),
                yellow: Color::rgb(240, 240, 120),
                blue: Color::rgb(120, 120, 240),
                magenta: Color::rgb(240, 120, 240),
                cyan: Color::rgb(120, 240, 240),
                white: Color::rgb(255, 255, 255),
            },
        }
    }

    /// Provides a default light terminal color scheme
    pub fn default_light() -> Self {
        Self {
            normal: ColorPalette {
                black: Color::rgb(0, 0, 0),
                red: Color::rgb(205, 49, 49),
                green: Color::rgb(13, 188, 121),
                yellow: Color::rgb(180, 180, 0),
                blue: Color::rgb(36, 114, 200),
                magenta: Color::rgb(188, 63, 188),
                cyan: Color::rgb(17, 168, 205),
                white: Color::rgb(200, 200, 200),
            },
            bright: ColorPalette {
                black: Color::rgb(128, 128, 128),
                red: Color::rgb(255, 0, 0),
                green: Color::rgb(0, 255, 0),
                yellow: Color::rgb(255, 255, 0),
                blue: Color::rgb(0, 0, 255),
                magenta: Color::rgb(255, 0, 255),
                cyan: Color::rgb(0, 255, 255),
                white: Color::rgb(255, 255, 255),
            },
        }
    }
}

/// Standard 8-color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
}

impl ColorPalette {
    /// Get color by ANSI color index (0-7)
    pub fn get_by_index(&self, index: u8) -> Option<Color> {
        match index {
            0 => Some(self.black),
            1 => Some(self.red),
            2 => Some(self.green),
            3 => Some(self.yellow),
            4 => Some(self.blue),
            5 => Some(self.magenta),
            6 => Some(self.cyan),
            7 => Some(self.white),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::rgb(255, 0, 0);
        assert_eq!(color.to_hex(), "#ff0000");
    }

    #[test]
    fn test_color_is_dark() {
        let dark = Color::rgb(50, 50, 50);
        let light = Color::rgb(200, 200, 200);
        
        assert!(dark.is_dark());
        assert!(!light.is_dark());
    }
}
