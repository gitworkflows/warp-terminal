//! Basic usage example of the Warp themes library

use warp_themes::{ThemeManager, ThemeCategory, Color, ColorPalette, TerminalColors, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Warp Themes Integration Example");
    println!("==================================");

    // Create a theme manager and load themes from the themes directory
    let themes_dir = std::env::current_dir()?.parent().unwrap().join("warp").join("themes");
    
    let manager = if themes_dir.exists() {
        println!("📁 Loading themes from: {}", themes_dir.display());
        ThemeManager::new().with_themes_directory(&themes_dir)
    } else {
        println!("⚠️  Themes directory not found, using empty manager");
        ThemeManager::new().with_bundled_themes()
    };

    // Show summary
    println!("\n📊 Theme Summary:");
    println!("Total themes loaded: {}", manager.theme_count());
    
    let summary = manager.category_summary();
    for (category, count) in summary {
        println!("  {:?}: {} themes", category, count);
    }

    // Try to get a specific theme
    if let Some(theme) = manager.get_theme("dracula") {
        println!("\n🧛 Found Dracula theme:");
        println!("  Background: {}", theme.background);
        println!("  Foreground: {}", theme.foreground);
        println!("  Accent: {}", theme.accent);
        println!("  Is dark theme: {}", theme.is_dark());
    } else {
        // Create a sample theme manually
        println!("\n🎨 Creating a sample theme:");
        let sample_theme = create_sample_theme();
        println!("  Created theme: {}", sample_theme.display_name());
        println!("  Background: {}", sample_theme.background);
        println!("  Foreground: {}", sample_theme.foreground);
    }

    // Search for themes
    let search_results = manager.search_themes("dark");
    if !search_results.is_empty() {
        println!("\n🔍 Themes matching 'dark' (showing first 5):");
        for theme in search_results.iter().take(5) {
            println!("  - {}", theme.display_name());
        }
    }

    println!("\n✅ Themes integration example completed!");
    Ok(())
}

fn create_sample_theme() -> Theme {
    let normal_palette = ColorPalette {
        black: Color::from_hex("#000000").unwrap(),
        red: Color::from_hex("#FF5555").unwrap(),
        green: Color::from_hex("#50FA7B").unwrap(),
        yellow: Color::from_hex("#F1FA8C").unwrap(),
        blue: Color::from_hex("#BD93F9").unwrap(),
        magenta: Color::from_hex("#FF79C6").unwrap(),
        cyan: Color::from_hex("#8BE9FD").unwrap(),
        white: Color::from_hex("#BBBBBB").unwrap(),
    };

    let bright_palette = ColorPalette {
        black: Color::from_hex("#555555").unwrap(),
        red: Color::from_hex("#FF5555").unwrap(),
        green: Color::from_hex("#50FA7B").unwrap(),
        yellow: Color::from_hex("#F1FA8C").unwrap(),
        blue: Color::from_hex("#CAA9FA").unwrap(),
        magenta: Color::from_hex("#FF79C6").unwrap(),
        cyan: Color::from_hex("#8BE9FD").unwrap(),
        white: Color::from_hex("#FFFFFF").unwrap(),
    };

    let terminal_colors = TerminalColors {
        normal: normal_palette,
        bright: bright_palette,
    };

    let mut theme = Theme::new(
        Color::from_hex("#BD93F9").unwrap(), // accent
        Color::from_hex("#282A36").unwrap(), // background
        Color::from_hex("#F8F8F2").unwrap(), // foreground
        terminal_colors,
    );

    theme.name = Some("Sample Dracula".to_string());
    theme
}
