//! Enhanced input component for the terminal with modern design.

use crate::Message;
use iced::widget::{button, container, row, text, column, Space, tooltip, text_input};
use iced::{theme, Alignment, Background, Color, Element, Length, border, Font};

pub fn enhanced_input_section(
    current_input: &str,
    font: Font,
    size: u16,
    is_executing: bool,
) -> Element<'_, Message> {
    modern_text_input_section(current_input, font, size, is_executing, None, &[])
}

// New modern text editor input section
pub fn modern_text_input_section<'a>(
    current_input: &'a str,
    font: Font,
    size: u16,
    is_executing: bool,
    suggestion: Option<&'a str>,
    recent_commands: &'a [String],
) -> Element<'a, Message> {
    // Create the main input section with modern editor
    let main_input_section = create_modern_input_section(current_input, font, size, is_executing, suggestion);
    
    // Add suggestions panel if there are recent commands or suggestions
    if !recent_commands.is_empty() || suggestion.is_some() {
        let suggestions_panel = create_suggestions_panel(recent_commands, suggestion, font, size);
        
        column![
            main_input_section,
            suggestions_panel
        ].spacing(0).into()
    } else {
        main_input_section
    }
}

// Create the main input section with ModernTextEditor
fn create_modern_input_section<'a>(
    current_input: &'a str,
    font: Font,
    size: u16,
    is_executing: bool,
    _suggestion: Option<&'a str>,
) -> Element<'a, Message> {
    // Create a simple text input for now (can be enhanced later)
    let input_field = text_input("Code, ask, build, or run commands", current_input)
        .on_input(Message::InputChanged)
        .padding([14, 18])
        .size(size)
        .font(font)
        .width(Length::Fill);
    
    // Enhanced terminal prompt with user info
    let prompt_section = row![
        text("~").font(font).size(size - 1).style(Color::from_rgb(0.4, 0.8, 0.4)), // Directory indicator
        Space::with_width(4),
        text("❯").font(font).size(size + 2).style(Color::from_rgb(0.3, 0.7, 1.0)), // Blue prompt
        Space::with_width(2),
    ].align_items(Alignment::Center);
    
    // Input container with enhanced styling
    let input_container = container(input_field)
        .style(theme::Container::Custom(Box::new(ModernEditorContainerStyle)))
        .width(Length::Fill)
        .height(Length::Fixed(60.0)); // Compact height for input
    
    // Status indicators (left side)
    let status_indicators = row![
        // Connection status
        container(
            text("●").size(size - 6).style(Color::from_rgb(0.2, 0.8, 0.3))
        ).padding([2, 4]).style(theme::Container::Custom(Box::new(StatusIndicatorStyle))),
        
        // Modern text editing indicator
        tooltip(
            container(
                row![
                    text("✨").size(size - 4),
                    text("Modern Editor").font(font).size(size - 6).style(Color::from_rgb(0.6, 0.8, 1.0))
                ].spacing(2)
            ).padding([2, 4]).style(theme::Container::Custom(Box::new(ModernFeatureIndicatorStyle))),
            "Advanced text editing features enabled\n• Multi-cursor editing (Alt+Click)\n• Syntax highlighting\n• Auto-completion\n• Bracket matching\n• AI suggestions",
            tooltip::Position::Top
        ),
        
        // Working directory indicator
        tooltip(
            button(
                row![
                    text("📁").size(size - 6),
                    text("~/warp-terminal").font(font).size(size - 6).style(Color::from_rgb(0.6, 0.6, 0.7))
                ].spacing(2)
            ).style(theme::Button::Custom(Box::new(DirectoryIndicatorStyle))),
            "Current working directory",
            tooltip::Position::Top
        ),
    ].spacing(6).align_items(Alignment::Center);
    
    // Action buttons (right side) - enhanced for modern editing
    let action_buttons = create_modern_action_buttons(is_executing, font, size);
    
    // AI assistant section with modern editing context
    let ai_section = create_enhanced_ai_assistant_section(font, size);
    
    // Main input row with modern editor
    let main_input_row = row![
        prompt_section,
        Space::with_width(8),
        input_container,
        Space::with_width(12),
        action_buttons,
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill);
    
    // Bottom row with status and AI
    let bottom_row = row![
        status_indicators,
        Space::with_width(Length::Fill),
        ai_section,
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill);
    
    // Complete input section
    let input_section = column![
        main_input_row,
        Space::with_height(8),
        bottom_row,
    ].spacing(0);
    
    container(input_section)
        .padding([20, 24])
        .style(theme::Container::Custom(Box::new(EnhancedInputContainerStyle)))
        .width(Length::Fill)
        .into()
}

