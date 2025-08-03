use crate::persistence::migration;
use crate::ui::settings::SettingsState;
use crate::ui::settings_handler::{SettingsHandler, SettingsProfile};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::{self, sync::RwLock, time::sleep};
use tracing::{debug, error, info, warn};
use crate::model::pane::SplitLayout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsMetadata {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub backup_count: u32,
    pub profile: Option<SettingsProfile>,
    pub auto_save_enabled: bool,
    pub format: SettingsFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettingsFormat {
    Json,
    Toml,
}

impl Default for SettingsMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            version: migration::CURRENT_VERSION.to_string(),
            created_at: now,
            last_modified: now,
            backup_count: 0,
            profile: None,
            auto_save_enabled: true,
            format: SettingsFormat::Json,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsFile {
    pub metadata: SettingsMetadata,
    pub settings: SettingsState,
}

#[derive(Clone)]
pub struct SettingsManager {
    settings_path: PathBuf,
    backup_dir: PathBuf,
    max_backups: usize,
    auto_save_enabled: bool,
    auto_save_delay: Duration,
    last_save_time: Arc<RwLock<Option<Instant>>>,
    pending_changes: Arc<RwLock<bool>>,
}

impl SettingsManager {
    /// Create a new settings manager with auto-save capabilities
    pub fn new(settings_path: &str) -> Self {
        let settings_path = PathBuf::from(settings_path);
        let backup_dir = settings_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("backups");

        Self {
            settings_path,
            backup_dir,
            max_backups: 10,
            auto_save_enabled: true,
            auto_save_delay: Duration::from_secs(3), // 3-second debounce
            last_save_time: Arc::new(RwLock::new(None)),
            pending_changes: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new settings manager with custom configuration
    pub fn with_config(
        settings_path: &str,
        max_backups: usize,
        auto_save_enabled: bool,
        auto_save_delay_secs: u64,
    ) -> Self {
        let settings_path = PathBuf::from(settings_path);
        let backup_dir = settings_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("backups");

        Self {
            settings_path,
            backup_dir,
            max_backups,
            auto_save_enabled,
            auto_save_delay: Duration::from_secs(auto_save_delay_secs),
            last_save_time: Arc::new(RwLock::new(None)),
            pending_changes: Arc::new(RwLock::new(false)),
        }
    }

    /// Create settings directory if it doesn't exist
    pub async fn ensure_settings_dir(&self) -> Result<()> {
        if let Some(parent) = self.settings_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create settings directory: {:?}", parent))?;
        }

        tokio::fs::create_dir_all(&self.backup_dir)
            .await
            .with_context(|| format!("Failed to create backup directory: {:?}", self.backup_dir))?;

        Ok(())
    }

    /// Load settings with comprehensive error handling, validation, and migration
    pub async fn load_settings(&self) -> (SettingsState, Option<SplitLayout>) {
        // Try to load settings file
        match self.load_settings_internal().await {
            Ok(mut settings_file) => {
                // Run migration if needed
                if let Err(e) = migration::migrate_settings(&mut settings_file) {
                    warn!("Settings migration failed: {}, trying backup", e);
                    let (backup_settings, backup_layout) = self.load_from_backup().await;
                    return backup_settings.map_or_else(
                        || {
                            warn!("Using default settings due to migration failure");
                            (SettingsState::default(), None)
                        },
                        |s| (s, backup_layout.clone()),
                    );
                }

                // Validate the loaded settings
                let validation_errors = SettingsHandler::validate(&settings_file.settings);
                if validation_errors.is_empty() {
                    info!(
                        "Settings loaded successfully (version: {})",
                        settings_file.metadata.version
                    );

                    // If migrated, save the updated version
                    if settings_file.metadata.version != migration::CURRENT_VERSION {
                        debug!("Settings migrated, saving updated version");
                        if let Err(e) = self.save_settings(&settings_file.settings, settings_file.settings.pane_layout.clone()).await {
                            warn!("Failed to save migrated settings: {}", e);
                        }
                    }

                    let layout = settings_file.settings.pane_layout.clone();
                    (settings_file.settings, layout)
                } else {
                    warn!("Settings validation failed: {:?}", validation_errors);
                    // Try to restore from backup
                    let (backup_settings, backup_layout) = self.load_from_backup().await;
                    backup_settings.map_or_else(
                        || {
                            warn!("Using default settings due to validation failures");
                            (SettingsState::default(), None)
                        },
                        |s| (s, backup_layout.clone()),
                    )
                }
            }
            Err(e) => {
                error!("Failed to load settings: {}", e);
                // Try to restore from backup
                let (backup_settings, backup_layout) = self.load_from_backup().await;
                backup_settings.map_or_else(
                    || {
                        info!("No valid backup found, using default settings");
                        (SettingsState::default(), None)
                    },
                    |s| (s, backup_layout.clone()),
                )
            }
        }
    }

    /// Internal method to load settings file with format detection
    async fn load_settings_internal(&self) -> Result<SettingsFile> {
        let content = tokio::fs::read_to_string(&self.settings_path)
            .await
            .with_context(|| format!("Failed to read settings file: {:?}", self.settings_path))?;

        // Try to parse as new format first (with metadata)
        if let Ok(settings_file) = serde_json::from_str::<SettingsFile>(&content) {
            debug!(
                "Loaded settings with metadata (version: {})",
                settings_file.metadata.version
            );
            return Ok(settings_file);
        }

        // Try TOML format
        if let Ok(settings_file) = toml::from_str::<SettingsFile>(&content) {
            debug!(
                "Loaded TOML settings with metadata (version: {})",
                settings_file.metadata.version
            );
            return Ok(settings_file);
        }

        // Fall back to legacy JSON format
        if let Ok(settings) = SettingsHandler::deserialize(&content) {
            info!("Loaded settings in legacy JSON format, will upgrade on next save");
            return Ok(migration::migrate_legacy_settings(settings));
        }

        // Try legacy TOML format
        if let Ok(settings) = toml::from_str::<SettingsState>(&content) {
            info!("Loaded settings in legacy TOML format, will upgrade on next save");
            return Ok(migration::migrate_legacy_settings(settings));
        }

        Err(anyhow::anyhow!(
            "Unable to parse settings file in any supported format"
        ))
    }

    /// Try to load settings from the most recent backup with format detection
    async fn load_from_backup(&self) -> (Option<SettingsState>, Option<SplitLayout>) {
        let backups_result = self.list_backups().await;
        let backups = match backups_result {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to list backups: {}", e);
                return (None, None);
            }
        };

        for backup_path in backups {
            if let Ok(content) = tokio::fs::read_to_string(&backup_path).await {
                // Try new format first
                if let Ok(mut settings_file) = serde_json::from_str::<SettingsFile>(&content) {
                    if migration::migrate_settings(&mut settings_file).is_ok() {
                        let validation_errors = SettingsHandler::validate(&settings_file.settings);
                        if validation_errors.is_empty() {
                            info!(
                                "Successfully loaded settings from JSON backup: {:?}",
                                backup_path
                            );
                            return (Some(settings_file.settings.clone()), settings_file.settings.pane_layout.clone());
                        }
                    }
                }

                // Try TOML format
                if let Ok(mut settings_file) = toml::from_str::<SettingsFile>(&content) {
                    if migration::migrate_settings(&mut settings_file).is_ok() {
                        let validation_errors = SettingsHandler::validate(&settings_file.settings);
                        if validation_errors.is_empty() {
                            info!(
                                "Successfully loaded settings from TOML backup: {:?}",
                                backup_path
                            );
                            return (Some(settings_file.settings.clone()), settings_file.settings.pane_layout.clone());
                        }
                    }
                }

                // Try legacy formats
                if let Ok(settings) = SettingsHandler::deserialize(&content) {
                    let validation_errors = SettingsHandler::validate(&settings);
                    if validation_errors.is_empty() {
                        info!(
                            "Successfully loaded legacy JSON settings from backup: {:?}",
                            backup_path
                        );
                        return (Some(settings.clone()), settings.pane_layout);
                    }
                }

                if let Ok(settings) = toml::from_str::<SettingsState>(&content) {
                    let validation_errors = SettingsHandler::validate(&settings);
                    if validation_errors.is_empty() {
                        info!(
                            "Successfully loaded legacy TOML settings from backup: {:?}",
                            backup_path
                        );
                        return (Some(settings.clone()), settings.pane_layout.clone());
                    }
                }
            }
            warn!("Backup file corrupt or invalid: {:?}", backup_path);
        }

        (None, None)
    }

    /// Save settings with backup, metadata, and format selection
    pub async fn save_settings(&self, state: &SettingsState, pane_layout: Option<SplitLayout>) -> Result<()> {
        self.ensure_settings_dir().await?;

        // Validate settings before saving
        let validation_errors = SettingsHandler::validate(state);
        if !validation_errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Settings validation failed: {:?}",
                validation_errors
            ));
        }

        // Create backup before saving
        self.create_timestamped_backup()
            .await
            .context("Failed to create backup before saving")?;

        // Load existing metadata or create new
        let mut metadata = self.load_metadata().await.unwrap_or_default();
        metadata.last_modified = chrono::Utc::now();
        metadata.version = migration::CURRENT_VERSION.to_string();

        let mut settings_state_to_save = state.clone();
        settings_state_to_save.pane_layout = pane_layout;

        let settings_file = SettingsFile {
            metadata: metadata.clone(),
            settings: settings_state_to_save,
        };

        // Serialize based on format preference
        let content = match metadata.format {
            SettingsFormat::Json => serde_json::to_string_pretty(&settings_file)
                .context("Failed to serialize settings as JSON")?,
            SettingsFormat::Toml => toml::to_string_pretty(&settings_file)
                .context("Failed to serialize settings as TOML")?,
        };

        // Write to temporary file first, then move to prevent corruption
        let temp_path = self.settings_path.with_extension("tmp");
        tokio::fs::write(&temp_path, content)
            .await
            .with_context(|| format!("Failed to write temporary settings file: {:?}", temp_path))?;

        tokio::fs::rename(&temp_path, &self.settings_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to move temporary file to settings path: {:?}",
                    self.settings_path
                )
            })?;

        // Update save tracking
        {
            let mut last_save = self.last_save_time.write().await;
            *last_save = Some(Instant::now());
        }
        {
            let mut pending = self.pending_changes.write().await;
            *pending = false;
        }

        // Clean up old backups
        self.cleanup_old_backups().await?;

        info!(
            "Settings saved successfully in {:?} format",
            metadata.format
        );
        Ok(())
    }

    /// Mark settings as changed and trigger auto-save if enabled
    pub async fn mark_settings_changed(&self, state: &SettingsState, pane_layout: Option<SplitLayout>) {
        if !self.auto_save_enabled {
            return;
        }

        {
            let mut pending = self.pending_changes.write().await;
            *pending = true;
        }

        // Clone necessary data for the async task
        let settings_manager = self.clone();
        let state = state.clone();
        let pane_layout = pane_layout.clone();

        // Spawn auto-save task with debouncing
        tokio::spawn(async move {
            sleep(settings_manager.auto_save_delay).await;

            let should_save = {
                let pending = settings_manager.pending_changes.read().await;
                *pending
            };

            if should_save {
                if let Err(e) = settings_manager.save_settings(&state, pane_layout).await {
                    error!("Auto-save failed: {}", e);
                } else {
                    debug!("Auto-save completed successfully");
                }
            }
        });
    }

    /// Load metadata from existing settings file with format detection
    async fn load_metadata(&self) -> Option<SettingsMetadata> {
        let content = tokio::fs::read_to_string(&self.settings_path).await.ok()?;

        // Try JSON first
        if let Ok(settings_file) = serde_json::from_str::<SettingsFile>(&content) {
            return Some(settings_file.metadata);
        }

        // Try TOML
        if let Ok(settings_file) = toml::from_str::<SettingsFile>(&content) {
            return Some(settings_file.metadata);
        }

        None
    }

    /// Create a timestamped backup
    pub async fn create_timestamped_backup(&self) -> Result<()> {
        if !self.settings_path.exists() {
            return Ok(());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("settings_{}.json", timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        tokio::fs::copy(&self.settings_path, &backup_path)
            .await
            .with_context(|| format!("Failed to create timestamped backup: {:?}", backup_path))?;

        info!("Created timestamped backup: {:?}", backup_path);
        Ok(())
    }

    /// List all backup files, sorted by creation time (newest first)
    pub async fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();

        if !self.backup_dir.exists() {
            return Ok(backups);
        }

        let mut dir = tokio::fs::read_dir(&self.backup_dir)
            .await
            .with_context(|| format!("Failed to read backup directory: {:?}", self.backup_dir))?;

        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().starts_with("settings_") {
                        backups.push(path);
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| {
            let a_metadata = fs::metadata(a)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            let b_metadata = fs::metadata(b)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            b_metadata.cmp(&a_metadata)
        });

        Ok(backups)
    }

    /// Clean up old backups, keeping only the most recent ones
    pub async fn cleanup_old_backups(&self) -> Result<()> {
        let backups = self.list_backups().await?;

        if backups.len() > self.max_backups {
            let to_remove = &backups[self.max_backups..];

            for backup_path in to_remove {
                if let Err(e) = tokio::fs::remove_file(backup_path).await {
                    warn!("Failed to remove old backup {:?}: {}", backup_path, e);
                } else {
                    info!("Removed old backup: {:?}", backup_path);
                }
            }
        }

        Ok(())
    }

    /// Restore settings from a specific backup
    pub async fn restore_from_backup(&self, backup_path: &Path) -> Result<SettingsState> {
        let content = tokio::fs::read_to_string(backup_path)
            .await
            .with_context(|| format!("Failed to read backup file: {:?}", backup_path))?;

        // Try new format first
        if let Ok(settings_file) = serde_json::from_str::<SettingsFile>(&content) {
            let validation_errors = SettingsHandler::validate(&settings_file.settings);
            if !validation_errors.is_empty() {
                return Err(anyhow::anyhow!(
                    "Backup settings validation failed: {:?}",
                    validation_errors
                ));
            }
            return Ok(settings_file.settings);
        }

        // Try legacy format
        let settings = SettingsHandler::deserialize(&content)
            .with_context(|| "Failed to parse backup file")?;

        let validation_errors = SettingsHandler::validate(&settings);
        if !validation_errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Backup settings validation failed: {:?}",
                validation_errors
            ));
        }

        Ok(settings)
    }

    /// Export settings to a specific file with metadata
    pub async fn export_settings(&self, state: &SettingsState, export_path: &Path) -> Result<()> {
        let metadata = SettingsMetadata {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            backup_count: 0,
            profile: None,
            auto_save_enabled: true,
            format: SettingsFormat::Json,
        };

        let settings_file = SettingsFile {
            metadata,
            settings: state.clone(),
        };

        let content = serde_json::to_string_pretty(&settings_file)
            .context("Failed to serialize settings for export")?;

        if let Some(parent) = export_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create export directory: {:?}", parent))?;
        }

        tokio::fs::write(export_path, content)
            .await
            .with_context(|| format!("Failed to write exported settings: {:?}", export_path))?;

        info!("Settings exported to: {:?}", export_path);
        Ok(())
    }

    /// Import settings from a file
    pub async fn import_settings(&self, import_path: &Path) -> Result<SettingsState> {
        let content = tokio::fs::read_to_string(import_path)
            .await
            .with_context(|| format!("Failed to read import file: {:?}", import_path))?;

        // Try new format first
        if let Ok(settings_file) = serde_json::from_str::<SettingsFile>(&content) {
            let validation_errors = SettingsHandler::validate(&settings_file.settings);
            if !validation_errors.is_empty() {
                return Err(anyhow::anyhow!(
                    "Imported settings validation failed: {:?}",
                    validation_errors
                ));
            }
            info!("Successfully imported settings from: {:?}", import_path);
            return Ok(settings_file.settings);
        }

        // Try legacy format
        let settings = SettingsHandler::deserialize(&content)
            .with_context(|| "Failed to parse import file")?;

        let validation_errors = SettingsHandler::validate(&settings);
        if !validation_errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Imported settings validation failed: {:?}",
                validation_errors
            ));
        }

        info!(
            "Successfully imported legacy settings from: {:?}",
            import_path
        );
        Ok(settings)
    }

    /// Load settings for a specific profile
    pub async fn load_profile_settings(&self, profile: SettingsProfile) -> SettingsState {
        SettingsHandler::get_profile_defaults(profile)
    }

    /// Get settings file path
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    /// Get backup directory path
    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    /// Set maximum number of backups to keep
    pub fn set_max_backups(&mut self, max_backups: usize) {
        self.max_backups = max_backups;
    }

    /// Enable or disable auto-save functionality
    pub fn set_auto_save_enabled(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }

    /// Set auto-save delay (debounce time)
    pub fn set_auto_save_delay(&mut self, delay: Duration) {
        self.auto_save_delay = delay;
    }

    /// Check if there are pending changes
    pub async fn has_pending_changes(&self) -> bool {
        let pending = self.pending_changes.read().await;
        *pending
    }

    /// Force save any pending changes
    pub async fn flush_pending_changes(&self, state: &SettingsState, pane_layout: Option<SplitLayout>) -> Result<()> {
        let has_pending = self.has_pending_changes().await;
        if has_pending {
            self.save_settings(state, pane_layout).await?;
        }
        Ok(())
    }

    /// Get current settings format preference
    pub async fn get_format_preference(&self) -> SettingsFormat {
        self.load_metadata()
            .await
            .map(|m| m.format)
            .unwrap_or(SettingsFormat::Json)
    }

    /// Set settings format preference (Json or Toml)
    pub async fn set_format_preference(
        &self,
        format: SettingsFormat,
        state: &SettingsState,
    ) -> Result<()> {
        let mut metadata = self.load_metadata().await.unwrap_or_default();
        metadata.format = format;
        metadata.last_modified = chrono::Utc::now();

        let settings_file = SettingsFile {
            metadata,
            settings: state.clone(),
        };

        // Save in the new format
        let content = match format {
            SettingsFormat::Json => serde_json::to_string_pretty(&settings_file)?,
            SettingsFormat::Toml => toml::to_string_pretty(&settings_file)?,
        };

        let temp_path = self.settings_path.with_extension("tmp");
        tokio::fs::write(&temp_path, content).await?;
        tokio::fs::rename(&temp_path, &self.settings_path).await?;

        info!("Settings format changed to {:?}", format);
        Ok(())
    }
}
