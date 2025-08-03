use iced::widget::{
    button, column, row, text, slider, checkbox, radio, text_input, container, 
    scrollable, Space
};
use iced::{Alignment, Element, Length, alignment};
use crate::Message;
use iced::theme;
use serde::{Deserialize, Serialize};
use crate::model::pane::SplitLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Normal,
    Bold,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorType {
    Bar,
    Block,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    Universal,
    Classic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryDedupMode {
    None,
    Consecutive,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsTab {
    Themes,
    Appearance,
    Window,
    Icon,
    Input,
    History,
    Features,
    Advanced,
}

impl std::fmt::Display for SettingsTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsTab::Themes => write!(f, "Themes"),
            SettingsTab::Appearance => write!(f, "Appearance"),
            SettingsTab::Window => write!(f, "Window"),
            SettingsTab::Icon => write!(f, "Icon"),
            SettingsTab::Input => write!(f, "Input"),
            SettingsTab::History => write!(f, "History"),
            SettingsTab::Features => write!(f, "Features"),
            SettingsTab::Advanced => write!(f, "Advanced"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    // Current active tab
    pub active_tab: SettingsTab,
    
    // Themes
    pub sync_with_os: bool,
    pub current_light_theme: String,
    pub current_dark_theme: String,
    
    // Appearance 
    pub font_family: String,
    pub font_weight: FontWeight,
    pub font_size: u16,
    pub line_height: f32,
    pub use_thin_strokes: bool,
    pub enforce_minimum_contrast: bool,
    pub show_ligatures: bool,
    pub cursor_type: CursorType,
    pub cursor_blink: bool,
    
    // Window
    pub open_new_windows_with_custom_size: bool,
    pub window_columns: u16,
    pub window_rows: u16,
    pub window_opacity: f32,
    pub window_blur_radius: u8,
    
    // Input
    pub input_type: InputType,
    
    // History
    pub history_enabled: bool,
    pub max_history_entries: u32,
    pub history_dedup_mode: HistoryDedupMode,
    pub history_save_on_exit: bool,
    pub history_sync_across_sessions: bool,
    pub history_exclude_patterns: Vec<String>,
    pub history_include_exit_codes: bool,
    pub history_auto_bookmark_successful: bool,
    pub history_retention_days: u32,
    pub history_search_fuzzy: bool,
    
    // Features
    pub enable_autocomplete: bool,
    pub enable_ai_command_search: bool,
    pub enable_smart_suggestions: bool,
    
    // Layout Management
    pub pane_layout: Option<SplitLayout>,
    pub auto_save_layout: bool,
    pub restore_layout_on_startup: bool,
    pub layout_autosave_interval: u32, // seconds
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    TabChanged(SettingsTab),
    // Theme settings
    SyncWithOSChanged(bool),
    LightThemeChanged(String),
    DarkThemeChanged(String),
    // Appearance settings
    FontFamilyChanged(String),
    FontWeightChanged(FontWeight),
    FontSizeChanged(u16),
    LineHeightChanged(f32),
    ThinStrokesChanged(bool),
    MinContrastChanged(bool),
    LigaturesChanged(bool),
    CursorTypeChanged(CursorType),
    CursorBlinkChanged(bool),
    // Window settings
    CustomWindowSizeChanged(bool),
    WindowColumnsChanged(String), // Changed to String for validation
    WindowRowsChanged(String),    // Changed to String for validation
    WindowOpacityChanged(f32),
    WindowBlurChanged(u8),
    // Input settings
    InputTypeChanged(InputType),
    // History settings
    HistoryEnabledChanged(bool),
    MaxHistoryEntriesChanged(String),
    HistoryDedupModeChanged(HistoryDedupMode),
    HistorySaveOnExitChanged(bool),
    HistorySyncChanged(bool),
    HistoryIncludeExitCodesChanged(bool),
    HistoryAutoBookmarkChanged(bool),
    HistoryFuzzySearchChanged(bool),
    HistoryRetentionDaysChanged(String),
    AddExcludePattern,
    RemoveExcludePattern(usize),
    ExcludePatternChanged(usize, String),
    ClearHistory,
    ExportHistory,
    // Feature settings
    AutocompleteChanged(bool),
    AiCommandSearchChanged(bool),
    SmartSuggestionsChanged(bool),
    // Layout settings
    AutoSaveLayoutChanged(bool),
    RestoreLayoutOnStartupChanged(bool),
    LayoutAutosaveIntervalChanged(String),
    SaveCurrentLayout,
    LoadLayout,
    ResetLayout,
    // Actions
    ResetToDefaults,
    CloseSettings,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_tab: SettingsTab::Themes,
            
            // Themes
            sync_with_os: true,
            current_light_theme: "Light".to_string(),
            current_dark_theme: "Dark".to_string(),
            
            // Appearance
            font_family: "Hack".to_string(),
            font_weight: FontWeight::Normal,
            font_size: 16,
            line_height: 1.2,
            use_thin_strokes: false,
            enforce_minimum_contrast: true,
            show_ligatures: false,
            cursor_type: CursorType::Block,
            cursor_blink: true,
            
            // Window
            open_new_windows_with_custom_size: false,
            window_columns: 80,
            window_rows: 40,
            window_opacity: 100.0,
            window_blur_radius: 1,
            
            // Input
            input_type: InputType::Universal,
            
            // History
            history_enabled: true,
            max_history_entries: 10000,
            history_dedup_mode: HistoryDedupMode::Consecutive,
            history_save_on_exit: true,
            history_sync_across_sessions: false,
            history_exclude_patterns: vec!["rm *".to_string(), "sudo *".to_string()],
            history_include_exit_codes: true,
            history_auto_bookmark_successful: false,
            history_retention_days: 365,
            history_search_fuzzy: true,
            
            // Features
            enable_autocomplete: true,
            enable_ai_command_search: true,
            enable_smart_suggestions: true,
            
            // Layout Management
            pane_layout: None,
            auto_save_layout: true,
            restore_layout_on_startup: true,
            layout_autosave_interval: 30,
        }
    }
}

