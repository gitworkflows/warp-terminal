//! Theme loader for discovering and parsing theme files from disk

use crate::{Theme, Result};
use std::path::Path;
use walkdir::WalkDir;

/// Theme source locations
#[derive(Debug, Clone)]
pub enum ThemeSource {
    /// Load from a specific file
    File(std::path::PathBuf),
    /// Load from a directory (recursively)
    Directory(std::path::PathBuf),
    /// Load bundled themes
    Bundled,
}

/// Theme loader for various sources
pub struct ThemeLoader;

impl ThemeLoader {
    /// Load a single theme from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Theme> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let mut theme = Theme::from_yaml(&content)?;
        
        // Set name from filename if not present
        if theme.name.is_none() {
            if let Some(stem) = path.as_ref().file_stem() {
                theme.name = Some(stem.to_string_lossy().to_string());
            }
        }
        
        Ok(theme)
    }

    /// Load all themes from a directory
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Vec<Theme>> {
        let mut themes = Vec::new();
        
        for entry in WalkDir::new(path.as_ref())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map_or(false, |ext| {
                    ext == "yaml" || ext == "yml"
                })
            })
        {
            match Self::load_from_file(entry.path()) {
                Ok(theme) => themes.push(theme),
                Err(e) => {
                    eprintln!("Warning: Failed to load theme from {}: {}", 
                             entry.path().display(), e);
                }
            }
        }
        
        Ok(themes)
    }

    /// Load bundled themes from the themes directory
    pub fn load_bundled() -> Result<Vec<Theme>> {
        let mut themes = Vec::new();
        
        // Get the themes directory relative to the current executable
        let themes_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .map(|mut p| {
                p.push("themes");
                p
            })
            .or_else(|| {
                // Fallback: look for themes directory in current working directory
                let mut cwd = std::env::current_dir().ok()?;
                cwd.push("themes");
                Some(cwd)
            });
        
        if let Some(themes_dir) = themes_dir {
            if themes_dir.exists() {
                // Load from all bundled categories
                for category in ["warp_bundled", "standard", "base16"] {
                    let category_path = themes_dir.join(category);
                    if category_path.exists() {
                        match Self::load_from_directory(&category_path) {
                            Ok(mut category_themes) => {
                                themes.append(&mut category_themes);
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to load themes from {}: {}", 
                                         category_path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(themes)
    }
}
