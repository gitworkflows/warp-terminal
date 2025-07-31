//! Comprehensive tests for the Settings Persistence System
//!
//! These tests verify all aspects of settings management including:
//! - Loading and saving settings
//! - Auto-save functionality
//! - Backup and restoration
//! - Migration between versions
//! - Import/export capabilities
//! - Error handling and recovery

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use warp_terminal::persistence::migration::{
    create_migration_report, migrate_legacy_settings, migrate_settings, CURRENT_VERSION,
};
use warp_terminal::persistence::settings_manager::{
    SettingsFile, SettingsFormat, SettingsManager, SettingsMetadata,
};
use warp_terminal::ui::settings::{
    CursorType, FontWeight, HistoryDedupMode, InputType, SettingsState, SettingsTab,
};
use warp_terminal::ui::settings_handler::{SettingsHandler, SettingsProfile};

/// Helper function to create a temporary settings manager for testing
fn create_test_manager(temp_dir: &TempDir) -> SettingsManager {
    let settings_path = temp_dir.path().join("test_settings.json");
    SettingsManager::with_config(
        settings_path.to_str().unwrap(),
        5,    // max_backups
        true, // auto_save_enabled
        1,    // auto_save_delay_secs
    )
}

/// Helper function to create test settings with specific values
fn create_test_settings() -> SettingsState {
    SettingsState {
        active_tab: SettingsTab::Appearance,
        sync_with_os: false,
        current_light_theme: "light-theme".to_string(),
        current_dark_theme: "dark-theme".to_string(),
        font_family: "JetBrains Mono".to_string(),
        font_weight: FontWeight::Normal,
        font_size: 16,
        line_height: 1.4,
        use_thin_strokes: true,
        enforce_minimum_contrast: false,
        show_ligatures: true,
        cursor_type: CursorType::Bar,
        cursor_blink: false,
        open_new_windows_with_custom_size: true,
        window_columns: 120,
        window_rows: 30,
        window_opacity: 95.0,
        window_blur_radius: 2,
        input_type: InputType::Universal,
        history_enabled: true,
        max_history_entries: 50000,
        history_dedup_mode: HistoryDedupMode::Global,
        history_save_on_exit: true,
        history_sync_across_sessions: true,
        history_exclude_patterns: vec!["secret*".to_string(), "*.key".to_string()],
        history_include_exit_codes: true,
        history_auto_bookmark_successful: false,
        history_retention_days: 90,
        history_search_fuzzy: true,
        enable_autocomplete: true,
        enable_ai_command_search: false,
        enable_smart_suggestions: true,
    }
}

#[tokio::test]
async fn test_settings_create_and_load_default() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);

    // Load settings when file doesn't exist should return defaults
    let loaded_settings = manager.load_settings().await;
    let default_settings = SettingsState::default();

    assert_eq!(loaded_settings.font_size, default_settings.font_size);
    assert_eq!(loaded_settings.cursor_type, default_settings.cursor_type);
    assert_eq!(
        loaded_settings.history_enabled,
        default_settings.history_enabled
    );

    println!("✅ Default settings loaded successfully when no file exists");
    Ok(())
}