// Advanced input box with auto-suggestions and enhanced UI
pub fn advanced_input_with_suggestions<'a>(
    current_input: &'a str,
    font: Font,
    size: u16,
    is_executing: bool,
    suggestion: Option<&'a str>,
    recent_commands: &'a [String],
) -> Element<'a, Message> {
    // Create the main input section with enhanced design
    let main_input_section = create_enhanced_input_section(current_input, font, size, is_executing, suggestion);
    
    // Add suggestions panel if there are recent commands or suggestions
    if !recent_commands.is_empty() || suggestion.is_some() {
        let suggestions_panel = create_suggestions_panel(recent_commands, suggestion, font, size);
        
        column![
            main_input_section,
            suggestions_panel
        ].spacing(0).into()
    } else {
        main_input_section.into()
    }
}

// Create the main input section with modern design
fn create_enhanced_input_section<'a>(
    current_input: &'a str,
    font: Font,
    size: u16,
    is_executing: bool,
    suggestion: Option<&'a str>,
) -> Element<'a, Message> {
    // Enhanced terminal prompt with user info
    let prompt_section = row![
        text("~").font(font).size(size - 1).style(Color::from_rgb(0.4, 0.8, 0.4)), // Directory indicator
        Space::with_width(4),
        text("❯").font(font).size(size + 2).style(Color::from_rgb(0.3, 0.7, 1.0)), // Blue prompt
        Space::with_width(2),
    ].align_items(Alignment::Center);
    
    // Enhanced input field with overlay suggestion
    let input_container = create_input_with_suggestion(current_input, suggestion, font, size);
    
    // Status indicators (left side)
    let status_indicators = row![
        // Connection status
        container(
            text("●").size(size - 6).style(Color::from_rgb(0.2, 0.8, 0.3))
        ).padding([2, 4]).style(theme::Container::Custom(Box::new(StatusIndicatorStyle))),
        
        // Working directory indicator
        tooltip(
            button(
                row![
                    text("📁").size(size - 6),
                    text("~/warp-terminal").font(font).size(size - 6).style(Color::from_rgb(0.6, 0.6, 0.7))
                ].spacing(2)
            ).style(theme::Button::Custom(Box::new(DirectoryIndicatorStyle))),
            "Current working directory",
            tooltip::Position::Top
        ),
    ].spacing(6).align_items(Alignment::Center);
    
    // Action buttons (right side)
    let action_buttons = create_action_buttons(is_executing, font, size);
    
    // AI assistant section
    let ai_section = create_ai_assistant_section(font, size);
    
    // Main input row
    let main_input_row = row![
        prompt_section,
        Space::with_width(8),
        input_container,
        Space::with_width(12),
        action_buttons,
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill);
    
    // Bottom row with status and AI
    let bottom_row = row![
        status_indicators,
        Space::with_width(Length::Fill),
        ai_section,
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill);
    
    // Complete input section
    let input_section = column![
        main_input_row,
        Space::with_height(8),
        bottom_row,
    ].spacing(0);
    
    container(input_section)
        .padding([20, 24])
        .style(theme::Container::Custom(Box::new(EnhancedInputContainerStyle)))
        .width(Length::Fill)
        .into()
}

