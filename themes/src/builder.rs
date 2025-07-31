//! Theme builder and generator
//!
//! This module provides tools for creating themes programmatically with:
//! - Automatic color palette generation
//! - Harmony-based theme creation
//! - Template-based theme building
//! - Color manipulation utilities

use crate::{Theme, Color, TerminalColors, ColorPalette, ThemeDetails, Result, ThemeError};
use std::collections::HashMap;

/// Theme generation strategies
#[derive(Debug, Clone)]
pub enum GenerationStrategy {
    /// Generate from a single base color
    MonochromaticFromColor(Color),
    /// Generate complementary color scheme
    ComplementaryFromColor(Color),
    /// Generate analogous color scheme
    AnalogousFromColor(Color),
    /// Generate triadic color scheme
    TriadicFromColor(Color),
    /// Import from popular color palette
    FromPalette(String),
}

/// Theme builder with fluent API
#[derive(Debug, Clone)]
pub struct ThemeBuilder {
    name: Option<String>,
    accent: Option<Color>,
    background: Option<Color>,
    foreground: Option<Color>,
    cursor: Option<Color>,
    details: Option<ThemeDetails>,
    terminal_colors: Option<TerminalColors>,
    metadata: HashMap<String, serde_yaml::Value>,
}

impl Default for ThemeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeBuilder {
    /// Create a new theme builder
    pub fn new() -> Self {
        Self {
            name: None,
            accent: None,
            background: None,
            foreground: None,
            cursor: None,
            details: None,
            terminal_colors: None,
            metadata: HashMap::new(),
        }
    }

    /// Generate a theme using a specific strategy
    pub fn from_strategy(strategy: GenerationStrategy) -> Result<Self> {
        let mut builder = Self::new();
        
        match strategy {
            GenerationStrategy::MonochromaticFromColor(base_color) => {
                builder = builder.monochromatic_from_color(base_color)?;
            }
            GenerationStrategy::ComplementaryFromColor(base_color) => {
                builder = builder.complementary_from_color(base_color)?;
            }
            GenerationStrategy::AnalogousFromColor(base_color) => {
                builder = builder.analogous_from_color(base_color)?;
            }
            GenerationStrategy::TriadicFromColor(base_color) => {
                builder = builder.triadic_from_color(base_color)?;
            }
            GenerationStrategy::FromPalette(palette_name) => {
                builder = builder.from_popular_palette(&palette_name)?;
            }
        }
        
        Ok(builder)
    }

