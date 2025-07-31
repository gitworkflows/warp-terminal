//! # Warp Themes
//!
//! A comprehensive theme system for Warp terminal with support for:
//! - 300+ pre-built themes from popular collections (Base16, standard themes)
//! - Custom theme loading from YAML files
//! - Runtime theme switching
//! - Color validation and manipulation
//! - Integration with UI frameworks (Iced)
//!
//! ## Features
//!
//! - **Base16 Themes**: 180+ themes based on the Base16 color scheme architecture
//! - **Standard Themes**: 120+ popular themes (Dracula, Nord, Solarized, etc.)
//! - **Special Edition**: Curated themes for holidays and special occasions
//! - **Warp Bundled**: Official themes that ship with Warp
//! - **Custom Backgrounds**: Support for background images
//!
//! ## Usage
//!
//! ```rust
//! use warp_themes::{ThemeManager, ThemeCategory};
//!
//! // Load theme manager with all bundled themes
//! let manager = ThemeManager::new().with_bundled_themes();
//!
//! // Get a specific theme
//! if let Some(theme) = manager.get_theme("dracula") {
//!     println!("Loaded theme: {}", theme.display_name());
//! }
//!
//! // List all themes in a category
//! let standard_themes = manager.list_themes_by_category(ThemeCategory::Standard);
//! ```

pub mod analyzer;
pub mod builder;
pub mod color;
pub mod error;
pub mod loader;
pub mod manager;
pub mod preview;
pub mod tabbed_pane;
pub mod theme;
pub mod validation;

#[cfg(feature = "iced_integration")]
pub mod iced_integration;

pub use analyzer::{ThemeAnalyzer, ThemeAnalysis, ColorHarmony, AccessibilityLevel};
pub use builder::{ThemeBuilder, GenerationStrategy};
pub use color::{Color, ColorPalette, TerminalColors};
pub use error::{ThemeError, Result};
pub use loader::{ThemeLoader, ThemeSource};
pub use manager::{ThemeManager, ThemeCategory};
pub use preview::{PreviewGenerator, PreviewOptions, PreviewFormat};
pub use theme::{Theme, ThemeDetails, BackgroundImage};

/// Current version of the theme format
pub const THEME_FORMAT_VERSION: &str = "1.0";

/// Default theme directory relative to user's config
pub const DEFAULT_THEME_DIR: &str = ".warp/themes";
