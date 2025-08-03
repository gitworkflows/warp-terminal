use super::settings::{SettingsState, SettingsMessage};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsProfile {
    Developer,
    Minimal,
    PowerUser,
}

#[derive(Default)]
pub struct SettingsHandler;

impl SettingsHandler {
    pub fn update(state: &mut SettingsState, message: SettingsMessage) -> Result<()> {
        match message {
            // Tab navigation
            SettingsMessage::TabChanged(tab) => state.active_tab = tab,

            // Theme settings
            SettingsMessage::SyncWithOSChanged(value) => state.sync_with_os = value,
            SettingsMessage::LightThemeChanged(theme) => state.current_light_theme = theme,
            SettingsMessage::DarkThemeChanged(theme) => state.current_dark_theme = theme,

            // Appearance settings
            SettingsMessage::FontFamilyChanged(family) => state.font_family = family,
            SettingsMessage::FontWeightChanged(weight) => state.font_weight = weight,
            SettingsMessage::FontSizeChanged(size) => {
                if size >= 8 && size <= 32 {
                    state.font_size = size;
                }
            }
            SettingsMessage::LineHeightChanged(height) => {
                if height >= 0.8 && height <= 2.5 {
                    state.line_height = height;
                }
            }
            SettingsMessage::ThinStrokesChanged(value) => state.use_thin_strokes = value,
            SettingsMessage::MinContrastChanged(value) => state.enforce_minimum_contrast = value,
            SettingsMessage::LigaturesChanged(value) => state.show_ligatures = value,
            SettingsMessage::CursorTypeChanged(cursor) => state.cursor_type = cursor,
            SettingsMessage::CursorBlinkChanged(value) => state.cursor_blink = value,

            // Window settings
            SettingsMessage::CustomWindowSizeChanged(value) => state.open_new_windows_with_custom_size = value,
            SettingsMessage::WindowColumnsChanged(cols_str) => {
                if let Ok(cols) = cols_str.parse::<u16>() {
                    if cols > 0 {
                        state.window_columns = cols;
                    }
                }
            }
            SettingsMessage::WindowRowsChanged(rows_str) => {
                if let Ok(rows) = rows_str.parse::<u16>() {
                    if rows > 0 {
                        state.window_rows = rows;
                    }
                }
            }
            SettingsMessage::WindowOpacityChanged(opacity) => {
                state.window_opacity = opacity.clamp(0.0, 100.0);
            }
            SettingsMessage::WindowBlurChanged(blur) => {
                state.window_blur_radius = blur.clamp(0, 5);
            }

            // Input settings
            SettingsMessage::InputTypeChanged(input_type) => state.input_type = input_type,

            // History settings
            SettingsMessage::HistoryEnabledChanged(value) => state.history_enabled = value,
            SettingsMessage::MaxHistoryEntriesChanged(val_str) => {
                if let Ok(val) = val_str.parse::<u32>() {
                    state.max_history_entries = val;
                }
            }
            SettingsMessage::HistoryDedupModeChanged(mode) => state.history_dedup_mode = mode,
            SettingsMessage::HistorySaveOnExitChanged(value) => state.history_save_on_exit = value,
            SettingsMessage::HistorySyncChanged(value) => state.history_sync_across_sessions = value,
            SettingsMessage::HistoryIncludeExitCodesChanged(value) => state.history_include_exit_codes = value,
            SettingsMessage::HistoryAutoBookmarkChanged(value) => state.history_auto_bookmark_successful = value,
            SettingsMessage::HistoryFuzzySearchChanged(value) => state.history_search_fuzzy = value,
            SettingsMessage::HistoryRetentionDaysChanged(days_str) => {
                if let Ok(days) = days_str.parse::<u32>() {
                    state.history_retention_days = days;
                }
            }
            SettingsMessage::AddExcludePattern => {
                state.history_exclude_patterns.push(String::new());
            }
            SettingsMessage::RemoveExcludePattern(index) => {
                if index < state.history_exclude_patterns.len() {
                    state.history_exclude_patterns.remove(index);
                }
            }
            SettingsMessage::ExcludePatternChanged(index, pattern) => {
                if let Some(p) = state.history_exclude_patterns.get_mut(index) {
                    *p = pattern;
                }
            }
            SettingsMessage::ClearHistory => {
                // Placeholder for actual history clearing logic
                println!("History cleared");
            }
            SettingsMessage::ExportHistory => {
                // Placeholder for actual history export logic
                println!("History exported");
            }

            // Feature settings
            SettingsMessage::AutocompleteChanged(value) => state.enable_autocomplete = value,
            SettingsMessage::AiCommandSearchChanged(value) => state.enable_ai_command_search = value,
            SettingsMessage::SmartSuggestionsChanged(value) => state.enable_smart_suggestions = value,

            // Layout settings
            SettingsMessage::AutoSaveLayoutChanged(value) => state.auto_save_layout = value,
            SettingsMessage::RestoreLayoutOnStartupChanged(value) => state.restore_layout_on_startup = value,
            SettingsMessage::LayoutAutosaveIntervalChanged(interval_str) => {
                if let Ok(interval) = interval_str.parse::<u32>() {
                    if interval >= 10 && interval <= 3600 { // 10 seconds to 1 hour
                        state.layout_autosave_interval = interval;
                    }
                }
            }
            SettingsMessage::SaveCurrentLayout => {
                // Placeholder for saving current layout
                println!("Saving current layout");
            }
            SettingsMessage::LoadLayout => {
                // Placeholder for loading layout
                println!("Loading layout");
            }
            SettingsMessage::ResetLayout => {
                // Reset the saved layout
                state.pane_layout = None;
                println!("Layout reset");
            }

            // Actions
            SettingsMessage::ResetToDefaults => *state = SettingsState::default(),
            SettingsMessage::CloseSettings => {
                // This should be handled by the main application loop
            }
        }
        Ok(())
    }

