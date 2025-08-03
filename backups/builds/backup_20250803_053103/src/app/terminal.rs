use crate::executor::command_executor::{CommandExecutor, ExecutionResult};
use crate::executor::shell_integration::{ShellIntegration, ShellConfig};

use crate::input::KeyboardShortcuts;
use crate::keyset::KeysetManager;
use crate::model::block::BlockManager;
use crate::model::pane::{PaneManager, SplitDirection};
use crate::model::{
    history::HistoryManager,
    synchronization::{SynchronizationManager, SynchronizationScope},
    theme::AppTheme,
};
use crate::persistence::settings_manager::SettingsManager;
use crate::ui::block::view_block;
use crate::ui::command_palette::CommandPalette;
use crate::ui::command_search::CommandSearchPanel;
use crate::ui::command_history::CommandHistoryUI;
use crate::ui::enhanced_input::EnhancedInputState;
use crate::ui::input::enhanced_input_section;
use crate::ui::settings::{
    settings_view, CursorType, HistoryDedupMode, InputType, SettingsMessage, SettingsState,
    SettingsTab,
};
use crate::ui::synchronization::CompactIndicatorStyle;
use crate::ui::welcome::welcome_screen;
use crate::handlers::BatchCommandHandler;
// Using arboard as a maintained alternative to the clipboard crate
use arboard::Clipboard;
use iced::widget::container;
use iced::widget::{button, column, row, scrollable, text};
use iced::{executor, theme, Alignment, Application, Color, Command, Element, Length};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;
// use warp_themes::{iced_integration::ButtonVariant, Theme as WarpTheme};

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct WarpTerminal {
    block_manager: BlockManager,
    current_input: String,
    command_executor: CommandExecutor,
    executing_commands: HashMap<Uuid, tokio::task::JoinHandle<(Uuid, ExecutionResult)>>,
    scroll_position: scrollable::Id,
    // theme_manager: warp_themes::ThemeManager,
    theme: AppTheme,
    clipboard: Clipboard,
    show_settings: bool,
    settings_state: SettingsState,
    settings_errors: Vec<String>,
    settings_manager: SettingsManager,
    command_search_panel: CommandSearchPanel,
    history_manager: HistoryManager,
    synchronization_manager: SynchronizationManager,
    #[allow(dead_code)]
    keyset_manager: KeysetManager,
    #[allow(dead_code)]
    pane_manager: PaneManager,
    #[allow(dead_code)]
    keyboard_shortcuts: KeyboardShortcuts,
    command_palette: CommandPalette,
    command_history_ui: CommandHistoryUI,
    #[allow(dead_code)]
    shell_integration: ShellIntegration,
    #[allow(dead_code)]
    enhanced_input_state: EnhancedInputState,
    resizing_state: ResizingState,
    initial_mouse_position: Option<iced::Point>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizingState {
    Idle,
    Resizing {
        pane_id_1: Uuid,
        pane_id_2: Uuid,
    },
}

// Custom style for command palette overlay
struct OverlayStyle;

impl container::StyleSheet for OverlayStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.0, 0.0, 0.0, 0.5,
            ))),
            border: iced::Border::default(),
            text_color: None,
            shadow: iced::Shadow::default(),
        }
    }
}

impl fmt::Debug for WarpTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WarpTerminal")
            .field("block_manager", &self.block_manager)
            .field("current_input", &self.current_input)
            .field("command_executor", &self.command_executor)
            .field("executing_commands", &self.executing_commands)
            .field("scroll_position", &self.scroll_position)
            // field("theme_manager", &self.theme_manager)
            .field("theme", &self.theme)
            .field("clipboard", &"ClipboardContext")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    ExecuteCommand,
    CommandCompleted(Uuid, ExecutionResult),
    CommandCompletedWithHistory(Uuid, Uuid, ExecutionResult), // block_id, entry_id, result
    UpdateBlockPid(Uuid, u32),
    ChangeTheme(String),
    CopyCommand(Uuid),
    CopyOutput(Uuid),
    CopyBoth(Uuid),
    ShareBlock(Uuid),
    ReInputCommand(Uuid),
    BookmarkBlock(Uuid),
    // Command Search messages
    ToggleCommandSearch,
    CommandSearchQueryChanged(String),
    CommandSearchSetFilter(crate::ui::command_search::SearchFilter),
    CommandSearchSelectResult(usize),
    CommandSearchExecuteSelected,
    // Synchronized Inputs messages
    StartSynchronization(SynchronizationScope),
    StopSynchronization,
    ToggleSynchronizationMode,
    ToggleSynchronizationPause,
    SyncInputChanged(String),
    // Additional UI messages
    HandleEscape,
    CommandSearchNavigateUp,
    CommandSearchNavigateDown,
    // Pane Management messages
    PaneSplitHorizontal,
    PaneSplitVertical,
    PaneClose,
    PaneFocusNext,
    PaneFocusPrevious,
    PaneFocusDirection(Direction),

    // Pane Resizing
    PaneResize(Uuid, Uuid, i16),
    MouseMoved(iced::Point),
    PaneResizeStart(Uuid, Uuid, iced::Point),
    PaneResizeEnd,
    EventOccurred(iced::Event),
    // Settings messages
    SettingsLoaded((SettingsState, Option<crate::model::pane::SplitLayout>)),
    SettingsSaved(Result<(), String>),
    SettingsChanged(SettingsMessage),
    ImportSettings,
    SettingsImported(Option<PathBuf>),
    ApplyImportedSettings(Result<SettingsState, String>),
    ExportSettings,
    SettingsExported(Result<(), String>),
    AutoSaveSettings,

    // Command Palette messages
    CommandPaletteShow,
    CommandPaletteHide,
    CommandPaletteToggle,
    CommandPaletteQueryChanged(String),
    CommandPaletteNavigateUp,
    CommandPaletteNavigateDown,
    CommandPaletteExecuteSelected,
    CommandPaletteSelectResult(usize),
    CommandPaletteSetCategory(Option<crate::model::command_registry::CommandCategory>),
    CommandPaletteToggleFavorites,
    CommandPaletteToggleFavorite(String),
    
    // Modern Text Editor messages
    EditorMessage(crate::editor::text_editor::EditorMessage),
    
    // Batch Processing messages
    BatchProcessorExecute(String, String), // command_id, directory
    BatchProcessorPreview(String, String), // command_id, directory
    BatchProcessorCompleted(Result<String, String>),
    
    // Command History UI messages
    CommandHistoryToggle,
    CommandHistoryQueryChanged(String),
    CommandHistoryNavigateUp,
    CommandHistoryNavigateDown,
    CommandHistoryExecuteSelected,
    CommandHistorySelectResult(usize),
    CommandHistorySetViewMode(crate::ui::command_history::HistoryViewMode),
    CommandHistorySetFilter(crate::ui::command_history::SearchFilters),
    CommandHistoryBookmarkCommand(String),
    CommandHistoryTagCommand(String, String),
}