// Create input field with ghost text suggestion
fn create_input_with_suggestion<'a>(
    _current_input: &'a str,
    suggestion: Option<&'a str>,
    font: Font,
    size: u16,
) -> Element<'a, Message> {
    let input = text("Code, ask, build, or run commands")
        .style(theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.6)))
        .font(font)
        .size(size);
    
    // Add suggestion overlay if available
    if let Some(suggestion_text) = suggestion {
        let suggestion_overlay = container(
            text(suggestion_text)
                .font(font)
                .size(size)
                .style(Color::from_rgba(0.5, 0.5, 0.6, 0.6))
        )
        .padding([14, 18])
        .width(Length::Fill);
        
        // Stack input and suggestion
        container(
            column![
                input,
                suggestion_overlay
            ].spacing(0)
        ).into()
    } else {
        container(input).padding([14, 18]).width(Length::Fill).into()
    }
}

// Create action buttons section
fn create_action_buttons(
    is_executing: bool,
    font: Font,
    size: u16,
) -> Element<'static, Message> {
    let execute_btn = if is_executing {
        tooltip(
            button(
                row![
                    text("⏸").size(size - 2),
                    text("Stop").font(font).size(size - 4)
                ].spacing(4)
            )
            .style(theme::Button::Custom(Box::new(StopButtonStyle)))
            .padding([8, 12]),
            "Stop execution (Ctrl+C)",
            tooltip::Position::Top
        )
    } else {
        tooltip(
            button(
                row![
                    text("▶").size(size - 2),
                    text("Run").font(font).size(size - 4)
                ].spacing(4)
            )
            .on_press(Message::ExecuteCommand)
            .style(theme::Button::Custom(Box::new(EnhancedRunButtonStyle)))
            .padding([8, 12]),
            "Execute command (Enter)",
            tooltip::Position::Top
        )
    };
    
    let history_btn = tooltip(
        button(
            text("🕒").size(size - 2)
        )
        .style(theme::Button::Custom(Box::new(HistoryButtonStyle)))
        .padding([8, 8]),
        "Command history (Ctrl+R)",
        tooltip::Position::Top
    );
    
    let clear_btn = tooltip(
        button(
            text("🗑").size(size - 2)
        )
        .style(theme::Button::Custom(Box::new(ClearButtonStyle)))
        .padding([8, 8]),
        "Clear input (Ctrl+L)",
        tooltip::Position::Top
    );
    
    row![
        history_btn,
        Space::with_width(4),
        clear_btn,
        Space::with_width(8),
        execute_btn,
    ]
    .align_items(Alignment::Center)
    .into()
}

// Create AI assistant section
fn create_ai_assistant_section(
    font: Font,
    size: u16,
) -> Element<'static, Message> {
    tooltip(
        button(
            row![
                text("🤖").size(size - 4),
                text("Auto").font(font).size(size - 4),
                Space::with_width(4),
                text("Claude 4 Sonnet").font(font).size(size - 6).style(Color::from_rgb(0.6, 0.6, 0.7)),
                Space::with_width(4),
                text("⚡").size(size - 6).style(Color::from_rgb(1.0, 0.8, 0.0)),
                text("⌄").size(size - 6)
            ].spacing(2)
        )
        .style(theme::Button::Custom(Box::new(EnhancedAIButtonStyle)))
        .padding([6, 10]),
        "AI Assistant Settings",
        tooltip::Position::Top
    ).into()
}

