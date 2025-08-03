//! Settings migration logic for Warp Terminal
//!
//! This module provides comprehensive settings migration functionality to handle
//! version upgrades, format changes, and backward compatibility.

use crate::persistence::settings_manager::{SettingsFile, SettingsFormat, SettingsMetadata};
use crate::ui::settings::SettingsState;
use anyhow::{Context, Result};
use chrono::Utc;

use tracing::{debug, info, warn};

/// The current settings version
pub const CURRENT_VERSION: &str = "1.0.0";

/// List of all supported version migrations
pub const SUPPORTED_VERSIONS: &[&str] = &["0.9.0", "0.9.5", "1.0.0"];

/// Migration step information
#[derive(Debug, Clone)]
pub struct MigrationStep {
    pub from_version: String,
    pub to_version: String,
    pub description: String,
    pub required: bool,
}

/// Get all available migration steps
pub fn get_migration_steps() -> Vec<MigrationStep> {
    vec![
        MigrationStep {
            from_version: "0.9.0".to_string(),
            to_version: "0.9.5".to_string(),
            description: "Add auto-save and format preferences".to_string(),
            required: false,
        },
        MigrationStep {
            from_version: "0.9.5".to_string(),
            to_version: "1.0.0".to_string(),
            description: "Enhanced metadata and validation".to_string(),
            required: true,
        },
    ]
}

/// Check if a version is supported for migration
pub fn is_version_supported(version: &str) -> bool {
    SUPPORTED_VERSIONS.contains(&version)
}

/// Get the migration path from one version to another
pub fn get_migration_path(from: &str, to: &str) -> Vec<MigrationStep> {
    let steps = get_migration_steps();
    let mut path = Vec::new();

    // Find the path from source to target version
    let mut current_version = from;
    while current_version != to {
        if let Some(step) = steps.iter().find(|s| s.from_version == current_version) {
            path.push(step.clone());
            current_version = &step.to_version;
        } else {
            // No migration path found
            break;
        }
    }

    path
}

/// Migrate a settings file from an older version to the current version
pub fn migrate_settings(settings_file: &mut SettingsFile) -> Result<()> {
    let from_version = &settings_file.metadata.version;

    if from_version == CURRENT_VERSION {
        debug!("Settings already at current version {}", CURRENT_VERSION);
        return Ok(());
    }

    if !is_version_supported(from_version) {
        return Err(anyhow::anyhow!(
            "Unsupported settings version: {}. Supported versions: {:?}",
            from_version,
            SUPPORTED_VERSIONS
        ));
    }

    info!(
        "Migrating settings from {} to {}",
        from_version, CURRENT_VERSION
    );

    let migration_path = get_migration_path(from_version, CURRENT_VERSION);
    if migration_path.is_empty() {
        return Err(anyhow::anyhow!(
            "No migration path found from {} to {}",
            from_version,
            CURRENT_VERSION
        ));
    }

    // Apply each migration step
    for step in migration_path {
        info!("Applying migration: {}", step.description);
        apply_migration_step(settings_file, &step)
            .with_context(|| format!("Failed to apply migration step: {}", step.description))?;
    }

    // Update final version and timestamp
    settings_file.metadata.version = CURRENT_VERSION.to_string();
    settings_file.metadata.last_modified = Utc::now();

    info!(
        "Successfully migrated settings to version {}",
        CURRENT_VERSION
    );
    Ok(())
}

/// Apply a specific migration step
fn apply_migration_step(settings_file: &mut SettingsFile, step: &MigrationStep) -> Result<()> {
    match (step.from_version.as_str(), step.to_version.as_str()) {
        ("0.9.0", "0.9.5") => migrate_0_9_0_to_0_9_5(settings_file),
        ("0.9.5", "1.0.0") => migrate_0_9_5_to_1_0_0(settings_file),
        _ => {
            warn!(
                "Unknown migration step: {} -> {}",
                step.from_version, step.to_version
            );
            Ok(())
        }
    }
}

/// Migration from 0.9.0 to 0.9.5: Add auto-save and format preferences
fn migrate_0_9_0_to_0_9_5(settings_file: &mut SettingsFile) -> Result<()> {
    debug!("Migrating from 0.9.0 to 0.9.5");

    // Add new metadata fields
    settings_file.metadata.auto_save_enabled = true;
    settings_file.metadata.format = SettingsFormat::Json;

    // Migrate any settings that need adjustment
    // For now, no settings changes are needed

    debug!("Successfully migrated to 0.9.5");
    Ok(())
}

/// Migration from 0.9.5 to 1.0.0: Enhanced metadata and validation
fn migrate_0_9_5_to_1_0_0(settings_file: &mut SettingsFile) -> Result<()> {
    debug!("Migrating from 0.9.5 to 1.0.0");

    // Validate and fix any invalid settings
    validate_and_fix_settings(&mut settings_file.settings)?;

    // Update backup count tracking
    settings_file.metadata.backup_count = 0;

    debug!("Successfully migrated to 1.0.0");
    Ok(())
}