#[tokio::test]
async fn test_settings_save_and_load_cycle() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let test_settings = create_test_settings();

    // Save test settings
    manager.save_settings(&test_settings).await?;
    assert!(
        manager.settings_path().exists(),
        "Settings file should exist after save"
    );

    // Load settings back
    let loaded_settings = manager.load_settings().await;

    // Verify key properties
    assert_eq!(loaded_settings.font_family, "JetBrains Mono");
    assert_eq!(loaded_settings.font_size, 16);
    assert_eq!(loaded_settings.cursor_type, CursorType::Bar);
    assert_eq!(loaded_settings.window_columns, 120);
    assert_eq!(loaded_settings.max_history_entries, 50000);
    assert_eq!(loaded_settings.history_exclude_patterns.len(), 2);

    println!("✅ Settings save/load cycle completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_settings_auto_save_functionality() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let mut test_settings = create_test_settings();

    // Mark settings as changed and wait for auto-save
    manager.mark_settings_changed(&test_settings).await;
    sleep(Duration::from_millis(1500)).await; // Wait for auto-save delay

    // Verify file was created
    assert!(
        manager.settings_path().exists(),
        "Settings file should exist after auto-save"
    );

    // Change settings again
    test_settings.font_size = 18;
    manager.mark_settings_changed(&test_settings).await;
    sleep(Duration::from_millis(1500)).await;

    // Load and verify changes
    let loaded_settings = manager.load_settings().await;
    assert_eq!(loaded_settings.font_size, 18);

    println!("✅ Auto-save functionality working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_backup_creation_and_restoration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let mut test_settings = create_test_settings();

    // Save initial settings
    manager.save_settings(&test_settings).await?;

    // Modify and save again (should create backup)
    test_settings.font_size = 20;
    manager.save_settings(&test_settings).await?;

    // Check that backup was created
    let backups = manager.list_backups().await?;
    assert!(!backups.is_empty(), "Backup should have been created");

    // Test restoration from backup
    let restored_settings = manager.restore_from_backup(&backups[0]).await?;
    assert_eq!(restored_settings.font_family, "JetBrains Mono");

    println!("✅ Backup creation and restoration working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_validation_and_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);

    // Create settings with invalid values
    let mut invalid_settings = create_test_settings();
    invalid_settings.font_size = 4; // Too small
    invalid_settings.line_height = 5.0; // Too large
    invalid_settings.window_columns = 0; // Invalid

    // Validation should catch these errors
    let validation_errors = SettingsHandler::validate(&invalid_settings);
    assert!(
        !validation_errors.is_empty(),
        "Validation should catch invalid settings"
    );
    assert!(validation_errors.iter().any(|e| e.contains("font size")));
    assert!(validation_errors.iter().any(|e| e.contains("line height")));
    assert!(validation_errors.iter().any(|e| e.contains("columns")));

    println!("✅ Settings validation working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_migration_system() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);

    // Test legacy settings migration
    let legacy_settings = create_test_settings();
    let migrated_file = migrate_legacy_settings(legacy_settings.clone());

    assert_eq!(migrated_file.metadata.version, CURRENT_VERSION);
    assert!(migrated_file.metadata.auto_save_enabled);
    assert_eq!(migrated_file.metadata.format, SettingsFormat::Json);
    assert_eq!(
        migrated_file.settings.font_family,
        legacy_settings.font_family
    );

    // Test migration report
    let mut old_settings_file = SettingsFile {
        metadata: SettingsMetadata {
            version: "0.9.0".to_string(),
            ..Default::default()
        },
        settings: create_test_settings(),
    };

    let migration_report = create_migration_report(&old_settings_file);
    assert!(migration_report.requires_migration);
    assert!(migration_report.can_migrate());
    assert!(!migration_report.steps.is_empty());

    // Apply migration
    migrate_settings(&mut old_settings_file)?;
    assert_eq!(old_settings_file.metadata.version, CURRENT_VERSION);

    println!("✅ Settings migration system working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_import_export() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let test_settings = create_test_settings();

    // Export settings
    let export_path = temp_dir.path().join("exported_settings.json");
    manager
        .export_settings(&test_settings, &export_path)
        .await?;
    assert!(export_path.exists(), "Export file should exist");

    // Import settings
    let imported_settings = manager.import_settings(&export_path).await?;

    // Verify imported settings match original
    assert_eq!(imported_settings.font_family, test_settings.font_family);
    assert_eq!(imported_settings.font_size, test_settings.font_size);
    assert_eq!(imported_settings.cursor_type, test_settings.cursor_type);
    assert_eq!(
        imported_settings.history_exclude_patterns,
        test_settings.history_exclude_patterns
    );

    println!("✅ Settings import/export working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_format_switching() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let test_settings = create_test_settings();

    // Save as JSON first
    manager.save_settings(&test_settings).await?;
    let json_format = manager.get_format_preference().await;
    assert_eq!(json_format, SettingsFormat::Json);

    // Switch to TOML format
    manager
        .set_format_preference(SettingsFormat::Toml, &test_settings)
        .await?;
    let toml_format = manager.get_format_preference().await;
    assert_eq!(toml_format, SettingsFormat::Toml);

    // Verify settings can still be loaded after format change
    let loaded_settings = manager.load_settings().await;
    assert_eq!(loaded_settings.font_family, test_settings.font_family);

    println!("✅ Settings format switching working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_profile_defaults() -> Result<()> {
    // Test different profile defaults
    let developer_settings = SettingsHandler::get_profile_defaults(SettingsProfile::Developer);
    assert_eq!(developer_settings.font_family, "JetBrains Mono");
    assert_eq!(developer_settings.max_history_entries, 50000);
    assert!(developer_settings.enable_autocomplete);
    assert!(developer_settings.history_search_fuzzy);

    let minimal_settings = SettingsHandler::get_profile_defaults(SettingsProfile::Minimal);
    assert_eq!(minimal_settings.font_family, "SF Mono");
    assert_eq!(minimal_settings.max_history_entries, 1000);
    assert!(!minimal_settings.enable_autocomplete);
    assert_eq!(minimal_settings.cursor_type, CursorType::Block);

    let power_user_settings = SettingsHandler::get_profile_defaults(SettingsProfile::PowerUser);
    assert_eq!(power_user_settings.font_family, "Hack");
    assert_eq!(power_user_settings.max_history_entries, 100000);
    assert_eq!(power_user_settings.history_retention_days, 1000);

    println!("✅ Settings profile defaults working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_concurrent_access() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let test_settings = create_test_settings();

    // Save initial settings
    manager.save_settings(&test_settings).await?;

    // Simulate concurrent access
    let manager1 = manager.clone();
    let manager2 = manager.clone();
    let settings1 = test_settings.clone();
    let mut settings2 = test_settings.clone();
    settings2.font_size = 18;

    // Concurrent operations
    let task1 = tokio::spawn(async move {
        manager1.mark_settings_changed(&settings1).await;
        sleep(Duration::from_millis(500)).await;
    });

    let task2 = tokio::spawn(async move {
        manager2.mark_settings_changed(&settings2).await;
        sleep(Duration::from_millis(500)).await;
    });

    // Wait for both tasks
    task1.await?;
    task2.await?;

    // Wait for auto-save to complete
    sleep(Duration::from_millis(2000)).await;

    // Verify final state
    let final_settings = manager.load_settings().await;
    assert!(final_settings.font_size == 16 || final_settings.font_size == 18);

    println!("✅ Concurrent settings access handled correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_backup_cleanup() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut manager = create_test_manager(&temp_dir);
    manager.set_max_backups(3); // Limit to 3 backups

    let mut test_settings = create_test_settings();

    // Create multiple backups by saving repeatedly
    for i in 0..6 {
        test_settings.font_size = 12 + i;
        manager.save_settings(&test_settings).await?;
        sleep(Duration::from_millis(100)).await; // Ensure different timestamps
    }

    // Check that only 3 backups remain
    let backups = manager.list_backups().await?;
    assert!(
        backups.len() <= 3,
        "Should keep only the maximum number of backups"
    );

    println!("✅ Backup cleanup working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_error_recovery() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);

    // Create a corrupted settings file
    let corrupted_content = "{ invalid json content !!!";
    fs::write(manager.settings_path(), corrupted_content)?;

    // Load settings should fall back to defaults
    let loaded_settings = manager.load_settings().await;
    let default_settings = SettingsState::default();
    assert_eq!(loaded_settings.font_size, default_settings.font_size);

    // Save valid settings should work and create backup of corrupted file
    let test_settings = create_test_settings();
    manager.save_settings(&test_settings).await?;

    // Verify settings are now loadable
    let recovered_settings = manager.load_settings().await;
    assert_eq!(recovered_settings.font_family, "JetBrains Mono");

    println!("✅ Settings error recovery working correctly");
    Ok(())
}

#[tokio::test]
async fn test_settings_pending_changes_tracking() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let test_settings = create_test_settings();

    // Initially no pending changes
    assert!(!manager.has_pending_changes().await);

    // Mark changes and check
    manager.mark_settings_changed(&test_settings).await;
    // Give a moment for the async task to set pending flag
    sleep(Duration::from_millis(100)).await;

    // Force flush pending changes
    manager.flush_pending_changes(&test_settings).await?;

    // After flush, should have no pending changes
    assert!(!manager.has_pending_changes().await);

    println!("✅ Pending changes tracking working correctly");
    Ok(())
}

/// Integration test demonstrating the complete settings lifecycle
#[tokio::test]
async fn test_complete_settings_lifecycle() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);

    println!("🚀 Testing complete settings lifecycle...");

    // 1. Start with defaults
    let mut settings = manager.load_settings().await;
    println!("   📁 Loaded default settings");

    // 2. Apply developer profile
    settings = SettingsHandler::get_profile_defaults(SettingsProfile::Developer);
    manager.save_settings(&settings).await?;
    println!("   👨‍💻 Applied developer profile");

    // 3. Customize some settings
    settings.font_size = 18;
    settings.window_columns = 100;
    settings
        .history_exclude_patterns
        .push("*.secret".to_string());
    manager.mark_settings_changed(&settings).await;
    sleep(Duration::from_millis(2000)).await; // Wait for auto-save
    println!("   ✏️  Applied custom changes with auto-save");

    // 4. Export settings
    let export_path = temp_dir.path().join("lifecycle_export.json");
    manager.export_settings(&settings, &export_path).await?;
    println!("   📤 Exported settings");

    // 5. Reset to minimal profile
    settings = SettingsHandler::get_profile_defaults(SettingsProfile::Minimal);
    manager.save_settings(&settings).await?;
    println!("   🔧 Switched to minimal profile");

    // 6. Import previous settings
    let imported_settings = manager.import_settings(&export_path).await?;
    manager.save_settings(&imported_settings).await?;
    println!("   📥 Re-imported previous settings");

    // 7. Verify final state
    let final_settings = manager.load_settings().await;
    assert_eq!(final_settings.font_size, 18);
    assert_eq!(final_settings.window_columns, 100);
    assert!(final_settings
        .history_exclude_patterns
        .contains(&"*.secret".to_string()));

    // 8. Check backups were created
    let backups = manager.list_backups().await?;
    assert!(
        !backups.is_empty(),
        "Backups should have been created during lifecycle"
    );

    println!("   ✅ Verified final state matches expectations");
    println!("🎉 Complete settings lifecycle test passed!");

    Ok(())
}

/// Performance test for settings operations
#[tokio::test]
async fn test_settings_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let manager = create_test_manager(&temp_dir);
    let test_settings = create_test_settings();

    let start = std::time::Instant::now();

    // Perform multiple save/load cycles
    for i in 0..100 {
        let mut settings = test_settings.clone();
        settings.font_size = 12 + (i % 20);
        manager.save_settings(&settings).await?;
        let _loaded = manager.load_settings().await;
    }

    let elapsed = start.elapsed();
    println!("⚡ 100 save/load cycles completed in {:?}", elapsed);

    // Performance should be reasonable (less than 5 seconds for 100 cycles)
    assert!(
        elapsed.as_secs() < 5,
        "Performance test failed: operations took too long"
    );

    println!("✅ Settings performance test passed");
    Ok(())
}
