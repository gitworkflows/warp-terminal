//! Theme validation logic for ensuring themes meet quality standards

use crate::{Theme, Result};

/// Validates a theme against a set of rules
pub fn validate_theme(theme: &Theme) -> Result<()> {
    theme.validate()
}

/// Theme validation rules and utilities
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validate a theme for completeness and consistency
    pub fn validate_theme(theme: &Theme) -> Result<()> {
        theme.validate()
    }

    /// Check if theme has good contrast ratios
    pub fn check_contrast(theme: &Theme) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check background vs foreground contrast
        let bg_luminance = Self::calculate_luminance(theme.background);
        let fg_luminance = Self::calculate_luminance(theme.foreground);
        
        let contrast_ratio = if bg_luminance > fg_luminance {
            (bg_luminance + 0.05) / (fg_luminance + 0.05)
        } else {
            (fg_luminance + 0.05) / (bg_luminance + 0.05)
        };

        if contrast_ratio < 4.5 {
            warnings.push(format!(
                "Low contrast ratio between background and foreground: {:.2}", 
                contrast_ratio
            ));
        }

        warnings
    }

    /// Calculate relative luminance of a color
    fn calculate_luminance(color: crate::Color) -> f64 {
        let r = Self::linearize_color_component(color.r as f64 / 255.0);
        let g = Self::linearize_color_component(color.g as f64 / 255.0);
        let b = Self::linearize_color_component(color.b as f64 / 255.0);
        
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Linearize color component for luminance calculation
    fn linearize_color_component(c: f64) -> f64 {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
}
