/*!
 * Enhanced Theme CLI Tool
 * 
 * This example demonstrates advanced theme management features including:
 * - Theme analysis and accessibility scoring
 * - Automatic theme generation from colors
 * - Multi-format preview generation
 * - Theme optimization suggestions
 */

use std::path::Path;
use warp_themes::{
    ThemeManager, ThemeBuilder, ThemeAnalyzer, PreviewGenerator,
    Color, GenerationStrategy, PreviewOptions, PreviewFormat,
    AccessibilityLevel, ColorHarmony
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Enhanced Warp Theme Manager");
    println!("===============================\n");

    // Initialize theme manager with bundled themes
    let manager = ThemeManager::new().with_bundled_themes();
    
    println!("📊 Theme Library Overview:");
    println!("  Total themes: {}", manager.theme_count());
    
    let summary = manager.category_summary();
    for (category, count) in summary {
        println!("  {:?}: {} themes", category, count);
    }
    println!();

    // Demonstrate theme analysis
    demonstrate_theme_analysis(&manager)?;
    
    // Demonstrate theme generation
    demonstrate_theme_generation()?;
    
    // Demonstrate preview generation
    demonstrate_preview_generation()?;
    
    // Demonstrate theme comparison
    demonstrate_theme_comparison(&manager)?;

    Ok(())
}

fn demonstrate_theme_analysis(manager: &ThemeManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Theme Analysis Demo");
    println!("======================");
    
    // Analyze some popular themes
    let themes_to_analyze = ["dracula", "nord", "solarized_dark", "gruvbox_dark"];
    
    for theme_name in &themes_to_analyze {
        if let Some(theme) = manager.get_theme(theme_name) {
            println!("\n📋 Analyzing: {}", theme.display_name());
            println!("   Colors: bg={}, fg={}, accent={}", 
                    theme.background.to_hex(), 
                    theme.foreground.to_hex(), 
                    theme.accent.to_hex());
            
            let analysis = ThemeAnalyzer::analyze(theme)?;
            
            println!("   📊 Overall Score: {:.1}/100", analysis.score);
            println!("   🎨 Harmony: {:?}", analysis.harmony);
            println!("   ♿ Accessibility: {:?}", analysis.accessibility);
            println!("   📖 Readability: {:.1}/100", analysis.readability_score);
            println!("   🌡️  Temperature: {:.2} ({})", 
                    analysis.temperature,
                    if analysis.temperature > 0.1 { "warm" } 
                    else if analysis.temperature < -0.1 { "cool" } 
                    else { "neutral" });
            
            // Show contrast ratios
            println!("   📐 Key Contrasts:");
            for (pair, ratio) in &analysis.contrast_ratios {
                if pair.contains("background_") {
                    let color_name = pair.strip_prefix("background_").unwrap_or(pair);
                    let status = if *ratio >= 7.0 { "AAA ✓" } 
                                else if *ratio >= 4.5 { "AA ✓" } 
                                else if *ratio >= 3.0 { "⚠️" } 
                                else { "❌" };
                    println!("      {}: {:.1}:1 {}", color_name, ratio, status);
                }
            }
            
            // Show suggestions
            if !analysis.suggestions.is_empty() {
                println!("   💡 Suggestions:");
                for suggestion in &analysis.suggestions {
                    println!("      • {}", suggestion);
                }
            }
        }
    }
    
    println!();
    Ok(())
}

fn demonstrate_theme_generation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Theme Generation Demo");
    println!("========================");
    
    // Generate themes using different strategies
    let base_colors = [
        ("Ocean Blue", Color::from_hex("#0077be")?),
        ("Forest Green", Color::from_hex("#2d5a27")?),
        ("Sunset Orange", Color::from_hex("#ff6b35")?),
        ("Purple Rain", Color::from_hex("#6a4c93")?),
    ];
    
    for (color_name, base_color) in &base_colors {
        println!("\n🎨 Generating themes from {} ({})", color_name, base_color.to_hex());
        
        let strategies = [
            ("Monochromatic", GenerationStrategy::MonochromaticFromColor(*base_color)),
            ("Complementary", GenerationStrategy::ComplementaryFromColor(*base_color)),
            ("Analogous", GenerationStrategy::AnalogousFromColor(*base_color)),
            ("Triadic", GenerationStrategy::TriadicFromColor(*base_color)),
        ];
        
        for (strategy_name, strategy) in &strategies {
            let theme = ThemeBuilder::from_strategy(strategy.clone())?
                .name(format!("{} {}", color_name, strategy_name))
                .build()?;
            
            let analysis = ThemeAnalyzer::analyze(&theme)?;
            
            println!("   {} Theme:", strategy_name);
            println!("     Background: {}", theme.background.to_hex());
            println!("     Foreground: {}", theme.foreground.to_hex());
            println!("     Score: {:.1}/100, Harmony: {:?}", analysis.score, analysis.harmony);
        }
    }
    
    println!();
    Ok(())
}