pub fn settings_view(state: &SettingsState, errors: &Vec<String>) -> Element<'static, Message> {
    // Error banner
    let error_banner: Element<Message> = if !errors.is_empty() {
        container(
            column![
                text("Settings Error(s):").size(16).style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))),
                column(
                    errors.iter().map(|e| text(e).size(13).style(iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.2, 0.2))).into()).collect::<Vec<Element<Message>>>()
                ).spacing(4)
            ]
        )
        .padding(12)
        .style(iced::theme::Container::Box)
        .width(Length::Fill)
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Header with title and close button
    let header = row![
        text("Settings").size(28),
        Space::with_width(Length::Fill),
        button(text("✕").size(16))
            .on_press(Message::InputChanged("toggle_settings".into()))
            .style(theme::Button::Destructive)
    ]
    .align_items(Alignment::Center)
    .spacing(20)
    .padding([20, 20, 10, 20]);

    // Navigation sidebar
    let sidebar_tabs = [
        SettingsTab::Themes,
        SettingsTab::Appearance, 
        SettingsTab::Window,
        SettingsTab::Icon,
        SettingsTab::Input,
        SettingsTab::History,
        SettingsTab::Features,
        SettingsTab::Advanced,
    ];

    let sidebar: Element<_> = sidebar_tabs.iter().fold(
        column![].spacing(8).padding([20, 20]),
        |col, &tab| {
            let is_active = tab == state.active_tab;
            let button_style = if is_active {
                theme::Button::Primary
            } else {
                theme::Button::Secondary
            };
            
            col.push(
                button(text(tab.to_string())
                    .size(14)
                    .horizontal_alignment(alignment::Horizontal::Left))
                    .width(Length::Fixed(120.0))
                    .on_press(Message::InputChanged(format!("tab_{:?}", tab)))
                    .style(button_style)
            )
        },
    ).into();

    // Main content area based on active tab
    let main_content = match state.active_tab {
        SettingsTab::Themes => themes_tab(state),
        SettingsTab::Appearance => appearance_tab(state),
        SettingsTab::Window => window_tab(state),
        SettingsTab::Icon => icon_tab(state),
        SettingsTab::Input => input_tab(state),
        SettingsTab::History => history_tab(state),
        SettingsTab::Features => features_tab(state),
        SettingsTab::Advanced => advanced_tab(state),
    };

    let content_area = scrollable(
        container(main_content)
            .padding([20, 40])
            .width(Length::Fill)
    )
    .height(Length::Fill);

    let body = row![
        container(sidebar)
            .width(Length::Fixed(160.0))
            .style(theme::Container::Box),
        iced::widget::vertical_rule(1),
        content_area,
    ].height(Length::Fill);

    column![
        error_banner,
        header,
        iced::widget::horizontal_rule(1),
        body,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// Helper components for better reusability
fn section_title(title: &str) -> Element<'static, Message> {
    column![
        text(title).size(24),
        Space::with_height(20)
    ].into()
}