impl Application for WarpTerminal {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let settings_manager = SettingsManager::with_config("settings.json", 10, true, 3);
        let initial_settings_state = SettingsState::default();
        let initial_pane_manager = PaneManager::new();

        let app = Self {
            block_manager: BlockManager::new(),
            current_input: String::new(),
            command_executor: CommandExecutor::new(),
            executing_commands: HashMap::new(),
            scroll_position: scrollable::Id::unique(),
            theme: AppTheme::default(),
            clipboard: Clipboard::new().unwrap(),
            show_settings: false,
            settings_state: initial_settings_state,
            settings_errors: Vec::new(),
            settings_manager: settings_manager.clone(),
            command_search_panel: CommandSearchPanel::new(),
            history_manager: HistoryManager::new(),
            synchronization_manager: SynchronizationManager::new(),
            keyset_manager: {
                let mut manager = KeysetManager::new();
                manager.discover_keysets("keysets").unwrap();
                if let Err(e) = manager.load_keyset("default-warp-keybindings") {
                    tracing::warn!("Failed to load default keyset: {}", e);
                }
                manager
            },
            pane_manager: initial_pane_manager,
            keyboard_shortcuts: KeyboardShortcuts::default(),
            command_palette: CommandPalette::new(),
            command_history_ui: CommandHistoryUI::new(),
            shell_integration: ShellIntegration::new(ShellConfig::default()),
            enhanced_input_state: EnhancedInputState::new(),
            resizing_state: ResizingState::Idle,
            initial_mouse_position: None,
        };

        // Load settings on startup
        let load_settings_command = Command::perform(
            async move { settings_manager.load_settings().await },
            |(loaded_settings, loaded_layout)| Message::SettingsLoaded((loaded_settings, loaded_layout)),
        );