/// Validate settings and apply fixes where possible
fn validate_and_fix_settings(settings: &mut SettingsState) -> Result<()> {
    // Fix font size range
    if settings.font_size < 8 {
        warn!(
            "Font size {} too small, setting to minimum 8",
            settings.font_size
        );
        settings.font_size = 8;
    } else if settings.font_size > 32 {
        warn!(
            "Font size {} too large, setting to maximum 32",
            settings.font_size
        );
        settings.font_size = 32;
    }

    // Fix line height range
    if settings.line_height < 0.8 {
        warn!(
            "Line height {} too small, setting to minimum 0.8",
            settings.line_height
        );
        settings.line_height = 0.8;
    } else if settings.line_height > 2.5 {
        warn!(
            "Line height {} too large, setting to maximum 2.5",
            settings.line_height
        );
        settings.line_height = 2.5;
    }

    // Fix window dimensions
    if settings.window_columns == 0 {
        warn!("Invalid window columns, setting to default 80");
        settings.window_columns = 80;
    }
    if settings.window_rows == 0 {
        warn!("Invalid window rows, setting to default 24");
        settings.window_rows = 24;
    }

    // Ensure window opacity is in valid range
    if settings.window_opacity < 0.0 {
        settings.window_opacity = 0.0;
    } else if settings.window_opacity > 100.0 {
        settings.window_opacity = 100.0;
    }

    // Validate history settings
    if settings.max_history_entries == 0 {
        warn!("Invalid max history entries, setting to default 10000");
        settings.max_history_entries = 10000;
    }

    // Clean up exclude patterns (remove empty ones)
    settings
        .history_exclude_patterns
        .retain(|p| !p.trim().is_empty());

    debug!("Settings validation and fixes applied");
    Ok(())
}

/// Migrate legacy settings (no metadata) to the latest format
pub fn migrate_legacy_settings(settings: SettingsState) -> SettingsFile {
    debug!("Migrating legacy settings without metadata");

    let mut settings = settings;

    // Apply validation and fixes to legacy settings
    if let Err(e) = validate_and_fix_settings(&mut settings) {
        warn!("Failed to validate legacy settings: {}", e);
    }

    let metadata = SettingsMetadata {
        version: CURRENT_VERSION.to_string(),
        created_at: Utc::now(),
        last_modified: Utc::now(),
        backup_count: 0,
        profile: None,
        auto_save_enabled: true,
        format: SettingsFormat::Json,
    };

    info!(
        "Successfully migrated legacy settings to version {}",
        CURRENT_VERSION
    );

    SettingsFile { metadata, settings }
}

/// Create a migration report showing what would be changed
pub fn create_migration_report(settings_file: &SettingsFile) -> MigrationReport {
    let from_version = &settings_file.metadata.version;
    let migration_path = get_migration_path(from_version, CURRENT_VERSION);

    MigrationReport {
        from_version: from_version.clone(),
        to_version: CURRENT_VERSION.to_string(),
        steps: migration_path,
        requires_migration: from_version != CURRENT_VERSION,
        is_supported: is_version_supported(from_version),
    }
}

/// Migration report structure
#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub from_version: String,
    pub to_version: String,
    pub steps: Vec<MigrationStep>,
    pub requires_migration: bool,
    pub is_supported: bool,
}

impl MigrationReport {
    /// Check if migration is possible
    pub fn can_migrate(&self) -> bool {
        self.is_supported && !self.steps.is_empty()
    }

    /// Get a human-readable description of the migration
    pub fn description(&self) -> String {
        if !self.requires_migration {
            return "No migration needed - settings are already up to date.".to_string();
        }

        if !self.is_supported {
            return format!("Migration not supported from version {}", self.from_version);
        }

        if self.steps.is_empty() {
            return format!(
                "No migration path available from {} to {}",
                self.from_version, self.to_version
            );
        }

        let step_descriptions: Vec<String> = self
            .steps
            .iter()
            .map(|s| {
                format!(
                    "• {} ({})",
                    s.description,
                    if s.required { "required" } else { "optional" }
                )
            })
            .collect();

        format!(
            "Migration from {} to {} will perform the following steps:\n{}",
            self.from_version,
            self.to_version,
            step_descriptions.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::settings::SettingsTab;

    #[test]
    fn test_version_support() {
        assert!(is_version_supported("1.0.0"));
        assert!(is_version_supported("0.9.0"));
        assert!(!is_version_supported("0.8.0"));
    }

    #[test]
    fn test_migration_path() {
        let path = get_migration_path("0.9.0", "1.0.0");
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].from_version, "0.9.0");
        assert_eq!(path[0].to_version, "0.9.5");
        assert_eq!(path[1].from_version, "0.9.5");
        assert_eq!(path[1].to_version, "1.0.0");
    }

    #[test]
    fn test_no_migration_needed() {
        let mut settings_file = SettingsFile {
            metadata: SettingsMetadata {
                version: CURRENT_VERSION.to_string(),
                ..Default::default()
            },
            settings: SettingsState::default(),
        };

        assert!(migrate_settings(&mut settings_file).is_ok());
    }

    #[test]
    fn test_legacy_migration() {
        let settings = SettingsState::default();
        let migrated = migrate_legacy_settings(settings);

        assert_eq!(migrated.metadata.version, CURRENT_VERSION);
        assert_eq!(migrated.metadata.format, SettingsFormat::Json);
        assert!(migrated.metadata.auto_save_enabled);
    }

    #[test]
    fn test_migration_report() {
        let settings_file = SettingsFile {
            metadata: SettingsMetadata {
                version: "0.9.0".to_string(),
                ..Default::default()
            },
            settings: SettingsState::default(),
        };

        let report = create_migration_report(&settings_file);
        assert!(report.requires_migration);
        assert!(report.can_migrate());
        assert_eq!(report.steps.len(), 2);
    }

    #[test]
    fn test_settings_validation() {
        let mut settings = SettingsState {
            font_size: 4,           // Invalid: too small
            line_height: 3.0,       // Invalid: too large
            window_columns: 0,      // Invalid: zero
            window_rows: 0,         // Invalid: zero
            max_history_entries: 0, // Invalid: zero
            ..Default::default()
        };

        validate_and_fix_settings(&mut settings).unwrap();

        assert_eq!(settings.font_size, 8);
        assert_eq!(settings.line_height, 2.5);
        assert_eq!(settings.window_columns, 80);
        assert_eq!(settings.window_rows, 24);
        assert_eq!(settings.max_history_entries, 10000);
    }
}