fn setting_row<'a>(label: &str, control: Element<'a, Message>) -> Element<'a, Message> {
    row![
        text(label).size(13).width(Length::Fixed(120.0)),
        Space::with_width(10),
        control
    ].align_items(Alignment::Center).into()
}

#[allow(dead_code)]
fn subsection_title(title: &str) -> Element<'static, Message> {
    column![
        Space::with_height(20),
        text(title).size(16),
        Space::with_height(10)
    ].into()
}

fn themes_tab(state: &SettingsState) -> Element<'static, Message> {
    column![
        text("Themes").size(24),
        Space::with_height(20),
        button(text("Create your own custom theme").size(14))
            .style(theme::Button::Primary)
            .on_press(Message::InputChanged("create_theme".into())),
        Space::with_height(20),
        checkbox(
            "Sync with OS",
            state.sync_with_os
        )
        .on_toggle(|_| Message::InputChanged("sync_os".into()))
        .text_size(13),
        text("Automatically switch between light and dark themes when your system does.").size(12),
        Space::with_height(20),
        row![
            text("Light").size(14),
            Space::with_width(Length::Fill),
            button(text("Light"))
                .on_press(Message::InputChanged("light_theme_picker".into()))
                .style(theme::Button::Secondary),
        ]
        .align_items(Alignment::Center),
        Space::with_height(10),
        row![
            text("Dark").size(14),
            Space::with_width(Length::Fill),
            button(text("Dark"))
                .on_press(Message::InputChanged("dark_theme_picker".into()))
                .style(theme::Button::Secondary),
        ]
        .align_items(Alignment::Center),
    ]
    .spacing(12)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}