        (app, load_settings_command)
    }

    fn title(&self) -> String {
        "Warp Terminal".to_string()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone().into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SettingsLoaded((loaded_settings, loaded_layout)) => {
                tracing::info!("Settings loaded successfully on startup");
                self.settings_state = loaded_settings;
                if let Some(layout) = loaded_layout {
                    self.pane_manager.root_layout = layout;
                }
                self.settings_errors =
                    crate::ui::settings_handler::SettingsHandler::validate(&self.settings_state);
                if !self.settings_errors.is_empty() {
                    tracing::warn!("Settings validation errors: {:?}", self.settings_errors);
                }
                Command::none()
            }

            Message::InputChanged(input) => {
                if input == "toggle_settings" {
                    self.show_settings = !self.show_settings;
                    return Command::none();
                }
                if input == "toggle_command_palette" || input == "palette" {
                    self.command_palette.toggle_visibility();
                    return Command::none();
                }
                if input == "import_settings" {
                    return Command::perform(
                        async {
                            rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .pick_file()
                                .await
                                .map(|f| f.path().to_path_buf())
                        },
                        Message::SettingsImported,
                    );
                }
                if input == "export_settings" {
                    let state = self.settings_state.clone();
                    let manager = self.settings_manager.clone();
                    return Command::perform(
                        async move {
                            if let Some(file) = rfd::AsyncFileDialog::new()
                                .set_file_name("warp_settings_export.json")
                                .save_file()
                                .await
                            {
                                let path = file.path().to_path_buf();
                                manager
                                    .export_settings(&state, &path)
                                    .await
                                    .map_err(|e| e.to_string())
                            } else {
                                Err("Export cancelled by user".to_string())
                            }
                        },
                        Message::SettingsExported,
                    );
                }

                // Handle settings tab navigation
                if let Some(tab_name) = input.strip_prefix("tab_") {
                    match tab_name {
                        "Themes" => self.settings_state.active_tab = SettingsTab::Themes,
                        "Appearance" => self.settings_state.active_tab = SettingsTab::Appearance,
                        "Window" => self.settings_state.active_tab = SettingsTab::Window,
                        "Icon" => self.settings_state.active_tab = SettingsTab::Icon,
                        "Input" => self.settings_state.active_tab = SettingsTab::Input,
                        "History" => self.settings_state.active_tab = SettingsTab::History,
                        "Features" => self.settings_state.active_tab = SettingsTab::Features,
                        "Advanced" => self.settings_state.active_tab = SettingsTab::Advanced,
                        _ => {}
                    }
                    return Command::none();
                }

                // Handle specific settings changes
                match input.as_str() {
                    // Theme settings
                    "sync_os" => {
                        self.settings_state.sync_with_os = !self.settings_state.sync_with_os
                    }
                    "light_theme_picker" => { /* TODO: Open theme picker */ }
                    "dark_theme_picker" => { /* TODO: Open theme picker */ }
                    "create_theme" => { /* TODO: Open theme creator */ }

                    // Font and appearance settings
                    "font_family" => { /* TODO: Handle font family input */ }
                    "font_size" => { /* TODO: Handle font size slider */ }
                    "ligatures" => {
                        self.settings_state.show_ligatures = !self.settings_state.show_ligatures
                    }

                    // Cursor settings
                    "cursor_block" => self.settings_state.cursor_type = CursorType::Block,
                    "cursor_bar" => self.settings_state.cursor_type = CursorType::Bar,
                    "cursor_underline" => self.settings_state.cursor_type = CursorType::Underline,
                    "cursor_blink" => {
                        self.settings_state.cursor_blink = !self.settings_state.cursor_blink
                    }

                    // Window settings
                    "custom_window_size" => {
                        self.settings_state.open_new_windows_with_custom_size =
                            !self.settings_state.open_new_windows_with_custom_size
                    }
                    "window_columns" => { /* TODO: Handle window columns input */ }
                    "window_rows" => { /* TODO: Handle window rows input */ }
                    "window_opacity" => { /* TODO: Handle window opacity slider */ }
                    "window_blur" => { /* TODO: Handle window blur slider */ }

                    // Input settings
                    "input_universal" => self.settings_state.input_type = InputType::Universal,
                    "input_classic" => self.settings_state.input_type = InputType::Classic,

                    // Features
                    "autocomplete" => {
                        self.settings_state.enable_autocomplete =
                            !self.settings_state.enable_autocomplete
                    }
                    "ai_search" => {
                        self.settings_state.enable_ai_command_search =
                            !self.settings_state.enable_ai_command_search
                    }
                    "smart_suggestions" => {
                        self.settings_state.enable_smart_suggestions =
                            !self.settings_state.enable_smart_suggestions
                    }

                    // Advanced
                    "reset_defaults" => {
                        self.settings_state = SettingsState::default();
                        self.history_manager = HistoryManager::new();
                    }

                    // Icon settings
                    "icon_default" => { /* TODO: Handle icon selection */ }

                    // If no specific setting, it's a regular input
                    "history_enabled" => {
                        self.settings_state.history_enabled = !self.settings_state.history_enabled
                    }
                    "max_history_entries" => {
                        // Handle max history entries input
                    }
                    "dedup_none" => self.settings_state.history_dedup_mode = HistoryDedupMode::None,
                    "dedup_consecutive" => {
                        self.settings_state.history_dedup_mode = HistoryDedupMode::Consecutive
                    }
                    "dedup_global" => {
                        self.settings_state.history_dedup_mode = HistoryDedupMode::Global
                    }
                    "history_save_on_exit" => {
                        self.settings_state.history_save_on_exit =
                            !self.settings_state.history_save_on_exit
                    }
                    "history_sync_across_sessions" => {
                        self.settings_state.history_sync_across_sessions =
                            !self.settings_state.history_sync_across_sessions
                    }
                    "history_include_exit_codes" => {
                        self.settings_state.history_include_exit_codes =
                            !self.settings_state.history_include_exit_codes
                    }
                    "history_auto_bookmark_successful" => {
                        self.settings_state.history_auto_bookmark_successful =
                            !self.settings_state.history_auto_bookmark_successful
                    }
                    "history_search_fuzzy" => {
                        self.settings_state.history_search_fuzzy =
                            !self.settings_state.history_search_fuzzy
                    }
                    "add_exclude_pattern" => {
                        self.settings_state
                            .history_exclude_patterns
                            .push("new-pattern".to_string());
                    }
                    "clear_history" => {
                        self.history_manager = HistoryManager::new();
                    }
                    _ => self.current_input = input,
                }
                // Validate settings after any change
                self.settings_errors =
                    crate::ui::settings_handler::SettingsHandler::validate(&self.settings_state);
                // Trigger auto-save with debouncing
                let state = self.settings_state.clone();
                let manager = self.settings_manager.clone();
                let pane_layout = Some(self.pane_manager.root_layout.clone());
                return Command::perform(
                    async move {
                        manager.mark_settings_changed(&state, pane_layout).await;
                        Ok(())
                    },
                    Message::SettingsSaved,
                );
            }

            Message::SettingsSaved(result) => {
                match result {
                    Ok(()) => {
                        tracing::info!("Settings saved successfully");
                        self.settings_errors.clear();
                    }
                    Err(err) => {
                        tracing::error!("Failed to save settings: {}", err);
                        self.settings_errors.push(format!("Save failed: {}", err));
                    }
                }
                Command::none()
            }
            Message::SettingsImported(path) => {
                if let Some(path) = path {
                    let manager = self.settings_manager.clone();
                    return Command::perform(
                        async move {
                            manager
                                .import_settings(&path)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::ApplyImportedSettings,
                    );
                }
                Command::none()
            }
            Message::ApplyImportedSettings(result) => {
                match result {
                    Ok(mut new_state) => {
                        if let Some(layout) = new_state.pane_layout.take() {
                            self.pane_manager.root_layout = layout;
                        }
                        self.settings_state = new_state;
                        self.settings_errors.clear();
                        // Auto-save the imported settings
                        let state = self.settings_state.clone();
                        let manager = self.settings_manager.clone();
                        let pane_layout = Some(self.pane_manager.root_layout.clone());
                        return Command::perform(
                            async move {
                                manager
                                    .save_settings(&state, pane_layout)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            Message::SettingsSaved,
                        );
                    }
                    Err(e) => {
                        self.settings_errors.push(format!("Import failed: {}", e));
                    }
                }
                Command::none()
            }
            Message::SettingsExported(result) => {
                match result {
                    Ok(_) => {
                        self.settings_errors.clear();
                        self.settings_errors
                            .push("Settings exported successfully.".to_string());
                    }
                    Err(e) => {
                        self.settings_errors.push(format!("Export failed: {}", e));
                    }
                }
                Command::none()
            }

            Message::ExecuteCommand => {
                let mut command_text = self.current_input.trim().to_string();
                tracing::info!(command = %command_text, "Executing command");
                if command_text.is_empty() {
                    return Command::none();
                }

                self.current_input.clear();

                let is_background_command = command_text.ends_with("&");

                if is_background_command {
                    command_text.pop(); // Remove the '&'
                    command_text = command_text.trim().to_string();

                    let block = self.block_manager.add_background_block(Some(command_text.clone()), None);
                    let block_id = block.id;
                    let executor = self.command_executor.clone();

                    // Spawn a detached process for background commands
                    tokio::spawn(async move {
                        let result = executor.execute_command(&command_text).await;
                        // Send the PID to update the block immediately
                        let pid = result.pid;
                        let command_completed_message = Message::CommandCompleted(block_id, result);
                        if let Some(p) = pid {
                            // Send a message to update the block with the PID
                            return Message::UpdateBlockPid(block_id, p);
                        }
                        command_completed_message
                    });
                    Command::none()
                } else {
                    let block = self.block_manager.add_command(command_text.clone());

                    // Add to history manager
                    let entry_id = self.history_manager.add_command(command_text.clone());

                    // Also add to command search panel for backwards compatibility
                    self.command_search_panel
                        .add_to_history(command_text.clone());

                    let executor = self.command_executor.clone();
                    let _block_id = block.id;
                    let mut history_manager = self.history_manager.clone();

                    let handle = tokio::spawn(async move {
                        let result = executor.execute_command(&command_text).await;
                        // Update history with completion data
                        history_manager.update_command_completion(
                            entry_id,
                            result.exit_code,
                            result.execution_time,
                        );
                        (entry_id, result)
                    });

                    self.executing_commands.insert(block.id, handle);

                    Command::perform(
                        Self::wait_for_command_completion_with_history(
                            block.id,
                            self.executing_commands.remove(&block.id),
                        ),
                        |(id, entry_id, result)| {
                            Message::CommandCompletedWithHistory(id, entry_id, result)
                        },
                    )
                }
            }

            Message::CommandCompleted(block_id, result) => {
                tracing::info!(block_id = %block_id, "Command completed");
                self.block_manager
                    .update_block_output(block_id, result.stdout.clone());
                self.block_manager
                    .set_block_exit_code(block_id, result.exit_code);
                self.executing_commands.remove(&block_id);
                scrollable::snap_to(
                    self.scroll_position.clone(),
                    scrollable::RelativeOffset::END,
                )
            }

            Message::CommandCompletedWithHistory(block_id, entry_id, result) => {
                tracing::info!(block_id = %block_id, entry_id = %entry_id, "Command completed with history update");
                self.block_manager
                    .update_block_output(block_id, result.stdout.clone());
                self.block_manager
                    .set_block_exit_code(block_id, result.exit_code);

                // Update history manager (should already be done in the spawned task, but ensure consistency)
                self.history_manager.update_command_completion(
                    entry_id,
                    result.exit_code,
                    result.execution_time,
                );

                self.executing_commands.remove(&block_id);
                scrollable::snap_to(
                    self.scroll_position.clone(),
                    scrollable::RelativeOffset::END,
                )
            }

            Message::UpdateBlockPid(block_id, pid) => {
                if let Some(block) = self.block_manager.blocks_mut().iter_mut().find(|b| b.id == block_id) {
                    if let crate::model::block::BlockContent::Background { pid: block_pid, .. } = &mut block.content {
                        *block_pid = Some(pid);
                    }
                }
                Command::none()
            }

            Message::ChangeTheme(theme_name) => {
                self.change_theme(&theme_name);
                Command::none()
            }

            Message::CopyCommand(id) => {
                if let Some(block) = self.block_manager.blocks().iter().find(|b| b.id == id) {
                    let _ = self.clipboard.set_text(block.get_command_text());
                }
                Command::none()
            }

            Message::CopyOutput(id) => {
                if let Some(block) = self.block_manager.blocks().iter().find(|b| b.id == id) {
                    let _ = self.clipboard.set_text(block.get_output_text());
                }
                Command::none()
            }

            Message::CopyBoth(id) => {
                if let Some(block) = self.block_manager.blocks().iter().find(|b| b.id == id) {
                    let _ = self.clipboard.set_text(block.get_both_text());
                }
                Command::none()
            }

            Message::ShareBlock(id) => {
                if let Some(block) = self.block_manager.blocks().iter().find(|b| b.id == id) {
                    let share_text = format!(
                        "Check out this command I ran in Warp:\n\n```\n{}\n```",
                        block.get_both_text()
                    );
                    let _ = self.clipboard.set_text(share_text);
                }
                Command::none()
            }

            Message::ReInputCommand(id) => {
                if let Some(block) = self.block_manager.blocks().iter().find(|b| b.id == id) {
                    self.current_input = block.get_command_text();
                }
                Command::none()
            }

            Message::BookmarkBlock(id) => {
                self.block_manager.toggle_bookmark(id);
                Command::none()
            }

            // Command Search message handling
            Message::ToggleCommandSearch => {
                self.command_search_panel.toggle_visibility();
                Command::none()
            }

            Message::CommandSearchQueryChanged(query) => {
                // Check if we need enhanced history search before updating the query
                let use_enhanced_history = !query.is_empty()
                    && (self.command_search_panel.active_filter
                        == crate::ui::command_search::SearchFilter::History
                        || self.command_search_panel.active_filter
                            == crate::ui::command_search::SearchFilter::All);

                if use_enhanced_history {
                    let results = self
                        .command_search_panel
                        .search_history_enhanced(&query, &self.history_manager);
                    self.command_search_panel.update_query(query);
                    // Override results with enhanced history search for History filter
                    if self.command_search_panel.active_filter
                        == crate::ui::command_search::SearchFilter::History
                    {
                        self.command_search_panel.results = results;
                    }
                } else {
                    self.command_search_panel.update_query(query);
                }
                Command::none()
            }

            Message::CommandSearchSetFilter(filter) => {
                self.command_search_panel.set_filter(filter);
                Command::none()
            }

            Message::CommandSearchSelectResult(index) => {
                self.command_search_panel.selected_index = index;
                Command::none()
            }

            Message::CommandSearchExecuteSelected => {
                if let Some(result) = self.command_search_panel.get_selected_result() {
                    self.current_input = result.text.clone();
                    self.command_search_panel
                        .add_to_history(result.text.clone());
                    self.command_search_panel.toggle_visibility();
                }
                Command::none()
            }

            // Synchronized Inputs message handling
            Message::StartSynchronization(scope) => {
                self.synchronization_manager.start_synchronization(scope);
                Command::none()
            }

            Message::StopSynchronization => {
                self.synchronization_manager.stop_synchronization();
                Command::none()
            }

            Message::ToggleSynchronizationMode => {
                self.synchronization_manager.toggle_synchronization();
                Command::none()
            }

            Message::SyncInputChanged(input) => {
                let _targets = self.synchronization_manager.update_input(input);
                // In a real implementation, you would propagate the input to the target panes here
                // For now, we just update the synchronization manager state
                Command::none()
            }

            Message::ToggleSynchronizationPause => {
                self.synchronization_manager.toggle_pause();
                Command::none()
            }

            Message::HandleEscape => {
                // Close command search if open, otherwise do nothing
                if self.command_search_panel.is_visible {
                    self.command_search_panel.toggle_visibility();
                }
                Command::none()
            }

            Message::CommandSearchNavigateUp => {
                if self.command_search_panel.selected_index > 0 {
                    self.command_search_panel.selected_index -= 1;
                }
                Command::none()
            }

            Message::CommandSearchNavigateDown => {
                if self.command_search_panel.selected_index
                    < self.command_search_panel.results.len().saturating_sub(1)
                {
                    self.command_search_panel.selected_index += 1;
                }
                Command::none()
            }

            // Pane Management message handling
            Message::PaneSplitHorizontal => {
                if let Err(e) = self
                    .pane_manager
                    .split_current_pane(SplitDirection::Horizontal)
                {
                    tracing::warn!("Failed to split pane horizontally: {}", e);
                }
                Command::none()
            }

            Message::PaneSplitVertical => {
                if let Err(e) = self
                    .pane_manager
                    .split_current_pane(SplitDirection::Vertical)
                {
                    tracing::warn!("Failed to split pane vertically: {}", e);
                }
                Command::none()
            }

            Message::PaneClose => {
                if let Err(e) = self.pane_manager.close_current_pane() {
                    tracing::warn!("Failed to close pane: {}", e);
                }
                Command::none()
            }

            Message::PaneFocusNext => {
                self.pane_manager.focus_next_pane();
                Command::none()
            }

            Message::PaneFocusPrevious => {
                self.pane_manager.focus_previous_pane();
                Command::none()
            }

            Message::PaneFocusDirection(_direction) => {
                // TODO: Implement directional pane navigation
                // For now, just cycle to next pane
                self.pane_manager.focus_next_pane();
                Command::none()
            }

            Message::PaneResize(pane_id_1, pane_id_2, delta) => {
                self.pane_manager.resize_pane(pane_id_1, pane_id_2, delta);
                Command::none()
            }
            Message::MouseMoved(current_position) => {
                if let ResizingState::Resizing { pane_id_1, pane_id_2 } = self.resizing_state {
                    if let Some(initial_mouse_position) = self.initial_mouse_position {
                        let direction = self.pane_manager.root_layout.direction;
                        let delta = if direction == SplitDirection::Horizontal {
                            (current_position.x - initial_mouse_position.x) as i16
                        } else {
                            (current_position.y - initial_mouse_position.y) as i16
                        };
                        self.pane_manager.resize_pane(pane_id_1, pane_id_2, delta);
                        self.initial_mouse_position = Some(current_position);
                    }
                }
                Command::none()
            }
            Message::PaneResizeStart(pane_id_1, pane_id_2, initial_position) => {
                self.resizing_state = ResizingState::Resizing {
                    pane_id_1,
                    pane_id_2,
                };
                self.initial_mouse_position = Some(initial_position);
                Command::none()
            }
            Message::PaneResizeEnd => {
                self.resizing_state = ResizingState::Idle;
                self.initial_mouse_position = None;
                Command::none()
            }
            Message::EventOccurred(event) => {
                let active_pane_id = self.pane_manager.root_layout.active_pane;
                if let Some(message) = self.keyboard_shortcuts.handle_event(&event, active_pane_id, &self.pane_manager) {
                    return self.update(message);
                }
                Command::none()
            }

            Message::ImportSettings => {
                // This is handled by the "import_settings" string in InputChanged
                Command::none()
            }

            Message::ExportSettings => {
                // This is handled by the "export_settings" string in InputChanged
                Command::none()
            }

            Message::SettingsChanged(settings_msg) => {
                // Apply the settings change using SettingsHandler
                if let Err(e) = crate::ui::settings_handler::SettingsHandler::update(
                    &mut self.settings_state,
                    settings_msg,
                ) {
                    tracing::error!("Failed to apply settings change: {}", e);
                    self.settings_errors
                        .push(format!("Settings update failed: {}", e));
                    return Command::none();
                }

                // Validate settings after change
                self.settings_errors =
                    crate::ui::settings_handler::SettingsHandler::validate(&self.settings_state);

                // Trigger auto-save
                let state = self.settings_state.clone();
                let manager = self.settings_manager.clone();
                let pane_layout = Some(self.pane_manager.root_layout.clone());
                Command::perform(
                    async move {
                        manager.mark_settings_changed(&state, pane_layout).await;
                        Ok(())
                    },
                    Message::SettingsSaved,
                )
            }

            Message::AutoSaveSettings => {
                // Force save any pending changes
                let state = self.settings_state.clone();
                let manager = self.settings_manager.clone();
                let pane_layout = Some(self.pane_manager.root_layout.clone());
                Command::perform(
                    async move {
                        manager
                            .flush_pending_changes(&state, pane_layout)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::SettingsSaved,
                )
            }

            // Command Palette message handling
            Message::CommandPaletteShow => {
                self.command_palette.show();
                Command::none()
            }

            Message::CommandPaletteHide => {
                self.command_palette.hide();
                Command::none()
            }

            Message::CommandPaletteToggle => {
                self.command_palette.toggle_visibility();
                Command::none()
            }

            Message::CommandPaletteQueryChanged(query) => {
                self.command_palette.update_query(query);
                Command::none()
            }

            Message::CommandPaletteNavigateUp => {
                self.command_palette.navigate_up();
                Command::none()
            }

            Message::CommandPaletteNavigateDown => {
                self.command_palette.navigate_down();
                Command::none()
            }

            Message::CommandPaletteExecuteSelected => {
                if let Some(command_id) = self.command_palette.execute_selected() {
                    // Record execution in command registry
                    self.command_palette.command_registry.record_execution(&command_id);
                    
                    // Execute the selected command
                    if command_id.starts_with("workflow.") {
                        // Handle workflow execution
                        let workflow_name = command_id.strip_prefix("workflow.").unwrap_or("");
                        tracing::info!("Executing workflow: {}", workflow_name);
                        
                        // For now, just add the workflow command to input
                        // In a real implementation, you would load and execute the workflow
                        self.current_input = format!("# Workflow: {}\necho 'Workflow execution not yet implemented'", workflow_name);
                        return self.update(Message::ExecuteCommand);
                    } else {
                        // Handle built-in commands
                        match command_id.as_str() {
                            "pane.split.horizontal" => {
                                return self.update(Message::PaneSplitHorizontal)
                            }
                            "pane.split.vertical" => return self.update(Message::PaneSplitVertical),
                            "pane.close" => return self.update(Message::PaneClose),
                            "pane.focus.next" => return self.update(Message::PaneFocusNext),
                            "settings.open" => {
                                self.show_settings = true;
                                return Command::none();
                            }
                            "settings.export" => return self.update(Message::ExportSettings),
                            "settings.import" => return self.update(Message::ImportSettings),
                            "history.search" => return self.update(Message::ToggleCommandSearch),
                            "palette.toggle" => return self.update(Message::CommandPaletteToggle),
                            // Handle batch processor commands
                            cmd if cmd.starts_with("batch.") => {
                                let current_dir = std::env::current_dir()
                                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                                    .to_string_lossy()
                                    .to_string();
                                return self.update(Message::BatchProcessorExecute(command_id, current_dir));
                            }
                            _ => {
                                tracing::warn!("Unknown command executed: {}", command_id);
                            }
                        }
                    }
                }
                Command::none()
            }

            Message::CommandPaletteSelectResult(index) => {
                self.command_palette.selected_index = index;
                Command::none()
            }

            Message::CommandPaletteSetCategory(category) => {
                self.command_palette.set_category_filter(category);
                Command::none()
            }

            Message::CommandPaletteToggleFavorites => {
                self.command_palette.toggle_favorites_only();
                Command::none()
            }

            Message::CommandPaletteToggleFavorite(command_id) => {
                self.command_palette.toggle_favorite(&command_id);
                Command::none()
            }
            
            // Modern Text Editor message handling
            Message::EditorMessage(editor_msg) => {
                // Handle editor messages and update the current input based on editor content
                // For now, we'll just log the message and handle basic input changes
                tracing::debug!("Received editor message: {:?}", editor_msg);
                
                match editor_msg {
                    crate::editor::text_editor::EditorMessage::InputChanged(content) => {
                        self.current_input = content;
                    }
                    crate::editor::text_editor::EditorMessage::KeyPressed(key, modifiers) => {
                        // Handle special key combinations for the modern editor
                        use iced::keyboard::{key::Named, Modifiers};
                        
                        if let iced::keyboard::key::Key::Named(Named::Enter) = key {
                            if !modifiers.contains(Modifiers::SHIFT) {
                                // Execute command on Enter (without Shift)
                                return self.update(Message::ExecuteCommand);
                            }
                        }
                        // Other key combinations are handled by the editor itself
                    }
                    _ => {
                        // Other editor messages are handled internally by the editor
                        // We don't need to do anything special here
                    }
                }
                Command::none()
            }
            
            // Batch Processing message handling
            Message::BatchProcessorExecute(command_id, directory) => {
                tracing::info!("Executing batch processor command: {} in directory: {}", command_id, directory);
                let handler = BatchCommandHandler::new();
                Command::perform(
                    async move {
                        handler.handle_batch_command(&command_id, &directory).await
                            .map_err(|e| e.to_string())
                    },
                    Message::BatchProcessorCompleted,
                )
            }
            
            Message::BatchProcessorPreview(command_id, directory) => {
                tracing::info!("Previewing batch processor command: {} in directory: {}", command_id, directory);
                let handler = BatchCommandHandler::new();
                Command::perform(
                    async move {
                        handler.preview_batch_operation(&command_id, &directory).await
                            .map_err(|e| e.to_string())
                    },
                    Message::BatchProcessorCompleted,
                )
            }
            
            Message::BatchProcessorCompleted(result) => {
                match result {
                    Ok(message) => {
                        tracing::info!("Batch processor completed successfully: {}", message);
                        // Add a block to show the result
                        let block = self.block_manager.add_command(format!("batch_operation_result"));
                        let block_id = block.id;
                        self.block_manager.update_block_output(block_id, message);
                        self.block_manager.set_block_exit_code(block_id, 0);
                    }
                    Err(error) => {
                        tracing::error!("Batch processor failed: {}", error);
                        // Add a block to show the error
                        let block = self.block_manager.add_command(format!("batch_operation_error"));
                        let block_id = block.id;
                        self.block_manager.update_block_output(block_id, format!("Error: {}", error));
                        self.block_manager.set_block_exit_code(block_id, 1);
                    }
                }
                Command::none()
            }
            
            // Command History UI message handling
            Message::CommandHistoryToggle => {
                self.command_history_ui.toggle_visibility();
                Command::none()
            }
            
            Message::CommandHistoryQueryChanged(query) => {
                self.command_history_ui.update_search_query(query, &self.history_manager);
                Command::none()
            }
            
            Message::CommandHistoryNavigateUp => {
                self.command_history_ui.navigate_up();
                Command::none()
            }
            
            Message::CommandHistoryNavigateDown => {
                self.command_history_ui.navigate_down();
                Command::none()
            }
            
            Message::CommandHistoryExecuteSelected => {
                if let Some(command) = self.command_history_ui.get_selected_command() {
                    self.current_input = command;
                    self.command_history_ui.hide();
                }
                Command::none()
            }
            
            Message::CommandHistorySelectResult(index) => {
                self.command_history_ui.selected_index = index;
                Command::none()
            }
            
            Message::CommandHistorySetViewMode(view_mode) => {
                self.command_history_ui.set_view_mode(view_mode, &self.history_manager);
                Command::none()
            }
            
            Message::CommandHistorySetFilter(filter) => {
                self.command_history_ui.active_filters = filter;
                Command::none()
            }
            
            Message::CommandHistoryBookmarkCommand(_command) => {
                // TODO: Add bookmark functionality to history manager
                // if let Some(entry_id) = self.history_manager.find_entry_by_command(&command) {
                //     self.history_manager.bookmark_command(entry_id);
                // }
                Command::none()
            }
            
            Message::CommandHistoryTagCommand(_command, _tag) => {
                // TODO: Add tag functionality to history manager
                // if let Some(entry_id) = self.history_manager.find_entry_by_command(&command) {
                //     self.history_manager.tag_command(entry_id, tag);
                // }
                Command::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::event::listen().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Self::Message> {
        let font_size = self.settings_state.font_size;
        let font = iced::Font::DEFAULT;

        // Get synchronization status for the entire view method
        let sync_status = self.synchronization_manager.get_status();
        let sync_active = sync_status.is_active;
        let sync_indicator_text = if sync_active {
            format!(
                "{} {}",
                sync_status.short_indicator(),
                sync_status.target_pane_count
            )
        } else {
            "⚫".to_string()
        };
        let sync_status_color = if sync_active {
            Color::from_rgb(0.4, 0.8, 0.4)
        } else {
            Color::from_rgb(0.4, 0.4, 0.5)
        };

        // Header with settings, command search, command history, and synchronization controls
        let header_buttons = row![
            button(text("Settings").font(font).size(font_size))
                .on_press(Message::InputChanged("toggle_settings".into())),
            button(text("⚡ Search (Ctrl+R)").font(font).size(font_size))
                .on_press(Message::ToggleCommandSearch),
            button(text("📜 History").font(font).size(font_size))
                .on_press(Message::CommandHistoryToggle),
            button(
                text(sync_indicator_text)
                    .size(font_size - 2)
                    .style(sync_status_color)
            )
            .on_press(Message::ToggleSynchronizationMode)
            .style(theme::Button::Custom(Box::new(CompactIndicatorStyle {
                is_active: sync_active,
            }))),
        ]
        .spacing(8);

        if self.show_settings {
            return column![
                header_buttons,
                settings_view(&self.settings_state, &self.settings_errors)
            ]
            .align_items(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        // Show Command Palette if visible (overlay)
        if self.command_palette.is_visible {
            // Create proper main content for overlay
            let main_content = if self.block_manager.blocks().is_empty() {
                column![
                    header_buttons,
                    welcome_screen(font, font_size),
                    enhanced_input_section(
                        &self.current_input,
                        font,
                        font_size,
                        !self.executing_commands.is_empty(),
                    )
                ]
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)
            } else {
                let blocks = self.block_manager.blocks().iter().fold(
                    column![].spacing(8).padding(16),
                    |col, block| {
                        let block_widget = view_block(block, font, font_size);
                        col.push(block_widget)
                    },
                );
                let scrollable_blocks = scrollable(blocks)
                    .id(self.scroll_position.clone())
                    .height(Length::Fill);
                column![
                    header_buttons,
                    scrollable_blocks,
                    enhanced_input_section(
                        &self.current_input,
                        font,
                        font_size,
                        !self.executing_commands.is_empty(),
                    )
                ]
                .width(Length::Fill)
                .height(Length::Fill)
            };

            // Use column with overlay instead of Stack
            return column![
                container(main_content)
                    .width(Length::Fill)
                    .height(Length::Fill),
                container(self.command_palette.view(font, font_size))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::Container::Custom(Box::new(OverlayStyle)))
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        let input_section = enhanced_input_section(
            &self.current_input,
            font,
            font_size,
            !self.executing_commands.is_empty(),
        );

        // Create main content based on state
        let _main_content: Element<Message> = if self.block_manager.blocks().is_empty() {
            column![
                header_buttons,
                welcome_screen(font, font_size),
                input_section
            ]
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            let blocks: Element<_> = {
                let blocks_column = self.block_manager.blocks().iter().fold(
                    column![].spacing(8).padding(16),
                    |col, block| {
                        let block_widget = view_block(block, font, font_size);
                        col.push(block_widget)
                    },
                );
                blocks_column.into()
            };
            let scrollable_blocks = scrollable(blocks)
                .id(self.scroll_position.clone())
                .height(Length::Fill);
            column![header_buttons, scrollable_blocks, input_section]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        // Show Command History UI if toggled, otherwise show main content
        if self.command_history_ui.is_visible {
            self.command_history_ui.view(font, font_size)
        } else if self.command_search_panel.is_visible {
            self.command_search_panel.view(font, font_size)
        } else {
            _main_content
        }
    }
}

impl WarpTerminal {
    #[allow(dead_code)]
    async fn wait_for_command_completion(
        block_id: Uuid,
        handle: Option<tokio::task::JoinHandle<ExecutionResult>>,
    ) -> (Uuid, ExecutionResult) {
        if let Some(handle) = handle {
            match handle.await {
                Ok(result) => (block_id, result),
                Err(_) => (
                    block_id,
                    ExecutionResult {
                        stderr: "Command execution failed".to_string(),
                        exit_code: -1,
                        ..Default::default()
                    },
                ),
            }
        } else {
            (
                block_id,
                ExecutionResult {
                    stderr: "Command handle not found".to_string(),
                    exit_code: -1,
                    ..Default::default()
                },
            )
        }
    }

    async fn wait_for_command_completion_with_history(
        block_id: Uuid,
        handle: Option<tokio::task::JoinHandle<(Uuid, ExecutionResult)>>,
    ) -> (Uuid, Uuid, ExecutionResult) {
        if let Some(handle) = handle {
            match handle.await {
                Ok((entry_id, result)) => (block_id, entry_id, result),
                Err(_) => (
                    block_id,
                    Uuid::nil(),
                    ExecutionResult {
                        stderr: "Command execution failed".to_string(),
                        exit_code: -1,
                        ..Default::default()
                    },
                ),
            }
        } else {
            (
                block_id,
                Uuid::nil(),
                ExecutionResult {
                    stderr: "Command handle not found".to_string(),
                    exit_code: -1,
                    ..Default::default()
                },
            )
        }
    }

    fn change_theme(&mut self, _theme_name: &str) {
        // if let Some(theme_data) = self.theme_manager.get_theme(theme_name) {
        //     self.theme.load_theme(theme_data.clone());
        // }
    }

    #[allow(dead_code)]
    fn theme_selector(&self) -> Element<Message> {
        // let mut theme_buttons = row![].spacing(4);
        // for category in [
        //     warp_themes::ThemeCategory::Base16,
        //     warp_themes::ThemeCategory::Standard,
        //     warp_themes::ThemeCategory::WarpBundled,
        // ] {
        //     let themes = self.theme_manager.list_themes_by_category(category);
        //     if !themes.is_empty() {
        //         for theme in themes.iter().take(3) {
        //             let button = button(text(theme.display_name()))
        //                 .on_press(Message::ChangeTheme(theme.name.clone().unwrap_or_default()))
        //                 .style(iced::theme::Button::Custom(Box::new(
        //                     self.theme.button_style(ButtonVariant::Secondary),
        //                 )));
        //             theme_buttons = theme_buttons.push(button);
        //         }
        //     }
        // }
        // theme_buttons.into()
        text("Theme selector temporarily disabled").into()
    }
}