fn demonstrate_preview_generation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖼️  Preview Generation Demo");
    println!("============================");
    
    // Create a custom theme for preview demonstration
    let custom_theme = ThemeBuilder::from_strategy(
        GenerationStrategy::FromPalette("tokyo_night".to_string())
    )?
    .name("Tokyo Night Demo")
    .build()?;
    
    println!("Generated preview theme: {}", custom_theme.display_name());
    println!("  Background: {}", custom_theme.background.to_hex());
    println!("  Foreground: {}", custom_theme.foreground.to_hex());
    println!("  Accent: {}", custom_theme.accent.to_hex());
    
    // Generate different preview formats
    let preview_formats = [
        ("Compact SVG", PreviewFormat::Svg, Some("compact".to_string())),
        ("Detailed SVG", PreviewFormat::Svg, Some("detailed".to_string())),
        ("Terminal SVG", PreviewFormat::Svg, Some("terminal".to_string())),
        ("JSON Data", PreviewFormat::Json, None),
    ];
    
    for (format_name, format, template) in &preview_formats {
        let options = PreviewOptions {
            format: format.clone(),
            width: 400,
            height: 300,
            show_terminal_colors: true,
            show_code_sample: true,
            template_name: template.clone(),
        };
        
        let preview = PreviewGenerator::generate_preview(&custom_theme, &options)?;
        
        println!("\n📄 {} Preview ({} chars):", format_name, preview.len());
        
        match format {
            PreviewFormat::Json => {
                // Pretty print first few lines of JSON
                let lines: Vec<&str> = preview.lines().take(10).collect();
                for line in lines {
                    println!("    {}", line);
                }
                if preview.lines().count() > 10 {
                    println!("    ... ({} more lines)", preview.lines().count() - 10);
                }
            },
            PreviewFormat::Svg => {
                println!("    Generated SVG with {} elements", preview.matches("<").count());
            },
            _ => {
                println!("    Preview generated successfully");
            }
        }
    }
    
    println!();
    Ok(())
}