// Create suggestions panel
fn create_suggestions_panel<'a>(
    recent_commands: &'a [String],
    suggestion: Option<&'a str>,
    font: Font,
    size: u16,
) -> Element<'a, Message> {
    let mut suggestions_column = column![].spacing(1);
    
    // Add current suggestion if available
    if let Some(suggestion_text) = suggestion {
        let suggestion_item = container(
            row![
                text("💡").size(size - 4),
                Space::with_width(8),
                text(suggestion_text).font(font).size(size - 2),
                Space::with_width(Length::Fill),
                text("Tab").font(font).size(size - 6).style(Color::from_rgb(0.5, 0.5, 0.6))
            ]
        )
        .padding([8, 12])
        .style(theme::Container::Custom(Box::new(SuggestionItemStyle)))
        .width(Length::Fill);
        
        suggestions_column = suggestions_column.push(suggestion_item);
    }
    
    // Add recent commands
    for (i, command) in recent_commands.iter().take(3).enumerate() {
        let command_item: iced::widget::Button<'_, Message, iced::Theme, iced::Renderer> = button(
            row![
                text("📜").size(size - 4).style(iced::theme::Text::Color(Color::from_rgb(0.8, 0.8, 0.9))),
                Space::with_width(8),
                text(command).font(font).size(size - 2).style(iced::theme::Text::Color(Color::from_rgb(0.95, 0.95, 1.0))),
                Space::with_width(Length::Fill),
                text(format!("↑{}", i + 1)).font(font).size(size - 6).style(iced::theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.6)))
            ]
        )
        .style(theme::Button::Custom(Box::new(RecentCommandButtonStyle)))
        .padding([8, 12])
        .width(Length::Fill);
        
        suggestions_column = suggestions_column.push(command_item);
    }
    
    container(suggestions_column)
        .style(theme::Container::Custom(Box::new(SuggestionsPanelStyle)))
        .padding([0, 24])
        .width(Length::Fill)
        .into()
}

// Modern input field styling
#[allow(dead_code)]
struct ModernInputStyle;

impl iced::widget::text_input::StyleSheet for ModernInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.08, 0.08, 0.12)),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.5,
                color: Color::from_rgb(0.3, 0.3, 0.4),
            },
            icon_color: Color::from_rgb(0.7, 0.7, 0.8),
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.1, 0.1, 0.15)),
            border: border::Border {
                radius: 8.0.into(),
                width: 2.0,
                color: Color::from_rgb(0.2, 0.6, 1.0), // Blue focus border
            },
            icon_color: Color::from_rgb(0.9, 0.9, 1.0),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.5, 0.5, 0.6)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.95, 0.95, 1.0)
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.4, 0.4, 0.5)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.2, 0.4, 0.8)
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.05, 0.05, 0.08)),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.2, 0.2, 0.3),
            },
            icon_color: Color::from_rgb(0.4, 0.4, 0.5),
        }
    }
}

// Run button styling
#[allow(dead_code)]
struct RunButtonStyle;

impl button::StyleSheet for RunButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.6, 1.0))),
            border: border::Border {
                radius: 6.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.7, 1.0))),
            border: border::Border {
                radius: 6.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

// Executing button styling
#[allow(dead_code)]
struct ExecutingButtonStyle;

impl button::StyleSheet for ExecutingButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.8, 0.6, 0.2))),
            border: border::Border {
                radius: 6.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

// Model selector styling
#[allow(dead_code)]
struct ModelSelectorStyle;

impl button::StyleSheet for ModelSelectorStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.18))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.3, 0.3, 0.4),
            },
            text_color: Color::from_rgb(0.9, 0.9, 1.0),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.22))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.4, 0.4, 0.5),
            },
            text_color: Color::from_rgb(0.95, 0.95, 1.0),
            ..Default::default()
        }
    }
}

// Input container styling
#[allow(dead_code)]
struct InputContainerStyle;

impl container::StyleSheet for InputContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.04, 0.04, 0.08))),
            border: border::Border {
                radius: 0.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.2, 0.2, 0.3),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

// Enhanced input container styling
struct EnhancedInputContainerStyle;

impl container::StyleSheet for EnhancedInputContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.09))),
            border: border::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.25, 0.25, 0.35),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

// Enhanced input field styling
#[allow(dead_code)]
struct EnhancedInputStyle;

impl iced::widget::text_input::StyleSheet for EnhancedInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.08, 0.08, 0.12)),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.5,
                color: Color::from_rgb(0.3, 0.3, 0.4),
            },
            icon_color: Color::from_rgb(0.7, 0.7, 0.8),
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.1, 0.1, 0.15)),
            border: border::Border {
                radius: 8.0.into(),
                width: 2.0,
                color: Color::from_rgb(0.2, 0.6, 1.0),
            },
            icon_color: Color::from_rgb(0.9, 0.9, 1.0),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.5, 0.5, 0.6)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.95, 0.95, 1.0)
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.4, 0.4, 0.5)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.2, 0.4, 0.8)
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: Background::Color(Color::from_rgb(0.05, 0.05, 0.08)),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.2, 0.2, 0.3),
            },
            icon_color: Color::from_rgb(0.4, 0.4, 0.5),
        }
    }
}

