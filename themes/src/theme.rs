//! Core theme data structures

use crate::{Color, TerminalColors, Result, ThemeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name (derived from filename if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Theme version (defaults to current version)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// Main accent color
    pub accent: Color,
    
    /// Cursor color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Color>,
    
    /// Background color
    pub background: Color,
    
    /// Foreground/text color
    pub foreground: Color,
    
    /// Theme details (lighter/darker)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ThemeDetails>,
    
    /// Terminal color palette
    pub terminal_colors: TerminalColors,
    
    /// Optional background image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image: Option<BackgroundImage>,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

impl Theme {
    /// Create a new theme with required fields
    pub fn new(
        accent: Color,
        background: Color,
        foreground: Color,
        terminal_colors: TerminalColors,
    ) -> Self {
        Self {
            name: None,
            version: Some(crate::THEME_FORMAT_VERSION.to_string()),
            accent,
            cursor: None,
            background,
            foreground,
            details: None,
            terminal_colors,
            background_image: None,
            metadata: HashMap::new(),
        }
    }

    /// Get the theme name, using filename as fallback
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| "Unnamed Theme".to_string())
    }

    /// Check if theme is considered dark based on background color
    pub fn is_dark(&self) -> bool {
        self.background.is_dark()
    }

    /// Validate theme structure and colors
    pub fn validate(&self) -> Result<()> {
        // Check that all required colors are present
        if self.accent.a == 0 && self.background.a == 0 && self.foreground.a == 0 {
            return Err(ThemeError::ValidationError(
                "Theme has no visible colors".to_string(),
            ));
        }

        // Validate terminal colors
        let colors_to_check = [
            self.terminal_colors.normal.black,
            self.terminal_colors.normal.red,
            self.terminal_colors.normal.green,
            self.terminal_colors.normal.yellow,
            self.terminal_colors.normal.blue,
            self.terminal_colors.normal.magenta,
            self.terminal_colors.normal.cyan,
            self.terminal_colors.normal.white,
        ];

        let bright_colors_to_check = [
            self.terminal_colors.bright.black,
            self.terminal_colors.bright.red,
            self.terminal_colors.bright.green,
            self.terminal_colors.bright.yellow,
            self.terminal_colors.bright.blue,
            self.terminal_colors.bright.magenta,
            self.terminal_colors.bright.cyan,
            self.terminal_colors.bright.white,
        ];

        // Check for any obviously invalid colors (all zeros except for alpha)
        for color in colors_to_check.iter().chain(bright_colors_to_check.iter()) {
            if color.r == 0 && color.g == 0 && color.b == 0 && color.a == 0 {
                return Err(ThemeError::ValidationError(
                    "Theme contains invalid transparent colors".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Convert theme to YAML format
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(ThemeError::from)
    }

    /// Load theme from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let mut theme: Theme = serde_yaml::from_str(yaml)?;
        
        // Set version if not present
        if theme.version.is_none() {
            theme.version = Some(crate::THEME_FORMAT_VERSION.to_string());
        }

        theme.validate()?;
        Ok(theme)
    }

    /// Provides a default dark theme as a fallback
    pub fn default_dark() -> Self {
        Theme {
            name: Some("Default Dark".to_string()),
            version: Some("1.0.0".to_string()),
            accent: Color::new(0, 128, 255, 255), // A nice blue
            cursor: None,
            background: Color::new(20, 20, 20, 255),
            foreground: Color::new(220, 220, 220, 255),
            details: Some(ThemeDetails::Darker),
            terminal_colors: TerminalColors::default_dark(),
            background_image: None,
            metadata: HashMap::new(),
        }
    }

    /// Provides a default light theme as a fallback
    pub fn default_light() -> Self {
        Theme {
            name: Some("Default Light".to_string()),
            version: Some("1.0.0".to_string()),
            accent: Color::new(0, 0, 0, 255), // A nice blue
            cursor: None,
            background: Color::new(255, 255, 255, 255),
            foreground: Color::new(0, 0, 0, 255),
            details: Some(ThemeDetails::Lighter),
            terminal_colors: TerminalColors::default_light(),
            background_image: None,
            metadata: HashMap::new(),
        }
    }
}

/// Theme details specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeDetails {
    Darker,
    Lighter,
}

/// Background image configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundImage {
    /// Path to background image (relative to theme directory or absolute)
    pub path: PathBuf,
    
    /// Background opacity (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    
    /// Background blur amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f32>,
}

impl BackgroundImage {
    /// Create a new background image configuration
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            opacity: None,
            blur: None,
        }
    }

    /// Set opacity (clamped to 0.0-1.0)
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Set blur amount
    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur = Some(blur.max(0.0));
        self
    }
}