fn demonstrate_theme_comparison(manager: &ThemeManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚖️  Theme Comparison Demo");
    println!("=========================");
    
    // Compare themes from different categories
    let themes_to_compare = [
        ("dracula", "Popular dark theme"),
        ("nord", "Minimal dark theme"),
        ("solarized_light", "Light theme"),
        ("gruvbox_dark", "Retro dark theme"),
    ];
    
    println!("Comparing themes by various metrics:\n");
    println!("{:<20} {:<8} {:<15} {:<12} {:<10}", 
             "Theme", "Score", "Harmony", "Accessibility", "Temperature");
    println!("{:-<75}", "");
    
    for (theme_name, description) in &themes_to_compare {
        if let Some(theme) = manager.get_theme(theme_name) {
            let analysis = ThemeAnalyzer::analyze(theme)?;
            
            let temperature_str = if analysis.temperature > 0.1 { "Warm" } 
                                 else if analysis.temperature < -0.1 { "Cool" } 
                                 else { "Neutral" };
            
            let harmony_str = match analysis.harmony {
                ColorHarmony::Complementary => "Complementary",
                ColorHarmony::Analogous => "Analogous",
                ColorHarmony::Monochromatic => "Monochromatic",
                ColorHarmony::Triadic => "Triadic",
                ColorHarmony::Tetradic => "Tetradic",
                ColorHarmony::SplitComplementary => "Split-Comp",
                ColorHarmony::Unknown => "Unknown",
            };
            
            let accessibility_str = match analysis.accessibility {
                AccessibilityLevel::AAA => "AAA ✓",
                AccessibilityLevel::AA => "AA ✓",
                AccessibilityLevel::Partial => "Partial ⚠️",
                AccessibilityLevel::Poor => "Poor ❌",
            };
            
            println!("{:<20} {:<8.1} {:<15} {:<12} {:<10}", 
                    theme.display_name(), 
                    analysis.score,
                    harmony_str,
                    accessibility_str,
                    temperature_str);
        }
    }
    
    println!();
    
    // Find best themes by different criteria
    let all_themes: Vec<_> = ["dracula", "nord", "solarized_dark", "solarized_light", 
                             "gruvbox_dark", "gruvbox_light", "tokyo_night"]
        .iter()
        .filter_map(|name| manager.get_theme(name))
        .collect();
    
    if !all_themes.is_empty() {
        println!("🏆 Theme Recommendations:");
        
        // Best overall score
        let mut scored_themes: Vec<_> = all_themes.iter()
            .map(|theme| (theme, ThemeAnalyzer::analyze(theme).unwrap_or_else(|_| {
                // Create a default analysis if analysis fails
                warp_themes::analyzer::ThemeAnalysis {
                    score: 0.0,
                    harmony: ColorHarmony::Unknown,
                    accessibility: AccessibilityLevel::Poor,
                    contrast_ratios: std::collections::HashMap::new(),
                    temperature: 0.0,
                    readability_score: 0.0,
                    suggestions: vec![],
                }
            })))
            .collect();
        
        scored_themes.sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some((best_theme, best_analysis)) = scored_themes.first() {
            println!("  🥇 Highest Overall Score: {} ({:.1}/100)", 
                    best_theme.display_name(), best_analysis.score);
        }
        
        // Best accessibility
        let best_accessible = scored_themes.iter()
            .filter(|(_, analysis)| matches!(analysis.accessibility, AccessibilityLevel::AAA | AccessibilityLevel::AA))
            .max_by(|a, b| a.1.readability_score.partial_cmp(&b.1.readability_score).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some((theme, analysis)) = best_accessible {
            println!("  ♿ Most Accessible: {} ({:?}, {:.1}/100 readability)", 
                    theme.display_name(), analysis.accessibility, analysis.readability_score);
        }
        
        // Most harmonious
        let best_harmony = scored_themes.iter()
            .filter(|(_, analysis)| !matches!(analysis.harmony, ColorHarmony::Unknown))
            .max_by_key(|(_, analysis)| match analysis.harmony {
                ColorHarmony::Complementary | ColorHarmony::Analogous => 3,
                ColorHarmony::Triadic | ColorHarmony::SplitComplementary => 2,
                ColorHarmony::Monochromatic => 1,
                _ => 0,
            });
        
        if let Some((theme, analysis)) = best_harmony {
            println!("  🎨 Best Color Harmony: {} ({:?})", 
                    theme.display_name(), analysis.harmony);
        }
    }
    
    println!();
    Ok(())
}

/// Utility function to print a color palette
fn _print_color_palette(theme: &warp_themes::Theme) {
    println!("  Color Palette:");
    println!("    🔴 Red: {} / {}", 
            theme.terminal_colors.normal.red.to_hex(),
            theme.terminal_colors.bright.red.to_hex());
    println!("    🟢 Green: {} / {}", 
            theme.terminal_colors.normal.green.to_hex(),
            theme.terminal_colors.bright.green.to_hex());
    println!("    🟡 Yellow: {} / {}", 
            theme.terminal_colors.normal.yellow.to_hex(),
            theme.terminal_colors.bright.yellow.to_hex());
    println!("    🔵 Blue: {} / {}", 
            theme.terminal_colors.normal.blue.to_hex(),
            theme.terminal_colors.bright.blue.to_hex());
    println!("    🟣 Magenta: {} / {}", 
            theme.terminal_colors.normal.magenta.to_hex(),
            theme.terminal_colors.bright.magenta.to_hex());
    println!("    🟦 Cyan: {} / {}", 
            theme.terminal_colors.normal.cyan.to_hex(),
            theme.terminal_colors.bright.cyan.to_hex());
}