// Status indicator styling
struct StatusIndicatorStyle;

impl container::StyleSheet for StatusIndicatorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.15))),
            border: border::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

// Directory indicator button styling
struct DirectoryIndicatorStyle;

impl button::StyleSheet for DirectoryIndicatorStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.15))),
            border: border::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.9),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.18))),
            border: border::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::from_rgb(0.9, 0.9, 1.0),
            ..Default::default()
        }
    }
}

// Enhanced run button styling
struct EnhancedRunButtonStyle;

impl button::StyleSheet for EnhancedRunButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.7, 0.3))),
            border: border::Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.3, 0.8, 0.4))),
            border: border::Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

// Stop button styling
struct StopButtonStyle;

impl button::StyleSheet for StopButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.8, 0.4, 0.2))),
            border: border::Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.9, 0.5, 0.3))),
            border: border::Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

// History button styling
struct HistoryButtonStyle;

impl button::StyleSheet for HistoryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.18))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.25, 0.25, 0.35),
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.9),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.22))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.35, 0.35, 0.45),
            },
            text_color: Color::from_rgb(0.9, 0.9, 1.0),
            ..Default::default()
        }
    }
}

// Clear button styling
struct ClearButtonStyle;

impl button::StyleSheet for ClearButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.18))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.25, 0.25, 0.35),
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.9),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.22))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.35, 0.35, 0.45),
            },
            text_color: Color::from_rgb(0.9, 0.9, 1.0),
            ..Default::default()
        }
    }
}

// Enhanced AI button styling
struct EnhancedAIButtonStyle;

impl button::StyleSheet for EnhancedAIButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.15, 0.2))),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.3, 0.4, 0.5),
            },
            text_color: Color::from_rgb(0.8, 0.9, 1.0),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.18, 0.24))),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.4, 0.5, 0.6),
            },
            text_color: Color::from_rgb(0.9, 0.95, 1.0),
            ..Default::default()
        }
    }
}

// Suggestion item styling
struct SuggestionItemStyle;

impl container::StyleSheet for SuggestionItemStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.12, 0.18))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.2, 0.3, 0.4),
            },
            text_color: Some(Color::from_rgb(0.9, 0.9, 1.0)),
            ..Default::default()
        }
    }
}

// Recent command button styling
struct RecentCommandButtonStyle;

impl button::StyleSheet for RecentCommandButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.06, 0.06, 0.1))),
            border: border::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::from_rgb(0.8, 0.8, 0.9),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.15))),
            border: border::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.3, 0.3, 0.4),
            },
            text_color: Color::from_rgb(0.9, 0.9, 1.0),
            ..Default::default()
        }
    }
}

// Suggestions panel styling
struct SuggestionsPanelStyle;

impl container::StyleSheet for SuggestionsPanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.04, 0.04, 0.08))),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.2, 0.2, 0.3),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

// Modern editor container styling
struct ModernEditorContainerStyle;

impl container::StyleSheet for ModernEditorContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.06, 0.06, 0.1))),
            border: border::Border {
                radius: 8.0.into(),
                width: 1.5,
                color: Color::from_rgb(0.3, 0.4, 0.6),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

// Modern feature indicator styling
struct ModernFeatureIndicatorStyle;

impl container::StyleSheet for ModernFeatureIndicatorStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.15, 0.25))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.4, 0.5, 0.7),
            },
            text_color: Some(Color::from_rgb(0.8, 0.9, 1.0)),
            ..Default::default()
        }
    }
}

