//! Theme analysis and color harmony detection
//!
//! This module provides advanced analysis capabilities for themes including:
//! - Color harmony detection
//! - Contrast ratio calculations
//! - Accessibility analysis
//! - Theme compatibility scoring

use crate::{Theme, Color, Result};
use std::collections::HashMap;

/// Color harmony types
#[derive(Debug, Clone, PartialEq)]
pub enum ColorHarmony {
    Monochromatic,
    Analogous,
    Complementary,
    Triadic,
    Tetradic,
    SplitComplementary,
    Unknown,
}

/// Accessibility compliance levels
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityLevel {
    AA,        // WCAG 2.1 AA compliant
    AAA,       // WCAG 2.1 AAA compliant
    Partial,   // Some contrast issues
    Poor,      // Many contrast issues
}

/// Theme analysis results
#[derive(Debug, Clone)]
pub struct ThemeAnalysis {
    /// Overall theme score (0-100)
    pub score: f64,
    /// Detected color harmony
    pub harmony: ColorHarmony,
    /// Accessibility compliance level
    pub accessibility: AccessibilityLevel,
    /// Contrast ratios for key color pairs
    pub contrast_ratios: HashMap<String, f64>,
    /// Color temperature (warm/cool bias)
    pub temperature: f64,
    /// Readability score
    pub readability_score: f64,
    /// Suggested improvements
    pub suggestions: Vec<String>,
}

/// Theme analyzer
pub struct ThemeAnalyzer;

impl ThemeAnalyzer {
    /// Perform comprehensive theme analysis
    pub fn analyze(theme: &Theme) -> Result<ThemeAnalysis> {
        let harmony = Self::detect_harmony(theme);
        let contrast_ratios = Self::calculate_contrast_ratios(theme);
        let accessibility = Self::assess_accessibility(&contrast_ratios);
        let temperature = Self::calculate_temperature(theme);
        let readability_score = Self::calculate_readability(theme, &contrast_ratios);
        let suggestions = Self::generate_suggestions(theme, &contrast_ratios);
        
        let score = Self::calculate_overall_score(
            &harmony,
            &accessibility,
            readability_score,
            temperature,
        );
        
        Ok(ThemeAnalysis {
            score,
            harmony,
            accessibility,
            contrast_ratios,
            temperature,
            readability_score,
            suggestions,
        })
    }

    /// Detect color harmony in the theme
    fn detect_harmony(theme: &Theme) -> ColorHarmony {
        let colors = vec![
            theme.accent,
            theme.background,
            theme.foreground,
            theme.terminal_colors.normal.red,
            theme.terminal_colors.normal.green,
            theme.terminal_colors.normal.blue,
        ];

        let hues: Vec<f64> = colors.iter()
            .map(|c| c.to_hsl().0)
            .collect();

        // Analyze hue relationships
        Self::analyze_hue_relationships(&hues)
    }

    /// Analyze hue relationships to determine harmony type
    fn analyze_hue_relationships(hues: &[f64]) -> ColorHarmony {
        if hues.len() < 2 {
            return ColorHarmony::Unknown;
        }

        let mut differences = Vec::new();
        for i in 0..hues.len() {
            for j in (i + 1)..hues.len() {
                let diff = (hues[i] - hues[j]).abs().min(360.0 - (hues[i] - hues[j]).abs());
                differences.push(diff);
            }
        }

        // Check for complementary (180°)
        if differences.iter().any(|&d| (d - 180.0).abs() < 15.0) {
            return ColorHarmony::Complementary;
        }

        // Check for triadic (120°)
        if differences.iter().any(|&d| (d - 120.0).abs() < 15.0) {
            return ColorHarmony::Triadic;
        }

        // Check for analogous (30-60°)
        if differences.iter().all(|&d| d < 60.0) {
            return ColorHarmony::Analogous;
        }

        // Check for monochromatic (very small differences)
        if differences.iter().all(|&d| d < 15.0) {
            return ColorHarmony::Monochromatic;
        }

        ColorHarmony::Unknown
    }

