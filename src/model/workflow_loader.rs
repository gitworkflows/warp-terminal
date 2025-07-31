//! Workflow Loader - Load and register workflows from YAML files into Command Palette
//!
//! This module provides functionality to dynamically load workflows from the 
//! workflows/specs directory and register them as commands in the Command Palette.

use crate::model::command_registry::{Command, CommandCategory, CommandRegistry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};
use warp_workflows_types::Workflow;

/// Workflow loader for integrating YAML workflows into Command Palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowLoader {
    /// Base directory containing workflow spec files
    pub specs_directory: String,
    /// Number of workflows loaded
    pub loaded_count: usize,
}

/// Workflow loading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStats {
    pub total_files_scanned: usize,
    pub workflows_loaded: usize,
    pub workflows_failed: usize,
    pub categories_found: Vec<String>,
}

impl WorkflowLoader {
    /// Create a new workflow loader
    pub fn new(specs_directory: impl Into<String>) -> Self {
        Self {
            specs_directory: specs_directory.into(),
            loaded_count: 0,
        }
    }

    /// Load all workflows from the specs directory into the command registry
    pub fn load_workflows(&mut self, registry: &mut CommandRegistry) -> Result<WorkflowStats, String> {
        let specs_path = Path::new(&self.specs_directory);
        
        if !specs_path.exists() {
            return Err(format!("Workflow specs directory '{}' does not exist", self.specs_directory));
        }

        info!("Loading workflows from: {}", self.specs_directory);

        let mut stats = WorkflowStats {
            total_files_scanned: 0,
            workflows_loaded: 0,
            workflows_failed: 0,
            categories_found: Vec::new(),
        };

        self.scan_directory_recursive(specs_path, registry, &mut stats)?;

        info!(
            "Workflow loading complete: {} loaded, {} failed, {} files scanned",
            stats.workflows_loaded, stats.workflows_failed, stats.total_files_scanned
        );

        self.loaded_count = stats.workflows_loaded;
        Ok(stats)
    }