fn appearance_tab(state: &SettingsState) -> Element<'static, Message> {
    column![
        section_title("Appearance"),
        
        // Font section
        container(
            column![
                text("Text").size(16),
                Space::with_height(12),
                
                setting_row("Font Family:",
                    text_input("e.g. JetBrains Mono, Hack, SF Mono", &state.font_family)
                        .width(Length::Fixed(250.0))
                        .size(13)
                        .on_input(|_| Message::InputChanged("font_family".into()))
                        .into()
                ),
                
                Space::with_height(8),
                
                setting_row("Font Size:",
                    row![
                        slider(8..=32, state.font_size, |_| Message::InputChanged("font_size".into()))
                            .width(Length::Fixed(180.0)),
                        Space::with_width(12),
                        container(
                            text(format!("{} pt", state.font_size))
                                .size(13)
                        ).width(Length::Fixed(50.0)),
                    ].into()
                ),
                
                Space::with_height(8),
                
                setting_row("Line Height:",
                    row![
                        slider(0.8..=2.5, state.line_height, |_| Message::InputChanged("line_height".into()))
                            .width(Length::Fixed(180.0)),
                        Space::with_width(12),
                        container(
                            text(format!("{:.1}", state.line_height))
                                .size(13)
                        ).width(Length::Fixed(50.0)),
                    ].into()
                ),
                
                Space::with_height(16),
                
                // Font options
                text("Font Options").size(14).style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                Space::with_height(8),
                
                checkbox("Enable font ligatures (!=, ->, etc.)", state.show_ligatures)
                    .on_toggle(|_| Message::InputChanged("ligatures".into()))
                    .text_size(13),
                    
                checkbox("Use thin strokes on Retina displays", state.use_thin_strokes)
                    .on_toggle(|_| Message::InputChanged("thin_strokes".into()))
                    .text_size(13),
                    
                checkbox("Enforce minimum contrast", state.enforce_minimum_contrast)
                    .on_toggle(|_| Message::InputChanged("min_contrast".into()))
                    .text_size(13),
            ]
        )
        .padding([16, 20])
        .style(theme::Container::Box),
        
        Space::with_height(24),
        
        // Cursor section
        container(
            column![
                text("Cursor").size(16),
                Space::with_height(12),
                
                text("Cursor Style").size(14).style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                Space::with_height(8),
                
                row![
                    radio("Block", CursorType::Block, Some(state.cursor_type), |_| Message::InputChanged("cursor_block".into())),
                    Space::with_width(24),
                    radio("Bar", CursorType::Bar, Some(state.cursor_type), |_| Message::InputChanged("cursor_bar".into())),
                    Space::with_width(24),
                    radio("Underline", CursorType::Underline, Some(state.cursor_type), |_| Message::InputChanged("cursor_underline".into())),
                ].align_items(Alignment::Center),
                
                Space::with_height(16),
                
                text("Cursor Options").size(14).style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                Space::with_height(8),
                
                checkbox("Blinking cursor", state.cursor_blink)
                    .on_toggle(|_| Message::InputChanged("cursor_blink".into()))
                    .text_size(13),
            ]
        )
        .padding([16, 20])
        .style(theme::Container::Box),
    ]
    .spacing(8)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}

fn window_tab(state: &SettingsState) -> Element<'static, Message> {
    column![
        text("Window").size(24),
        Space::with_height(20),
        checkbox(
            "Open new windows with custom size",
            state.open_new_windows_with_custom_size
        )
        .on_toggle(|_| Message::InputChanged("custom_window_size".into()))
        .text_size(13),
        Space::with_height(16),
        row![
            text("Columns").size(13).width(Length::Fixed(80.0)),
            text_input("", &state.window_columns.to_string())
                .width(Length::Fixed(80.0))
                .size(13)
                .on_input(|_| Message::InputChanged("window_columns".into())),
        ]
        .align_items(Alignment::Center),
        row![
            text("Rows").size(13).width(Length::Fixed(80.0)),
            text_input("", &state.window_rows.to_string())
                .width(Length::Fixed(80.0))
                .size(13)
                .on_input(|_| Message::InputChanged("window_rows".into())),
        ]
        .align_items(Alignment::Center),
        Space::with_height(20),
        text(format!("Window Opacity: {}", state.window_opacity)).size(14),
        slider(0.0..=100.0, state.window_opacity, |_| {
            Message::InputChanged("window_opacity".into())
        })
        .width(Length::Fixed(200.0)),
        Space::with_height(16),
        row![
            text("Window Blur Radius: ").size(13),
            text(state.window_blur_radius.to_string()).size(13),
        ]
        .align_items(Alignment::Center),
        slider(0..=5, state.window_blur_radius, |_| {
            Message::InputChanged("window_blur".into())
        })
        .width(Length::Fixed(200.0)),
    ]
    .spacing(12)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}

fn icon_tab(_state: &SettingsState) -> Element<'static, Message> {
    column![
        text("Icon").size(24),
        Space::with_height(20),
        text("Customize your app icon").size(14),
        Space::with_height(20),
        button(text("Default"))
            .on_press(Message::InputChanged("icon_default".into())),
    ]
    .spacing(12)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}

