use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::model::pane::SplitLayout;

/// Represents a saved layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLayout {
    /// Unique identifier for the layout
    pub id: String,
    /// Human-readable name for the layout
    pub name: String,
    /// The actual pane layout structure
    pub layout: SplitLayout,
    /// Timestamp when the layout was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the layout was last modified
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Optional description of the layout
    pub description: Option<String>,
    /// Tags for categorizing layouts
    pub tags: Vec<String>,
}

/// Manages the persistence of pane layouts
#[derive(Debug, Clone)]
pub struct LayoutPersistence {
    /// Directory path where layouts are stored
    storage_path: PathBuf,
    /// Cache of loaded layouts
    layouts: HashMap<String, SavedLayout>,
}

impl LayoutPersistence {
    /// Creates a new LayoutPersistence instance
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            layouts: HashMap::new(),
        }
    }

    /// Creates a new LayoutPersistence instance with default storage path
    pub fn with_default_path() -> Self {
        let storage_path = Self::default_storage_path();
        Self::new(storage_path)
    }

    /// Gets the default storage path for layouts
    pub fn default_storage_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("warp-terminal")
            .join("layouts")
    }

    /// Ensures the storage directory exists
    pub fn ensure_storage_dir(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)?;
        }
        Ok(())
    }

    /// Saves a layout to persistent storage
    pub fn save_layout(
        &mut self,
        name: String,
        layout: SplitLayout,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_storage_dir()?;

        let now = chrono::Utc::now();
        let id = format!("layout_{}", now.timestamp_millis());

        let saved_layout = SavedLayout {
            id: id.clone(),
            name: name.clone(),
            layout,
            created_at: now,
            modified_at: now,
            description,
            tags,
        };

        // Save to file
        let filename = format!("{}.json", id);
        let file_path = self.storage_path.join(filename);
        let json = serde_json::to_string_pretty(&saved_layout)?;
        fs::write(file_path, json)?;

        // Update cache
        self.layouts.insert(id.clone(), saved_layout);

        Ok(id)
    }

    /// Updates an existing layout
    pub fn update_layout(
        &mut self,
        id: &str,
        name: Option<String>,
        layout: Option<SplitLayout>,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut saved_layout = self.layouts.get(id)
            .ok_or_else(|| format!("Layout with id '{}' not found", id))?
            .clone();

        // Update fields if provided
        if let Some(name) = name {
            saved_layout.name = name;
        }
        if let Some(layout) = layout {
            saved_layout.layout = layout;
        }
        if let Some(description) = description {
            saved_layout.description = Some(description);
        }
        if let Some(tags) = tags {
            saved_layout.tags = tags;
        }
        saved_layout.modified_at = chrono::Utc::now();

        // Save to file
        let filename = format!("{}.json", id);
        let file_path = self.storage_path.join(filename);
        let json = serde_json::to_string_pretty(&saved_layout)?;
        fs::write(file_path, json)?;

        // Update cache
        self.layouts.insert(id.to_string(), saved_layout);

        Ok(())
    }

    /// Loads a layout by ID
    pub fn load_layout(&mut self, id: &str) -> Result<SplitLayout, Box<dyn std::error::Error>> {
        // Try cache first
        if let Some(saved_layout) = self.layouts.get(id) {
            return Ok(saved_layout.layout.clone());
        }

        // Load from file
        let filename = format!("{}.json", id);
        let file_path = self.storage_path.join(&filename);
        
        if !file_path.exists() {
            return Err(format!("Layout file '{}' not found", filename).into());
        }

        let json = fs::read_to_string(file_path)?;
        let saved_layout: SavedLayout = serde_json::from_str(&json)?;
        let layout = saved_layout.layout.clone();

        // Update cache
        self.layouts.insert(id.to_string(), saved_layout);

        Ok(layout)
    }

    /// Loads all available layouts
    pub fn load_all_layouts(&mut self) -> Result<Vec<SavedLayout>, Box<dyn std::error::Error>> {
        self.ensure_storage_dir()?;

        let mut layouts = Vec::new();

        // Read all JSON files in the storage directory
        for entry in fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match fs::read_to_string(&path) {
                    Ok(json) => {
                        match serde_json::from_str::<SavedLayout>(&json) {
                            Ok(saved_layout) => {
                                self.layouts.insert(saved_layout.id.clone(), saved_layout.clone());
                                layouts.push(saved_layout);
                            }
                            Err(e) => {
                                eprintln!("Error parsing layout file {:?}: {}", path, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading layout file {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by name
        layouts.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(layouts)
    }

    /// Deletes a layout by ID
    pub fn delete_layout(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("{}.json", id);
        let file_path = self.storage_path.join(filename);
        
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }

        self.layouts.remove(id);
        Ok(())
    }

    /// Gets all layout metadata (without loading the full layout data)
    pub fn get_layout_list(&mut self) -> Result<Vec<(String, String, chrono::DateTime<chrono::Utc>)>, Box<dyn std::error::Error>> {
        let layouts = self.load_all_layouts()?;
        Ok(layouts.into_iter()
            .map(|layout| (layout.id, layout.name, layout.modified_at))
            .collect())
    }

    /// Searches layouts by name or tag
    pub fn search_layouts(&mut self, query: &str) -> Result<Vec<SavedLayout>, Box<dyn std::error::Error>> {
        let all_layouts = self.load_all_layouts()?;
        let query_lower = query.to_lowercase();

        let filtered = all_layouts.into_iter().filter(|layout| {
            layout.name.to_lowercase().contains(&query_lower) ||
            layout.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower)) ||
            layout.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
        }).collect();

        Ok(filtered)
    }

    /// Exports a layout to a specific file path
    pub fn export_layout(&self, id: &str, export_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(saved_layout) = self.layouts.get(id) {
            let json = serde_json::to_string_pretty(saved_layout)?;
            fs::write(export_path, json)?;
            Ok(())
        } else {
            Err(format!("Layout with id '{}' not found", id).into())
        }
    }

    /// Imports a layout from a specific file path
    pub fn import_layout(&mut self, import_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(import_path)?;
        let mut saved_layout: SavedLayout = serde_json::from_str(&json)?;

        // Generate new ID and timestamps for imported layout
        let now = chrono::Utc::now();
        saved_layout.id = format!("layout_{}", now.timestamp_millis());
        saved_layout.created_at = now;
        saved_layout.modified_at = now;

        // Add "(imported)" suffix to name if not already present
        if !saved_layout.name.ends_with("(imported)") {
            saved_layout.name = format!("{} (imported)", saved_layout.name);
        }

        let id = saved_layout.id.clone();

        // Save to storage
        self.ensure_storage_dir()?;
        let filename = format!("{}.json", id);
        let file_path = self.storage_path.join(filename);
        let json = serde_json::to_string_pretty(&saved_layout)?;
        fs::write(file_path, json)?;

        // Update cache
        self.layouts.insert(id.clone(), saved_layout);

        Ok(id)
    }

    /// Gets the storage path
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }

    /// Clears the cache
    pub fn clear_cache(&mut self) {
        self.layouts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::pane::{PaneState, PaneId};
    use tempfile::TempDir;

    fn create_test_layout() -> SplitLayout {
        SplitLayout {
            pane_id: PaneId::new(),
            pane: Some(PaneState::new()),
            split_type: None,
            size_ratio: 1.0,
            children: Vec::new(),
            is_focused: true,
        }
    }

    #[test]
    fn test_save_and_load_layout() {
        let temp_dir = TempDir::new().unwrap();
        let mut persistence = LayoutPersistence::new(temp_dir.path().to_path_buf());
        
        let layout = create_test_layout();
        let layout_clone = layout.clone();

        let id = persistence.save_layout(
            "Test Layout".to_string(),
            layout,
            Some("A test layout".to_string()),
            vec!["test".to_string()],
        ).unwrap();

        let loaded_layout = persistence.load_layout(&id).unwrap();
        assert_eq!(loaded_layout.pane_id, layout_clone.pane_id);
    }

    #[test]
    fn test_load_all_layouts() {
        let temp_dir = TempDir::new().unwrap();
        let mut persistence = LayoutPersistence::new(temp_dir.path().to_path_buf());
        
        let layout1 = create_test_layout();
        let layout2 = create_test_layout();

        persistence.save_layout("Layout 1".to_string(), layout1, None, vec![]).unwrap();
        persistence.save_layout("Layout 2".to_string(), layout2, None, vec![]).unwrap();

        let all_layouts = persistence.load_all_layouts().unwrap();
        assert_eq!(all_layouts.len(), 2);
    }

    #[test]
    fn test_search_layouts() {
        let temp_dir = TempDir::new().unwrap();
        let mut persistence = LayoutPersistence::new(temp_dir.path().to_path_buf());
        
        let layout = create_test_layout();

        persistence.save_layout(
            "Development Layout".to_string(),
            layout,
            Some("For development work".to_string()),
            vec!["dev".to_string(), "coding".to_string()],
        ).unwrap();

        let results = persistence.search_layouts("dev").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Development Layout");
    }
}
