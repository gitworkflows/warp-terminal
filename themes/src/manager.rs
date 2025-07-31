//! Top-level theme manager for discovering, loading, and accessing themes

use crate::{Theme, ThemeLoader};
use std::collections::BTreeMap;

/// Theme categories for organization
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ThemeCategory {
    Standard,
    Base16,
    SpecialEdition,
    WarpBundled,
    User,
}

impl ThemeCategory {
    /// Get directory name for this category
    pub fn directory_name(&self) -> &'static str {
        match self {
            ThemeCategory::Base16 => "base16",
            ThemeCategory::Standard => "standard", 
            ThemeCategory::SpecialEdition => "special_edition",
            ThemeCategory::WarpBundled => "warp_bundled",
            ThemeCategory::User => "custom",
        }
    }
}

/// Central theme management
#[derive(Clone, Debug, Default)]
pub struct ThemeManager {
    themes: BTreeMap<String, Theme>,
    categories: BTreeMap<ThemeCategory, Vec<String>>,
}

impl ThemeManager {
    /// Create a new, empty theme manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add bundled themes to the manager
    pub fn with_bundled_themes(mut self) -> Self {
        if let Ok(themes) = ThemeLoader::load_bundled() {
            for theme in themes {
                self.add_theme(theme, ThemeCategory::WarpBundled);
            }
        }
        self
    }

    /// Load themes from the themes directory
    pub fn with_themes_directory<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        let base_path = path.as_ref();
        
        // Load each category
        for category in [
            ThemeCategory::Base16,
            ThemeCategory::Standard,
            ThemeCategory::SpecialEdition,
            ThemeCategory::WarpBundled,
            ThemeCategory::User,
        ] {
            let category_path = base_path.join(category.directory_name());
            if category_path.exists() {
                if let Ok(themes) = ThemeLoader::load_from_directory(&category_path) {
                    for theme in themes {
                        self.add_theme(theme, category.clone());
                    }
                }
            }
        }
        
        self
    }

    /// Add a single theme to the manager
    pub fn add_theme(&mut self, theme: Theme, category: ThemeCategory) {
        let name = theme.display_name().to_lowercase();
        
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());
            
        self.themes.insert(name, theme);
    }

    /// Get a theme by name (case-insensitive)
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(&name.to_lowercase())
    }

    /// List all theme names
    pub fn list_theme_names(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// List themes by category
    pub fn list_themes_by_category(&self, category: ThemeCategory) -> Vec<&Theme> {
        self.categories
            .get(&category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.themes.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all categories with theme counts
    pub fn category_summary(&self) -> BTreeMap<ThemeCategory, usize> {
        self.categories
            .iter()
            .map(|(cat, themes)| (cat.clone(), themes.len()))
            .collect()
    }

    /// Search themes by name pattern
    pub fn search_themes(&self, pattern: &str) -> Vec<&Theme> {
        let pattern = pattern.to_lowercase();
        self.themes
            .iter()
            .filter(|(name, _)| name.contains(&pattern))
            .map(|(_, theme)| theme)
            .collect()
    }

    /// Get total number of themes
    pub fn theme_count(&self) -> usize {
        self.themes.len()
    }
}