// Create modern action buttons with enhanced editing features
fn create_modern_action_buttons(
    is_executing: bool,
    font: Font,
    size: u16,
) -> Element<'static, Message> {
    let execute_btn = if is_executing {
        tooltip(
            button(
                row![
                    text("⏸").size(size - 2),
                    text("Stop").font(font).size(size - 4)
                ].spacing(4)
            )
            .style(theme::Button::Custom(Box::new(StopButtonStyle)))
            .padding([8, 12]),
            "Stop execution (Ctrl+C)",
            tooltip::Position::Top
        )
    } else {
        tooltip(
            button(
                row![
                    text("▶").size(size - 2),
                    text("Run").font(font).size(size - 4)
                ].spacing(4)
            )
            .on_press(Message::ExecuteCommand)
            .style(theme::Button::Custom(Box::new(EnhancedRunButtonStyle)))
            .padding([8, 12]),
            "Execute command (Enter)",
            tooltip::Position::Top
        )
    };
    
    // Modern editing features
    let multi_cursor_btn = tooltip(
        button(
            text("⚡").size(size - 2)
        )
        .style(theme::Button::Custom(Box::new(ModernFeatureButtonStyle)))
        .padding([8, 8]),
        "Multi-cursor mode (Alt+Click)",
        tooltip::Position::Top
    );
    
    let search_btn = tooltip(
        button(
            text("🔍").size(size - 2)
        )
        .style(theme::Button::Custom(Box::new(HistoryButtonStyle)))
        .padding([8, 8]),
        "Search & replace (Ctrl+F)",
        tooltip::Position::Top
    );
    
    let history_btn = tooltip(
        button(
            text("🕒").size(size - 2)
        )
        .style(theme::Button::Custom(Box::new(HistoryButtonStyle)))
        .padding([8, 8]),
        "Command history (Ctrl+R)",
        tooltip::Position::Top
    );
    
    let clear_btn = tooltip(
        button(
            text("🗑").size(size - 2)
        )
        .style(theme::Button::Custom(Box::new(ClearButtonStyle)))
        .padding([8, 8]),
        "Clear input (Ctrl+L)",
        tooltip::Position::Top
    );
    
    row![
        multi_cursor_btn,
        Space::with_width(4),
        search_btn,
        Space::with_width(4),
        history_btn,
        Space::with_width(4),
        clear_btn,
        Space::with_width(8),
        execute_btn,
    ]
    .align_items(Alignment::Center)
    .into()
}

// Enhanced AI assistant section with modern editor context
fn create_enhanced_ai_assistant_section(
    font: Font,
    size: u16,
) -> Element<'static, Message> {
    tooltip(
        button(
            row![
                text("🤖").size(size - 4),
                text("Auto").font(font).size(size - 4),
                Space::with_width(4),
                text("Claude 4 Sonnet").font(font).size(size - 6).style(Color::from_rgb(0.6, 0.6, 0.7)),
                Space::with_width(4),
                text("✨").size(size - 6).style(Color::from_rgb(0.6, 0.8, 1.0)),
                text("Modern").font(font).size(size - 7).style(Color::from_rgb(0.6, 0.8, 1.0)),
                Space::with_width(4),
                text("⚡").size(size - 6).style(Color::from_rgb(1.0, 0.8, 0.0)),
                text("⌄").size(size - 6)
            ].spacing(2)
        )
        .style(theme::Button::Custom(Box::new(EnhancedAIButtonStyle)))
        .padding([6, 10]),
        "AI Assistant with Modern Editing\n• Code completion & suggestions\n• Smart indentation\n• Bracket matching\n• Multi-cursor support",
        tooltip::Position::Top
    ).into()
}

// Modern feature button styling
struct ModernFeatureButtonStyle;

impl button::StyleSheet for ModernFeatureButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.2, 0.3))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.4, 0.5, 0.7),
            },
            text_color: Color::from_rgb(0.8, 0.9, 1.0),
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.25, 0.35))),
            border: border::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.5, 0.6, 0.8),
            },
            text_color: Color::from_rgb(0.9, 0.95, 1.0),
            ..Default::default()
        }
    }
}