    /// Calculate contrast ratios for important color pairs
    fn calculate_contrast_ratios(theme: &Theme) -> HashMap<String, f64> {
        let mut ratios = HashMap::new();

        // Background to foreground (most important)
        ratios.insert(
            "background_foreground".to_string(),
            Self::contrast_ratio(theme.background, theme.foreground),
        );

        // Background to accent
        ratios.insert(
            "background_accent".to_string(),
            Self::contrast_ratio(theme.background, theme.accent),
        );

        // Terminal colors against background
        let terminal_colors = [
            ("red", theme.terminal_colors.normal.red),
            ("green", theme.terminal_colors.normal.green),
            ("blue", theme.terminal_colors.normal.blue),
            ("yellow", theme.terminal_colors.normal.yellow),
            ("cyan", theme.terminal_colors.normal.cyan),
            ("magenta", theme.terminal_colors.normal.magenta),
        ];

        for (name, color) in terminal_colors {
            ratios.insert(
                format!("background_{}", name),
                Self::contrast_ratio(theme.background, color),
            );
        }

        ratios
    }

    /// Calculate contrast ratio between two colors
    fn contrast_ratio(color1: Color, color2: Color) -> f64 {
        let l1 = Self::relative_luminance(color1);
        let l2 = Self::relative_luminance(color2);
        
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Calculate relative luminance of a color
    fn relative_luminance(color: Color) -> f64 {
        let to_linear = |c: u8| -> f64 {
            let c = c as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * to_linear(color.r) + 0.7152 * to_linear(color.g) + 0.0722 * to_linear(color.b)
    }

    /// Assess accessibility compliance
    fn assess_accessibility(contrast_ratios: &HashMap<String, f64>) -> AccessibilityLevel {
        let bg_fg_ratio = contrast_ratios.get("background_foreground").unwrap_or(&1.0);
        
        // Check main text contrast
        if *bg_fg_ratio >= 7.0 {
            // Check all other ratios for AAA compliance
            let all_good = contrast_ratios.values().all(|&ratio| ratio >= 4.5);
            if all_good {
                AccessibilityLevel::AAA
            } else {
                AccessibilityLevel::AA
            }
        } else if *bg_fg_ratio >= 4.5 {
            AccessibilityLevel::AA
        } else if *bg_fg_ratio >= 3.0 {
            AccessibilityLevel::Partial
        } else {
            AccessibilityLevel::Poor
        }
    }

    /// Calculate color temperature bias
    fn calculate_temperature(theme: &Theme) -> f64 {
        let colors = vec![
            theme.accent,
            theme.background,
            theme.foreground,
        ];

        let mut warm_score = 0.0;
        let mut cool_score = 0.0;

        for color in colors {
            let (hue, _saturation, _lightness) = color.to_hsl();
            
            // Warm colors: red to yellow (0-60° and 300-360°)
            // Cool colors: cyan to blue (180-300°)
            if hue <= 60.0 || hue >= 300.0 {
                warm_score += 1.0;
            } else if hue >= 180.0 && hue <= 300.0 {
                cool_score += 1.0;
            }
        }

        // Return temperature bias (-1 to 1, negative = cool, positive = warm)
        (warm_score - cool_score) / (warm_score + cool_score + 1.0)
    }

    /// Calculate readability score
    fn calculate_readability(theme: &Theme, contrast_ratios: &HashMap<String, f64>) -> f64 {
        let bg_fg_ratio = contrast_ratios.get("background_foreground").unwrap_or(&1.0);
        
        // Base score from main contrast ratio
        let base_score = (*bg_fg_ratio / 21.0).min(1.0) * 70.0; // Max 70 points
        
        // Color differentiation score
        let color_diff_score = Self::calculate_color_differentiation(theme) * 20.0; // Max 20 points
        
        // Terminal color visibility score
        let terminal_score = Self::calculate_terminal_visibility(contrast_ratios) * 10.0; // Max 10 points
        
        base_score + color_diff_score + terminal_score
    }

    /// Calculate color differentiation score
    fn calculate_color_differentiation(theme: &Theme) -> f64 {
        let terminal_colors = [
            theme.terminal_colors.normal.red,
            theme.terminal_colors.normal.green,
            theme.terminal_colors.normal.blue,
            theme.terminal_colors.normal.yellow,
            theme.terminal_colors.normal.cyan,
            theme.terminal_colors.normal.magenta,
        ];

        let mut min_distance = f64::MAX;
        
        for i in 0..terminal_colors.len() {
            for j in (i + 1)..terminal_colors.len() {
                let distance = Self::color_distance(terminal_colors[i], terminal_colors[j]);
                min_distance = min_distance.min(distance);
            }
        }

        // Normalize to 0-1 range
        (min_distance / 255.0).min(1.0)
    }

    /// Calculate Euclidean distance between two colors in RGB space
    fn color_distance(color1: Color, color2: Color) -> f64 {
        let dr = (color1.r as f64 - color2.r as f64).powi(2);
        let dg = (color1.g as f64 - color2.g as f64).powi(2);
        let db = (color1.b as f64 - color2.b as f64).powi(2);
        (dr + dg + db).sqrt()
    }

    /// Calculate terminal color visibility score
    fn calculate_terminal_visibility(contrast_ratios: &HashMap<String, f64>) -> f64 {
        let terminal_keys = ["red", "green", "blue", "yellow", "cyan", "magenta"];
        let mut total_score = 0.0;
        let mut count = 0;

        for key in &terminal_keys {
            if let Some(&ratio) = contrast_ratios.get(&format!("background_{}", key)) {
                total_score += (ratio / 7.0).min(1.0); // Normalize against AAA standard
                count += 1;
            }
        }

        if count > 0 {
            total_score / count as f64
        } else {
            0.0
        }
    }

    /// Generate improvement suggestions
    fn generate_suggestions(theme: &Theme, contrast_ratios: &HashMap<String, f64>) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check main contrast
        if let Some(&bg_fg_ratio) = contrast_ratios.get("background_foreground") {
            if bg_fg_ratio < 4.5 {
                suggestions.push("Consider increasing contrast between background and foreground colors for better readability".to_string());
            }
        }

        // Check accent visibility
        if let Some(&bg_accent_ratio) = contrast_ratios.get("background_accent") {
            if bg_accent_ratio < 3.0 {
                suggestions.push("Accent color may be too similar to background - consider a more contrasting color".to_string());
            }
        }

        // Check terminal colors
        let terminal_colors = ["red", "green", "blue", "yellow", "cyan", "magenta"];
        let mut poor_colors = Vec::new();
        
        for color in &terminal_colors {
            if let Some(&ratio) = contrast_ratios.get(&format!("background_{}", color)) {
                if ratio < 3.0 {
                    poor_colors.push(*color);
                }
            }
        }

        if !poor_colors.is_empty() {
            suggestions.push(format!(
                "Terminal colors with poor contrast: {}. Consider adjusting these colors.",
                poor_colors.join(", ")
            ));
        }

        // Color harmony suggestions
        let harmony = Self::detect_harmony(theme);
        match harmony {
            ColorHarmony::Unknown => {
                suggestions.push("Color scheme lacks clear harmony - consider using complementary or analogous colors".to_string());
            }
            _ => {}
        }

        suggestions
    }

    /// Calculate overall theme score
    fn calculate_overall_score(
        harmony: &ColorHarmony,
        accessibility: &AccessibilityLevel,
        readability_score: f64,
        _temperature: f64,
    ) -> f64 {
        let mut score = readability_score; // Base score from readability (0-100)

        // Harmony bonus
        let harmony_bonus = match harmony {
            ColorHarmony::Complementary | ColorHarmony::Analogous => 10.0,
            ColorHarmony::Triadic | ColorHarmony::Tetradic => 8.0,
            ColorHarmony::Monochromatic => 5.0,
            ColorHarmony::SplitComplementary => 6.0,
            ColorHarmony::Unknown => -5.0,
        };

        // Accessibility bonus
        let accessibility_bonus = match accessibility {
            AccessibilityLevel::AAA => 15.0,
            AccessibilityLevel::AA => 10.0,
            AccessibilityLevel::Partial => 0.0,
            AccessibilityLevel::Poor => -10.0,
        };

        score += harmony_bonus + accessibility_bonus;
        score.max(0.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TerminalColors;

    #[test]
    fn test_contrast_ratio_calculation() {
        let white = Color::rgb(255, 255, 255);
        let black = Color::rgb(0, 0, 0);
        
        let ratio = ThemeAnalyzer::contrast_ratio(white, black);
        assert!(ratio > 20.0); // Should be 21:1 for perfect white/black
    }

    #[test]
    fn test_theme_analysis() {
        let theme = Theme::new(
            Color::rgb(0, 123, 255), // accent
            Color::rgb(30, 30, 30),  // background
            Color::rgb(240, 240, 240), // foreground
            TerminalColors::default_dark(),
        );

        let analysis = ThemeAnalyzer::analyze(&theme).unwrap();
        assert!(analysis.score >= 0.0 && analysis.score <= 100.0);
        assert!(!analysis.suggestions.is_empty() || analysis.score > 80.0);
    }
}