fn input_tab(state: &SettingsState) -> Element<'static, Message> {
    column![
        text("Input").size(24),
        Space::with_height(20),
        text("Input type").size(14),
        Space::with_height(10),
        row![
            radio("Universal", InputType::Universal, Some(state.input_type), |_| Message::InputChanged("input_universal".into())),
            radio("Classic", InputType::Classic, Some(state.input_type), |_| Message::InputChanged("input_classic".into())),
        ].spacing(20),
    ]
    .spacing(12)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}

fn features_tab(state: &SettingsState) -> Element<'static, Message> {
    column![
        text("Features").size(24),
        Space::with_height(20),
        
        // Command Features
        container(
            column![
                text("Command Features").size(16),
                Space::with_height(12),
                
                checkbox("Enable autocomplete", state.enable_autocomplete)
                    .on_toggle(|_| Message::InputChanged("autocomplete".into()))
                    .text_size(13),
                checkbox("AI command search", state.enable_ai_command_search)
                    .on_toggle(|_| Message::InputChanged("ai_search".into()))
                    .text_size(13),
                checkbox("Smart suggestions", state.enable_smart_suggestions)
                    .on_toggle(|_| Message::InputChanged("smart_suggestions".into()))
                    .text_size(13),
            ]
        )
        .padding([16, 20])
        .style(theme::Container::Box),
        
        Space::with_height(24),
        
        // Layout Management
        container(
            column![
                text("Layout Management").size(16),
                Space::with_height(12),
                
                checkbox("Auto-save layout changes", state.auto_save_layout)
                    .on_toggle(|_| Message::InputChanged("auto_save_layout".into()))
                    .text_size(13),
                checkbox("Restore layout on startup", state.restore_layout_on_startup)
                    .on_toggle(|_| Message::InputChanged("restore_layout_on_startup".into()))
                    .text_size(13),
                
                Space::with_height(12),
                
                setting_row("Auto-save interval (sec):",
                    text_input("", &state.layout_autosave_interval.to_string())
                        .width(Length::Fixed(80.0))
                        .size(13)
                        .on_input(|_| Message::InputChanged("layout_autosave_interval".into()))
                        .into()
                ),
                
                Space::with_height(16),
                
                text("Layout Actions").size(14).style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                Space::with_height(8),
                
                row![
                    button(text("Save Current Layout").size(13))
                        .on_press(Message::InputChanged("save_current_layout".into()))
                        .style(theme::Button::Primary),
                    Space::with_width(12),
                    button(text("Load Layout").size(13))
                        .on_press(Message::InputChanged("load_layout".into()))
                        .style(theme::Button::Secondary),
                    Space::with_width(12),
                    button(text("Reset Layout").size(13))
                        .on_press(Message::InputChanged("reset_layout".into()))
                        .style(theme::Button::Destructive),
                ].align_items(Alignment::Center),
                
                Space::with_height(8),
                
                if state.pane_layout.is_some() {
                    text("✓ Layout saved").size(12).style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.8, 0.2)))
                } else {
                    text("No saved layout").size(12).style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)))
                },
            ]
        )
        .padding([16, 20])
        .style(theme::Container::Box),
    ]
    .spacing(8)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}