    /// Recursively scan directory for workflow files
    fn scan_directory_recursive(
        &self,
        dir: &Path,
        registry: &mut CommandRegistry,
        stats: &mut WorkflowStats,
    ) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory_recursive(&path, registry, stats)?;
            } else if self.is_workflow_file(&path) {
                stats.total_files_scanned += 1;
                
                match self.load_workflow_file(&path) {
                    Ok(workflow) => {
                        let workflow_id = self.generate_workflow_id(&path, &workflow);
                        let category_name = self.extract_category_from_path(&path);
                        
                        if !stats.categories_found.contains(&category_name) {
                            stats.categories_found.push(category_name);
                        }

                        let command = self.workflow_to_command(&workflow_id, workflow)?;
                        
                        match registry.register_or_replace(command) {
                            _ => {
                                stats.workflows_loaded += 1;
                                debug!("Loaded workflow: {} ({})", workflow_id, path.display());
                            }
                        }
                    }
                    Err(e) => {
                        stats.workflows_failed += 1;
                        warn!("Failed to load workflow '{}': {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if file is a workflow file
    fn is_workflow_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            matches!(extension.to_str(), Some("yaml") | Some("yml"))
        } else {
            false
        }
    }

    /// Load a single workflow file
    fn load_workflow_file(&self, path: &Path) -> Result<Workflow, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;

        serde_yaml::from_str::<Workflow>(&content)
            .map_err(|e| format!("Failed to parse YAML in '{}': {}", path.display(), e))
    }

    /// Generate a unique workflow ID based on path and content
    fn generate_workflow_id(&self, path: &Path, _workflow: &Workflow) -> String {
        let category = self.extract_category_from_path(path);
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        // Clean filename for use as ID
        let clean_filename = filename
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>();

        format!("workflow.{}.{}", category, clean_filename)
    }

    /// Extract category from file path
    fn extract_category_from_path(&self, path: &Path) -> String {
        let specs_path = Path::new(&self.specs_directory);
        
        // Get relative path from specs directory
        if let Ok(relative_path) = path.strip_prefix(specs_path) {
            if let Some(first_component) = relative_path.components().next() {
                if let Some(category_str) = first_component.as_os_str().to_str() {
                    return category_str.to_lowercase();
                }
            }
        }

        "general".to_string()
    }

    /// Convert a Workflow to a Command
    fn workflow_to_command(&self, workflow_id: &str, workflow: Workflow) -> Result<Command, String> {
        // Extract keywords from tags and command
        let mut keywords = workflow.tags.clone();
        
        // Add some keywords based on command content
        let command_words: Vec<String> = workflow.command
            .split_whitespace()
            .take(5) // Limit to first 5 words
            .map(|s| s.to_lowercase())
            .filter(|s| s.len() > 2) // Only words longer than 2 chars
            .collect();
        keywords.extend(command_words);

        // Create description
        let description = if let Some(desc) = workflow.description {
            if desc.len() > 100 {
                format!("{}...", &desc[..97])
            } else {
                desc
            }
        } else {
            format!("Execute workflow: {}", workflow.name)
        };

        // Build command
        Command::builder(workflow_id, &workflow.name)
            .description(&description)
            .category(CommandCategory::Workflow)
            .keywords(keywords)
            .priority(90) // Slightly lower priority than built-ins
            .build()
    }

    /// Get workflow details by ID for execution
    pub fn get_workflow_by_id(&self, workflow_id: &str) -> Result<Workflow, String> {
        // Extract category and filename from workflow ID
        let parts: Vec<&str> = workflow_id.split('.').collect();
        if parts.len() != 3 || parts[0] != "workflow" {
            return Err(format!("Invalid workflow ID format: {}", workflow_id));
        }

        let category = parts[1];
        let filename = parts[2];
        
        // Construct potential file paths
        let possible_paths = vec![
            format!("{}/{}/{}.yaml", self.specs_directory, category, filename),
            format!("{}/{}/{}.yml", self.specs_directory, category, filename),
        ];

        for path_str in possible_paths {
            let path = Path::new(&path_str);
            if path.exists() {
                return self.load_workflow_file(path);
            }
        }

        Err(format!("Workflow file not found for ID: {}", workflow_id))
    }

    /// Reload workflows (useful for development)
    pub fn reload_workflows(&mut self, registry: &mut CommandRegistry) -> Result<WorkflowStats, String> {
        info!("Reloading workflows...");
        
        // Remove existing workflow commands
        let all_commands = registry.get_all_commands();
        for command in all_commands {
            if command.id.starts_with("workflow.") {
                registry.unregister(&command.id);
            }
        }

        // Reload workflows
        self.load_workflows(registry)
    }

    /// Get loading statistics
    pub fn get_stats(&self) -> (usize, String) {
        (self.loaded_count, self.specs_directory.clone())
    }
}

impl Default for WorkflowLoader {
    fn default() -> Self {
        Self::new("workflows/specs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_loader_creation() {
        let loader = WorkflowLoader::new("test/path");
        assert_eq!(loader.specs_directory, "test/path");
        assert_eq!(loader.loaded_count, 0);
    }

    #[test]
    fn test_is_workflow_file() {
        let loader = WorkflowLoader::default();
        
        assert!(loader.is_workflow_file(Path::new("test.yaml")));
        assert!(loader.is_workflow_file(Path::new("test.yml")));
        assert!(!loader.is_workflow_file(Path::new("test.txt")));
        assert!(!loader.is_workflow_file(Path::new("test")));
    }

    #[test]
    fn test_extract_category_from_path() {
        let loader = WorkflowLoader::new("workflows/specs");
        
        let path = Path::new("workflows/specs/git/clone.yaml");
        assert_eq!(loader.extract_category_from_path(path), "git");
        
        let path = Path::new("workflows/specs/docker/list_images.yaml");
        assert_eq!(loader.extract_category_from_path(path), "docker");
    }

    #[test]
    fn test_generate_workflow_id() {
        let loader = WorkflowLoader::new("workflows/specs");
        
        let workflow = Workflow {
            name: "Test Workflow".to_string(),
            command: "echo test".to_string(),
            tags: vec![],
            description: None,
            arguments: vec![],
            source_url: None,
            author: None,
            author_url: None,
            shells: vec![],
        };
        
        let path = Path::new("workflows/specs/git/clone-with-ssh.yaml");
        let id = loader.generate_workflow_id(path, &workflow);
        assert_eq!(id, "workflow.git.clone-with-ssh");
    }
}