    pub fn validate(state: &SettingsState) -> Vec<String> {
        let mut errors = Vec::new();
        if state.font_size < 8 || state.font_size > 32 {
            errors.push("Font size must be between 8 and 32".to_string());
        }
        if state.line_height < 0.8 || state.line_height > 2.5 {
            errors.push("Line height must be between 0.8 and 2.5".to_string());
        }
        if state.window_columns == 0 {
            errors.push("Window columns must be greater than 0".to_string());
        }
        if state.window_rows == 0 {
            errors.push("Window rows must be greater than 0".to_string());
        }
        errors
    }

    pub fn serialize(state: &SettingsState) -> Result<String> {
        serde_json::to_string_pretty(state).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn deserialize(data: &str) -> Result<SettingsState> {
        serde_json::from_str(data).map_err(|e| anyhow::anyhow!(e))
    }
    
    pub fn get_profile_defaults(profile: SettingsProfile) -> SettingsState {
        use super::settings::{CursorType, HistoryDedupMode};
        
        let mut state = SettingsState::default();
        
        match profile {
            SettingsProfile::Developer => {
                state.font_family = "JetBrains Mono".to_string();
                state.font_size = 14;
                state.line_height = 1.4;
                state.show_ligatures = true;
                state.enable_autocomplete = true;
                state.enable_ai_command_search = true;
                state.enable_smart_suggestions = true;
                state.history_enabled = true;
                state.max_history_entries = 50000;
                state.history_dedup_mode = HistoryDedupMode::Global;
                state.history_search_fuzzy = true;
            }
            SettingsProfile::Minimal => {
                state.font_family = "SF Mono".to_string();
                state.font_size = 12;
                state.line_height = 1.2;
                state.show_ligatures = false;
                state.cursor_type = CursorType::Block;
                state.enable_autocomplete = false;
                state.enable_ai_command_search = false;
                state.enable_smart_suggestions = false;
                state.history_enabled = true;
                state.max_history_entries = 1000;
                state.history_dedup_mode = HistoryDedupMode::Consecutive;
            }
            SettingsProfile::PowerUser => {
                state.font_family = "Hack".to_string();
                state.font_size = 16;
                state.line_height = 1.3;
                state.show_ligatures = true;
                state.cursor_blink = false;
                state.enable_autocomplete = true;
                state.enable_ai_command_search = true;
                state.enable_smart_suggestions = true;
                state.history_enabled = true;
                state.max_history_entries = 100000;
                state.history_dedup_mode = HistoryDedupMode::Global;
                state.history_search_fuzzy = true;
                state.history_retention_days = 1000;
            }
        }
        
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_size_validation() {
        let mut state = SettingsState::default();
        
        // Valid update
        SettingsHandler::update(&mut state, SettingsMessage::FontSizeChanged(16)).unwrap();
        assert_eq!(state.font_size, 16);

        // Invalid update (below range)
        SettingsHandler::update(&mut state, SettingsMessage::FontSizeChanged(4)).unwrap();
        assert_eq!(state.font_size, 16); // Should not change

        // Invalid update (above range)
        SettingsHandler::update(&mut state, SettingsMessage::FontSizeChanged(40)).unwrap();
        assert_eq!(state.font_size, 16); // Should not change
    }

    #[test]
    fn test_validation_logic() {
        let mut state = SettingsState::default();
        assert!(SettingsHandler::validate(&state).is_empty());

        state.font_size = 5;
        let errors = SettingsHandler::validate(&state);
        assert!(!errors.is_empty());
        assert_eq!(errors[0], "Font size must be between 8 and 32");
    }

    #[test]
    fn test_serialization_deserialization() {
        let state = SettingsState::default();
        let serialized = SettingsHandler::serialize(&state).unwrap();
        let deserialized = SettingsHandler::deserialize(&serialized).unwrap();
        
        assert_eq!(state.font_size, deserialized.font_size);
        assert_eq!(state.active_tab, deserialized.active_tab);
        assert_eq!(state.sync_with_os, deserialized.sync_with_os);
    }
}