fn history_tab(state: &SettingsState) -> Element<'static, Message> {
    column![
        text("History").size(24),
        Space::with_height(20),

        checkbox("Enable history", state.history_enabled)
            .on_toggle(|_| Message::InputChanged("history_enabled".into()))
            .text_size(13),
        Space::with_height(10),

        row![
            text("Max entries:").size(14),
            Space::with_width(10),
            text_input("", &state.max_history_entries.to_string())
                .size(14)
                .on_input(|_| Message::InputChanged("max_history_entries".into()))
                .width(Length::Fixed(80.0)),
        ].align_items(Alignment::Center),
        Space::with_height(10),

        text("Deduplication mode:").size(14),
        row![
            radio("None", HistoryDedupMode::None, Some(state.history_dedup_mode), |_| Message::InputChanged("dedup_none".into())),
            radio("Consecutive", HistoryDedupMode::Consecutive, Some(state.history_dedup_mode), |_| Message::InputChanged("dedup_consecutive".into())),
            radio("Global", HistoryDedupMode::Global, Some(state.history_dedup_mode), |_| Message::InputChanged("dedup_global".into())),
        ].spacing(20),
        Space::with_height(10),
        
        checkbox("Save history on exit", state.history_save_on_exit)
            .on_toggle(|_| Message::InputChanged("history_save_on_exit".into()))
            .text_size(13),
        checkbox("Sync history across sessions", state.history_sync_across_sessions)
            .on_toggle(|_| Message::InputChanged("history_sync_across_sessions".into()))
            .text_size(13),
        checkbox("Include exit codes in history", state.history_include_exit_codes)
            .on_toggle(|_| Message::InputChanged("history_include_exit_codes".into()))
            .text_size(13),
        checkbox("Auto-bookmark successful commands", state.history_auto_bookmark_successful)
            .on_toggle(|_| Message::InputChanged("history_auto_bookmark_successful".into()))
            .text_size(13),
        checkbox("Enable fuzzy search in history", state.history_search_fuzzy)
            .on_toggle(|_| Message::InputChanged("history_search_fuzzy".into()))
            .text_size(13),

        Space::with_height(20),
        text("Exclude patterns:").size(14),
        
        column(
            state.history_exclude_patterns.iter().enumerate().map(|(i, pattern)| {
                row![
                    text(pattern).size(13),
                    Space::with_width(Length::Fill),
                    button(text("✕").size(12))
                        .on_press(Message::InputChanged(format!("remove_exclude_pattern_{}", i)))
                        .style(theme::Button::Destructive),
                ].align_items(Alignment::Center).into()
            }).collect::<Vec<Element<Message>>>()
        ).spacing(8),

        Space::with_height(20),
        button(text("+ Add Exclude Pattern").size(13))
            .on_press(Message::InputChanged("add_exclude_pattern".into()))
            .style(theme::Button::Secondary),

        Space::with_height(20),
        row![
            text("Retention (days):").size(14),
            Space::with_width(10),
            text_input("", &state.history_retention_days.to_string())
                .size(14)
                .on_input(|_| Message::InputChanged("history_retention_days".into()))
                .width(Length::Fixed(80.0)),
        ].align_items(Alignment::Center),

        Space::with_height(20),
        row![
            button(text("Clear History").size(13))
                .on_press(Message::InputChanged("clear_history".into()))
                .style(theme::Button::Destructive),
            Space::with_width(20),
            button(text("Export History").size(13))
                .on_press(Message::InputChanged("export_history".into()))
                .style(theme::Button::Secondary),
        ].align_items(Alignment::Center),
    ]
    .spacing(12)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}

fn advanced_tab(_state: &SettingsState) -> Element<'static, Message> {
    column![
        text("Advanced").size(24),
        Space::with_height(20),
        button(text("Reset to Defaults")
            .size(13))
            .style(theme::Button::Destructive)
            .on_press(Message::InputChanged("reset_defaults".into())),
        Space::with_height(20),
        button(text("Export Settings").size(13))
            .on_press(Message::InputChanged("export_settings".into()))
            .style(theme::Button::Secondary),
        button(text("Import Settings").size(13))
            .on_press(Message::InputChanged("import_settings".into()))
            .style(theme::Button::Secondary),
        button(text("Restore from Backup").size(13))
            .on_press(Message::InputChanged("restore_backup".into()))
            .style(theme::Button::Secondary),
        button(text("Create Manual Backup").size(13))
            .on_press(Message::InputChanged("manual_backup".into()))
            .style(theme::Button::Secondary),
    ]
    .spacing(12)
    .align_items(Alignment::Start)
    .width(Length::Fill)
    .into()
}