    /// Set theme name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set accent color
    pub fn accent(mut self, color: Color) -> Self {
        self.accent = Some(color);
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set foreground color
    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    /// Set cursor color
    pub fn cursor(mut self, color: Color) -> Self {
        self.cursor = Some(color);
        self
    }

    /// Set theme details
    pub fn details(mut self, details: ThemeDetails) -> Self {
        self.details = Some(details);
        self
    }

    /// Set terminal colors
    pub fn terminal_colors(mut self, colors: TerminalColors) -> Self {
        self.terminal_colors = Some(colors);
        self
    }

    /// Add metadata
    pub fn metadata<K, V>(mut self, key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: Into<serde_yaml::Value>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Generate monochromatic theme from base color
    pub fn monochromatic_from_color(mut self, base_color: Color) -> Result<Self> {
        let (hue, saturation, lightness) = base_color.to_hsl();
        
        // Generate background and foreground based on lightness
        let is_dark_base = lightness < 0.5;
        
        let (background, foreground) = if is_dark_base {
            // Dark theme
            let bg = Self::hsl_to_color(hue, saturation * 0.3, 0.08);
            let fg = Self::hsl_to_color(hue, saturation * 0.1, 0.9);
            (bg, fg)
        } else {
            // Light theme
            let bg = Self::hsl_to_color(hue, saturation * 0.1, 0.95);
            let fg = Self::hsl_to_color(hue, saturation * 0.3, 0.1);
            (bg, fg)
        };

        self.accent = Some(base_color);
        self.background = Some(background);
        self.foreground = Some(foreground);
        self.details = Some(if is_dark_base { ThemeDetails::Darker } else { ThemeDetails::Lighter });

        // Generate terminal colors with variations in lightness
        let terminal_colors = self.generate_monochromatic_terminal_colors(hue, saturation, is_dark_base);
        self.terminal_colors = Some(terminal_colors);

        Ok(self)
    }

    /// Generate complementary theme from base color
    pub fn complementary_from_color(mut self, base_color: Color) -> Result<Self> {
        let (hue, saturation, lightness) = base_color.to_hsl();
        let complement_hue = (hue + 180.0) % 360.0;
        
        let is_dark_base = lightness < 0.5;
        
        let (background, foreground) = if is_dark_base {
            let bg = Self::hsl_to_color(hue, saturation * 0.2, 0.08);
            let fg = Self::hsl_to_color(complement_hue, saturation * 0.1, 0.9);
            (bg, fg)
        } else {
            let bg = Self::hsl_to_color(hue, saturation * 0.1, 0.95);
            let fg = Self::hsl_to_color(complement_hue, saturation * 0.2, 0.1);
            (bg, fg)
        };

        self.accent = Some(base_color);
        self.background = Some(background);
        self.foreground = Some(foreground);
        self.details = Some(if is_dark_base { ThemeDetails::Darker } else { ThemeDetails::Lighter });

        // Generate terminal colors using both hues
        let terminal_colors = self.generate_complementary_terminal_colors(hue, complement_hue, saturation, is_dark_base);
        self.terminal_colors = Some(terminal_colors);

        Ok(self)
    }

    /// Generate analogous theme from base color
    pub fn analogous_from_color(mut self, base_color: Color) -> Result<Self> {
        let (hue, saturation, lightness) = base_color.to_hsl();
        let is_dark_base = lightness < 0.5;
        
        // Create analogous hues (±30 degrees)
        let hue1 = (hue + 30.0) % 360.0;
        let hue2 = (hue - 30.0 + 360.0) % 360.0;
        
        let (background, foreground) = if is_dark_base {
            let bg = Self::hsl_to_color(hue2, saturation * 0.2, 0.08);
            let fg = Self::hsl_to_color(hue1, saturation * 0.1, 0.9);
            (bg, fg)
        } else {
            let bg = Self::hsl_to_color(hue2, saturation * 0.1, 0.95);
            let fg = Self::hsl_to_color(hue1, saturation * 0.2, 0.1);
            (bg, fg)
        };

        self.accent = Some(base_color);
        self.background = Some(background);
        self.foreground = Some(foreground);
        self.details = Some(if is_dark_base { ThemeDetails::Darker } else { ThemeDetails::Lighter });

        // Generate terminal colors using analogous hues
        let terminal_colors = self.generate_analogous_terminal_colors(hue, hue1, hue2, saturation, is_dark_base);
        self.terminal_colors = Some(terminal_colors);

        Ok(self)
    }

    /// Generate triadic theme from base color
    pub fn triadic_from_color(mut self, base_color: Color) -> Result<Self> {
        let (hue, saturation, lightness) = base_color.to_hsl();
        let is_dark_base = lightness < 0.5;
        
        // Create triadic hues (120° apart)
        let hue1 = (hue + 120.0) % 360.0;
        let hue2 = (hue + 240.0) % 360.0;
        
        let (background, foreground) = if is_dark_base {
            let bg = Self::hsl_to_color(hue, saturation * 0.2, 0.08);
            let fg = Self::hsl_to_color(hue1, saturation * 0.1, 0.9);
            (bg, fg)
        } else {
            let bg = Self::hsl_to_color(hue, saturation * 0.1, 0.95);
            let fg = Self::hsl_to_color(hue1, saturation * 0.2, 0.1);
            (bg, fg)
        };

        self.accent = Some(Self::hsl_to_color(hue2, saturation, lightness));
        self.background = Some(background);
        self.foreground = Some(foreground);
        self.details = Some(if is_dark_base { ThemeDetails::Darker } else { ThemeDetails::Lighter });

        // Generate terminal colors using triadic hues
        let terminal_colors = self.generate_triadic_terminal_colors(hue, hue1, hue2, saturation, is_dark_base);
        self.terminal_colors = Some(terminal_colors);

        Ok(self)
    }

    /// Generate theme from popular color palette
    pub fn from_popular_palette(self, palette_name: &str) -> Result<Self> {
        let palette = match palette_name.to_lowercase().as_str() {
            "nord" => self.nord_palette(),
            "dracula" => self.dracula_palette(),
            "solarized_dark" => self.solarized_dark_palette(),
            "solarized_light" => self.solarized_light_palette(),
            "gruvbox_dark" => self.gruvbox_dark_palette(),
            "gruvbox_light" => self.gruvbox_light_palette(),
            "tokyo_night" => self.tokyo_night_palette(),
            "catppuccin" => self.catppuccin_palette(),
            _ => return Err(ThemeError::InvalidFormat(format!("Unknown palette: {}", palette_name))),
        };

        palette
    }

    /// Build the final theme
    pub fn build(self) -> Result<Theme> {
        let accent = self.accent.ok_or_else(|| {
            ThemeError::MissingField("accent color is required".to_string())
        })?;

        let background = self.background.ok_or_else(|| {
            ThemeError::MissingField("background color is required".to_string())
        })?;

        let foreground = self.foreground.ok_or_else(|| {
            ThemeError::MissingField("foreground color is required".to_string())
        })?;

        let terminal_colors = self.terminal_colors.unwrap_or_else(|| {
            if background.is_dark() {
                TerminalColors::default_dark()
            } else {
                TerminalColors::default_light()
            }
        });

        let mut theme = Theme::new(accent, background, foreground, terminal_colors);
        
        theme.name = self.name;
        theme.cursor = self.cursor;
        theme.details = self.details;
        theme.metadata = self.metadata;

        Ok(theme)
    }

    // Helper methods for color generation

    fn hsl_to_color(hue: f64, saturation: f64, lightness: f64) -> Color {
        let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = lightness - c / 2.0;

        let (r_prime, g_prime, b_prime) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0).round() as u8;
        let g = ((g_prime + m) * 255.0).round() as u8;
        let b = ((b_prime + m) * 255.0).round() as u8;

        Color::rgb(r, g, b)
    }

    fn generate_monochromatic_terminal_colors(&self, _hue: f64, _saturation: f64, is_dark: bool) -> TerminalColors {
        let base_lightness = if is_dark { 0.4 } else { 0.6 };
        let bright_lightness = if is_dark { 0.7 } else { 0.3 };

        let normal = ColorPalette {
            black: if is_dark { Color::rgb(40, 40, 40) } else { Color::rgb(0, 0, 0) },
            red: Self::hsl_to_color(0.0, 0.8, base_lightness),
            green: Self::hsl_to_color(120.0, 0.8, base_lightness),
            yellow: Self::hsl_to_color(60.0, 0.8, base_lightness),
            blue: Self::hsl_to_color(240.0, 0.8, base_lightness),
            magenta: Self::hsl_to_color(300.0, 0.8, base_lightness),
            cyan: Self::hsl_to_color(180.0, 0.8, base_lightness),
            white: if is_dark { Color::rgb(200, 200, 200) } else { Color::rgb(255, 255, 255) },
        };

        let bright = ColorPalette {
            black: if is_dark { Color::rgb(100, 100, 100) } else { Color::rgb(128, 128, 128) },
            red: Self::hsl_to_color(0.0, 0.9, bright_lightness),
            green: Self::hsl_to_color(120.0, 0.9, bright_lightness),
            yellow: Self::hsl_to_color(60.0, 0.9, bright_lightness),
            blue: Self::hsl_to_color(240.0, 0.9, bright_lightness),
            magenta: Self::hsl_to_color(300.0, 0.9, bright_lightness),
            cyan: Self::hsl_to_color(180.0, 0.9, bright_lightness),
            white: if is_dark { Color::rgb(255, 255, 255) } else { Color::rgb(200, 200, 200) },
        };

        TerminalColors { normal, bright }
    }

    fn generate_complementary_terminal_colors(&self, hue1: f64, hue2: f64, saturation: f64, is_dark: bool) -> TerminalColors {
        let base_lightness = if is_dark { 0.5 } else { 0.5 };
        let bright_lightness = if is_dark { 0.7 } else { 0.3 };

        let normal = ColorPalette {
            black: if is_dark { Color::rgb(40, 40, 40) } else { Color::rgb(0, 0, 0) },
            red: Self::hsl_to_color(0.0, saturation, base_lightness),
            green: Self::hsl_to_color(120.0, saturation, base_lightness),
            yellow: Self::hsl_to_color(60.0, saturation, base_lightness),
            blue: Self::hsl_to_color(hue1, saturation, base_lightness),
            magenta: Self::hsl_to_color(hue2, saturation, base_lightness),
            cyan: Self::hsl_to_color(180.0, saturation, base_lightness),
            white: if is_dark { Color::rgb(200, 200, 200) } else { Color::rgb(255, 255, 255) },
        };

        let bright = ColorPalette {
            black: if is_dark { Color::rgb(100, 100, 100) } else { Color::rgb(128, 128, 128) },
            red: Self::hsl_to_color(0.0, saturation, bright_lightness),
            green: Self::hsl_to_color(120.0, saturation, bright_lightness),
            yellow: Self::hsl_to_color(60.0, saturation, bright_lightness),
            blue: Self::hsl_to_color(hue1, saturation, bright_lightness),
            magenta: Self::hsl_to_color(hue2, saturation, bright_lightness),
            cyan: Self::hsl_to_color(180.0, saturation, bright_lightness),
            white: if is_dark { Color::rgb(255, 255, 255) } else { Color::rgb(200, 200, 200) },
        };

        TerminalColors { normal, bright }
    }

    fn generate_analogous_terminal_colors(&self, hue: f64, hue1: f64, hue2: f64, saturation: f64, is_dark: bool) -> TerminalColors {
        let base_lightness = if is_dark { 0.5 } else { 0.5 };
        let bright_lightness = if is_dark { 0.7 } else { 0.3 };

        let normal = ColorPalette {
            black: if is_dark { Color::rgb(40, 40, 40) } else { Color::rgb(0, 0, 0) },
            red: Self::hsl_to_color(0.0, saturation, base_lightness),
            green: Self::hsl_to_color(hue1, saturation, base_lightness),
            yellow: Self::hsl_to_color(60.0, saturation, base_lightness),
            blue: Self::hsl_to_color(hue, saturation, base_lightness),
            magenta: Self::hsl_to_color(hue2, saturation, base_lightness),
            cyan: Self::hsl_to_color(180.0, saturation, base_lightness),
            white: if is_dark { Color::rgb(200, 200, 200) } else { Color::rgb(255, 255, 255) },
        };

        let bright = ColorPalette {
            black: if is_dark { Color::rgb(100, 100, 100) } else { Color::rgb(128, 128, 128) },
            red: Self::hsl_to_color(0.0, saturation, bright_lightness),
            green: Self::hsl_to_color(hue1, saturation, bright_lightness),
            yellow: Self::hsl_to_color(60.0, saturation, bright_lightness),
            blue: Self::hsl_to_color(hue, saturation, bright_lightness),
            magenta: Self::hsl_to_color(hue2, saturation, bright_lightness),
            cyan: Self::hsl_to_color(180.0, saturation, bright_lightness),
            white: if is_dark { Color::rgb(255, 255, 255) } else { Color::rgb(200, 200, 200) },
        };

        TerminalColors { normal, bright }
    }

    fn generate_triadic_terminal_colors(&self, hue: f64, hue1: f64, hue2: f64, saturation: f64, is_dark: bool) -> TerminalColors {
        let base_lightness = if is_dark { 0.5 } else { 0.5 };
        let bright_lightness = if is_dark { 0.7 } else { 0.3 };

        let normal = ColorPalette {
            black: if is_dark { Color::rgb(40, 40, 40) } else { Color::rgb(0, 0, 0) },
            red: Self::hsl_to_color(hue, saturation, base_lightness),
            green: Self::hsl_to_color(hue1, saturation, base_lightness),
            yellow: Self::hsl_to_color(60.0, saturation, base_lightness),
            blue: Self::hsl_to_color(hue2, saturation, base_lightness),
            magenta: Self::hsl_to_color(300.0, saturation, base_lightness),
            cyan: Self::hsl_to_color(180.0, saturation, base_lightness),
            white: if is_dark { Color::rgb(200, 200, 200) } else { Color::rgb(255, 255, 255) },
        };

        let bright = ColorPalette {
            black: if is_dark { Color::rgb(100, 100, 100) } else { Color::rgb(128, 128, 128) },
            red: Self::hsl_to_color(hue, saturation, bright_lightness),
            green: Self::hsl_to_color(hue1, saturation, bright_lightness),
            yellow: Self::hsl_to_color(60.0, saturation, bright_lightness),
            blue: Self::hsl_to_color(hue2, saturation, bright_lightness),
            magenta: Self::hsl_to_color(300.0, saturation, bright_lightness),
            cyan: Self::hsl_to_color(180.0, saturation, bright_lightness),
            white: if is_dark { Color::rgb(255, 255, 255) } else { Color::rgb(200, 200, 200) },
        };

        TerminalColors { normal, bright }
    }

    // Popular palette presets

    fn nord_palette(mut self) -> Result<Self> {
        self.name = Some("Nord".to_string());
        self.background = Some(Color::from_hex("#2E3440")?);
        self.foreground = Some(Color::from_hex("#D8DEE9")?);
        self.accent = Some(Color::from_hex("#5E81AC")?);
        self.details = Some(ThemeDetails::Darker);
        
        let normal = ColorPalette {
            black: Color::from_hex("#3B4252")?,
            red: Color::from_hex("#BF616A")?,
            green: Color::from_hex("#A3BE8C")?,
            yellow: Color::from_hex("#EBCB8B")?,
            blue: Color::from_hex("#81A1C1")?,
            magenta: Color::from_hex("#B48EAD")?,
            cyan: Color::from_hex("#88C0D0")?,
            white: Color::from_hex("#E5E9F0")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#4C566A")?,
            red: Color::from_hex("#BF616A")?,
            green: Color::from_hex("#A3BE8C")?,
            yellow: Color::from_hex("#EBCB8B")?,
            blue: Color::from_hex("#81A1C1")?,
            magenta: Color::from_hex("#B48EAD")?,
            cyan: Color::from_hex("#8FBCBB")?,
            white: Color::from_hex("#ECEFF4")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }

    fn dracula_palette(mut self) -> Result<Self> {
        self.name = Some("Dracula".to_string());
        self.background = Some(Color::from_hex("#282a36")?);
        self.foreground = Some(Color::from_hex("#f8f8f2")?);
        self.accent = Some(Color::from_hex("#ff79c6")?);
        self.details = Some(ThemeDetails::Darker);
        
        let normal = ColorPalette {
            black: Color::from_hex("#000000")?,
            red: Color::from_hex("#ff5555")?,
            green: Color::from_hex("#50fa7b")?,
            yellow: Color::from_hex("#f1fa8c")?,
            blue: Color::from_hex("#bd93f9")?,
            magenta: Color::from_hex("#ff79c6")?,
            cyan: Color::from_hex("#8be9fd")?,
            white: Color::from_hex("#bbbbbb")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#555555")?,
            red: Color::from_hex("#ff5555")?,
            green: Color::from_hex("#50fa7b")?,
            yellow: Color::from_hex("#f1fa8c")?,
            blue: Color::from_hex("#caa9fa")?,
            magenta: Color::from_hex("#ff79c6")?,
            cyan: Color::from_hex("#8be9fd")?,
            white: Color::from_hex("#ffffff")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }

    fn solarized_dark_palette(mut self) -> Result<Self> {
        self.name = Some("Solarized Dark".to_string());
        self.background = Some(Color::from_hex("#002b36")?);
        self.foreground = Some(Color::from_hex("#839496")?);
        self.accent = Some(Color::from_hex("#268bd2")?);
        self.details = Some(ThemeDetails::Darker);
        
        let normal = ColorPalette {
            black: Color::from_hex("#073642")?,
            red: Color::from_hex("#dc322f")?,
            green: Color::from_hex("#859900")?,
            yellow: Color::from_hex("#b58900")?,
            blue: Color::from_hex("#268bd2")?,
            magenta: Color::from_hex("#d33682")?,
            cyan: Color::from_hex("#2aa198")?,
            white: Color::from_hex("#eee8d5")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#002b36")?,
            red: Color::from_hex("#cb4b16")?,
            green: Color::from_hex("#586e75")?,
            yellow: Color::from_hex("#657b83")?,
            blue: Color::from_hex("#839496")?,
            magenta: Color::from_hex("#6c71c4")?,
            cyan: Color::from_hex("#93a1a1")?,
            white: Color::from_hex("#fdf6e3")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }

    fn solarized_light_palette(mut self) -> Result<Self> {
        self.name = Some("Solarized Light".to_string());
        self.background = Some(Color::from_hex("#fdf6e3")?);
        self.foreground = Some(Color::from_hex("#657b83")?);
        self.accent = Some(Color::from_hex("#268bd2")?);
        self.details = Some(ThemeDetails::Lighter);
        
        let normal = ColorPalette {
            black: Color::from_hex("#073642")?,
            red: Color::from_hex("#dc322f")?,
            green: Color::from_hex("#859900")?,
            yellow: Color::from_hex("#b58900")?,
            blue: Color::from_hex("#268bd2")?,
            magenta: Color::from_hex("#d33682")?,
            cyan: Color::from_hex("#2aa198")?,
            white: Color::from_hex("#eee8d5")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#002b36")?,
            red: Color::from_hex("#cb4b16")?,
            green: Color::from_hex("#586e75")?,
            yellow: Color::from_hex("#657b83")?,
            blue: Color::from_hex("#839496")?,
            magenta: Color::from_hex("#6c71c4")?,
            cyan: Color::from_hex("#93a1a1")?,
            white: Color::from_hex("#fdf6e3")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }

    fn gruvbox_dark_palette(mut self) -> Result<Self> {
        self.name = Some("Gruvbox Dark".to_string());
        self.background = Some(Color::from_hex("#282828")?);
        self.foreground = Some(Color::from_hex("#ebdbb2")?);
        self.accent = Some(Color::from_hex("#83a598")?);
        self.details = Some(ThemeDetails::Darker);
        
        let normal = ColorPalette {
            black: Color::from_hex("#282828")?,
            red: Color::from_hex("#cc241d")?,
            green: Color::from_hex("#98971a")?,
            yellow: Color::from_hex("#d79921")?,
            blue: Color::from_hex("#458588")?,
            magenta: Color::from_hex("#b16286")?,
            cyan: Color::from_hex("#689d6a")?,
            white: Color::from_hex("#a89984")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#928374")?,
            red: Color::from_hex("#fb4934")?,
            green: Color::from_hex("#b8bb26")?,
            yellow: Color::from_hex("#fabd2f")?,
            blue: Color::from_hex("#83a598")?,
            magenta: Color::from_hex("#d3869b")?,
            cyan: Color::from_hex("#8ec07c")?,
            white: Color::from_hex("#ebdbb2")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }

    fn gruvbox_light_palette(mut self) -> Result<Self> {
        self.name = Some("Gruvbox Light".to_string());
        self.background = Some(Color::from_hex("#fbf1c7")?);
        self.foreground = Some(Color::from_hex("#3c3836")?);
        self.accent = Some(Color::from_hex("#076678")?);
        self.details = Some(ThemeDetails::Lighter);
        
        let normal = ColorPalette {
            black: Color::from_hex("#fbf1c7")?,
            red: Color::from_hex("#cc241d")?,
            green: Color::from_hex("#98971a")?,
            yellow: Color::from_hex("#d79921")?,
            blue: Color::from_hex("#458588")?,
            magenta: Color::from_hex("#b16286")?,
            cyan: Color::from_hex("#689d6a")?,
            white: Color::from_hex("#7c6f64")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#928374")?,
            red: Color::from_hex("#9d0006")?,
            green: Color::from_hex("#79740e")?,
            yellow: Color::from_hex("#b57614")?,
            blue: Color::from_hex("#076678")?,
            magenta: Color::from_hex("#8f3f71")?,
            cyan: Color::from_hex("#427b58")?,
            white: Color::from_hex("#3c3836")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }

    fn tokyo_night_palette(mut self) -> Result<Self> {
        self.name = Some("Tokyo Night".to_string());
        self.background = Some(Color::from_hex("#1a1b26")?);
        self.foreground = Some(Color::from_hex("#c0caf5")?);
        self.accent = Some(Color::from_hex("#7aa2f7")?);
        self.details = Some(ThemeDetails::Darker);
        
        let normal = ColorPalette {
            black: Color::from_hex("#15161e")?,
            red: Color::from_hex("#f7768e")?,
            green: Color::from_hex("#9ece6a")?,
            yellow: Color::from_hex("#e0af68")?,
            blue: Color::from_hex("#7aa2f7")?,
            magenta: Color::from_hex("#bb9af7")?,
            cyan: Color::from_hex("#7dcfff")?,
            white: Color::from_hex("#a9b1d6")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#414868")?,
            red: Color::from_hex("#f7768e")?,
            green: Color::from_hex("#9ece6a")?,
            yellow: Color::from_hex("#e0af68")?,
            blue: Color::from_hex("#7aa2f7")?,
            magenta: Color::from_hex("#bb9af7")?,
            cyan: Color::from_hex("#7dcfff")?,
            white: Color::from_hex("#c0caf5")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }

    fn catppuccin_palette(mut self) -> Result<Self> {
        self.name = Some("Catppuccin".to_string());
        self.background = Some(Color::from_hex("#1e1e2e")?);
        self.foreground = Some(Color::from_hex("#cdd6f4")?);
        self.accent = Some(Color::from_hex("#89b4fa")?);
        self.details = Some(ThemeDetails::Darker);
        
        let normal = ColorPalette {
            black: Color::from_hex("#45475a")?,
            red: Color::from_hex("#f38ba8")?,
            green: Color::from_hex("#a6e3a1")?,
            yellow: Color::from_hex("#f9e2af")?,
            blue: Color::from_hex("#89b4fa")?,
            magenta: Color::from_hex("#f5c2e7")?,
            cyan: Color::from_hex("#94e2d5")?,
            white: Color::from_hex("#bac2de")?,
        };

        let bright = ColorPalette {
            black: Color::from_hex("#585b70")?,
            red: Color::from_hex("#f38ba8")?,
            green: Color::from_hex("#a6e3a1")?,
            yellow: Color::from_hex("#f9e2af")?,
            blue: Color::from_hex("#89b4fa")?,
            magenta: Color::from_hex("#f5c2e7")?,
            cyan: Color::from_hex("#94e2d5")?,
            white: Color::from_hex("#a6adc8")?,
        };

        self.terminal_colors = Some(TerminalColors { normal, bright });
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_builder_basic() {
        let theme = ThemeBuilder::new()
            .name("Test Theme")
            .accent(Color::rgb(255, 0, 0))
            .background(Color::rgb(0, 0, 0))
            .foreground(Color::rgb(255, 255, 255))
            .build()
            .unwrap();

        assert_eq!(theme.display_name(), "Test Theme");
        assert_eq!(theme.accent, Color::rgb(255, 0, 0));
    }

    #[test]
    fn test_monochromatic_generation() {
        let base_color = Color::rgb(100, 150, 200);
        let builder = ThemeBuilder::from_strategy(
            GenerationStrategy::MonochromaticFromColor(base_color)
        ).unwrap();
        
        let theme = builder.build().unwrap();
        assert!(theme.accent == base_color);
    }

    #[test]
    fn test_popular_palette() {
        let builder = ThemeBuilder::from_strategy(
            GenerationStrategy::FromPalette("dracula".to_string())
        ).unwrap();
        
        let theme = builder.build().unwrap();
        assert_eq!(theme.display_name(), "Dracula");
    }
}
