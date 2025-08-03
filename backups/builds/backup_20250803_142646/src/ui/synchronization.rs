//! UI components for Synchronized Inputs feature.
//!
//! This module provides UI widgets for displaying synchronization status,
//! controls for managing synchronization modes, and visual indicators.

use crate::model::synchronization::{SynchronizationStatus, SynchronizationScope};
use crate::Message;
use iced::widget::{button, column, container, row, text, tooltip};
use iced::{theme, Alignment, Background, Color, Element, Font, Length, border};

/// Create a synchronization status indicator with controls
pub fn synchronization_panel(
    status: &SynchronizationStatus,
    font: Font,
    font_size: u16,
) -> Element<'_, Message> {
    let status_color = if status.is_active {
        Color::from_rgb(0.4, 0.8, 0.4) // Green for active
    } else {
        Color::from_rgb(0.6, 0.6, 0.7) // Gray for inactive
    };

    // Status indicator with emoji
    let status_indicator = text(status.short_indicator())
        .size(font_size)
        .style(status_color);

    // Status text description
    let status_text = text(status.description())
        .font(font)
        .size(font_size - 2)
        .style(status_color);

    // Toggle button for cycling through synchronization modes
    let sync_button = button(
        row![
            text("🔄").size(font_size - 2),
            text(" Sync").font(font).size(font_size - 2),
        ]
        .spacing(4)
        .align_items(Alignment::Center),
    )
    .on_press(Message::ToggleSynchronizationMode)
    .style(theme::Button::Custom(Box::new(SyncButtonStyle { 
        is_active: status.is_active 
    })));

    // Individual scope buttons for quick access
    let current_tab_button = button(
        text("Tab").font(font).size(font_size - 3)
    )
    .on_press(Message::StartSynchronization(SynchronizationScope::CurrentTab))
    .style(theme::Button::Custom(Box::new(ScopeButtonStyle {
        is_selected: matches!(status.scope, SynchronizationScope::CurrentTab),
        is_active: status.is_active,
    })));

    let all_tabs_button = button(
        text("All").font(font).size(font_size - 3)
    )
    .on_press(Message::StartSynchronization(SynchronizationScope::AllTabs))
    .style(theme::Button::Custom(Box::new(ScopeButtonStyle {
        is_selected: matches!(status.scope, SynchronizationScope::AllTabs),
        is_active: status.is_active,
    })));

    let stop_button = button(
        text("⏹").size(font_size - 3)
    )
    .on_press(Message::StopSynchronization)
    .style(theme::Button::Custom(Box::new(StopButtonStyle)));

    // Main synchronization controls row
    let controls_row = row![
        status_indicator,
        status_text,
        sync_button,
        current_tab_button,
        all_tabs_button,
        stop_button,
    ]
    .spacing(8)
    .align_items(Alignment::Center);

    // Wrap in a tooltip with keyboard shortcuts
    let tooltip_text = if status.is_active {
        format!(
            "Synchronized Inputs: {}\nPress Ctrl+Alt+I (Cmd+Opt+I) to toggle\nSync active for {} panes",
            status.description(),
            status.target_pane_count
        )
    } else {
        "Synchronized Inputs: Off\nPress Ctrl+Alt+I (Cmd+Opt+I) to enable\nClick 'Tab' or 'All' to choose scope".to_string()
    };

    tooltip(
        controls_row,
        text(tooltip_text).size(font_size - 4),
        tooltip::Position::Top,
    )
    .style(theme::Container::Custom(Box::new(TooltipStyle)))
    .into()
}

/// Create a compact synchronization status indicator for the input area
pub fn compact_sync_indicator(
    status: &SynchronizationStatus,
    font_size: u16,
) -> Element<Message> {
    let indicator_text = if status.is_active {
        format!("{} {}", status.short_indicator(), status.target_pane_count)
    } else {
        "⚫".to_string()
    };

    let status_color = if status.is_active {
        Color::from_rgb(0.4, 0.8, 0.4)
    } else {
        Color::from_rgb(0.4, 0.4, 0.5)
    };

    button(text(indicator_text).size(font_size - 2).style(status_color))
        .on_press(Message::ToggleSynchronizationMode)
        .style(theme::Button::Custom(Box::new(CompactIndicatorStyle {
            is_active: status.is_active,
        })))
        .into()
}

/// Create a settings panel section for synchronization configuration
pub fn synchronization_settings_section(
    status: &SynchronizationStatus,
    font: Font,
    font_size: u16,
) -> Element<'_, Message> {
    let title = text("Synchronized Inputs")
        .font(font)
        .size(font_size + 2)
        .style(Color::from_rgb(0.9, 0.9, 1.0));

    let description = text(
        "Sync commands across multiple panes. Choose scope and manage synchronization behavior."
    )
    .font(font)
    .size(font_size - 2)
    .style(Color::from_rgb(0.7, 0.7, 0.8));

    let current_status = text(format!("Current Status: {}", status.description()))
        .font(font)
        .size(font_size - 1)
        .style(if status.is_active {
            Color::from_rgb(0.4, 0.8, 0.4)
        } else {
            Color::from_rgb(0.8, 0.8, 0.9)
        });

    // Control buttons
    let enable_current_tab = button(
        column![
            text("Current Tab").font(font).size(font_size - 2),
            text("Sync panes in active tab").font(font).size(font_size - 4).style(Color::from_rgb(0.6, 0.6, 0.7)),
        ]
        .align_items(Alignment::Center)
        .spacing(2)
    )
    .on_press(Message::StartSynchronization(SynchronizationScope::CurrentTab))
    .style(theme::Button::Custom(Box::new(SettingsButtonStyle)));

    let enable_all_tabs = button(
        column![
            text("All Tabs").font(font).size(font_size - 2),
            text("Sync panes across all tabs").font(font).size(font_size - 4).style(Color::from_rgb(0.6, 0.6, 0.7)),
        ]
        .align_items(Alignment::Center)
        .spacing(2)
    )
    .on_press(Message::StartSynchronization(SynchronizationScope::AllTabs))
    .style(theme::Button::Custom(Box::new(SettingsButtonStyle)));

    let disable_sync = button(
        column![
            text("Disable").font(font).size(font_size - 2),
            text("Turn off synchronization").font(font).size(font_size - 4).style(Color::from_rgb(0.6, 0.6, 0.7)),
        ]
        .align_items(Alignment::Center)
        .spacing(2)
    )
    .on_press(Message::StopSynchronization)
    .style(theme::Button::Custom(Box::new(SettingsButtonStyle)));

    let controls_row = row![
        enable_current_tab,
        enable_all_tabs,
        disable_sync,
    ]
    .spacing(12);

    let keyboard_shortcuts = text(
        "Keyboard Shortcuts:\n• Ctrl+Alt+I (Cmd+Opt+I): Toggle synchronization mode\n• Works with vim/emacs modes when active"
    )
    .font(font)
    .size(font_size - 3)
    .style(Color::from_rgb(0.6, 0.6, 0.7));

    column![
        title,
        description,
        current_status,
        controls_row,
        keyboard_shortcuts,
    ]
    .spacing(12)
    .padding(16)
    .width(Length::Fill)
    .into()
}

// Custom button styles for synchronization UI

struct SyncButtonStyle {
    is_active: bool,
}

impl button::StyleSheet for SyncButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active {
                Color::from_rgb(0.2, 0.6, 1.0)
            } else {
                Color::from_rgb(0.3, 0.3, 0.4)
            })),
            border: border::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active {
                Color::from_rgb(0.3, 0.7, 1.0)
            } else {
                Color::from_rgb(0.4, 0.4, 0.5)
            })),
            border: border::Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

struct ScopeButtonStyle {
    is_selected: bool,
    is_active: bool,
}

impl button::StyleSheet for ScopeButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        let (bg_color, text_color) = if self.is_selected && self.is_active {
            (Color::from_rgb(0.4, 0.8, 0.4), Color::WHITE)
        } else {
            (Color::from_rgb(0.2, 0.2, 0.3), Color::from_rgb(0.7, 0.7, 0.8))
        };

        button::Appearance {
            background: Some(Background::Color(bg_color)),
            border: border::Border {
                radius: 3.0.into(),
                width: 1.0,
                color: if self.is_selected {
                    Color::from_rgb(0.5, 0.9, 0.5)
                } else {
                    Color::from_rgb(0.3, 0.3, 0.4)
                },
            },
            text_color,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_selected && self.is_active {
                Color::from_rgb(0.5, 0.9, 0.5)
            } else {
                Color::from_rgb(0.3, 0.3, 0.4)
            })),
            border: border::Border {
                radius: 3.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.5, 0.5, 0.6),
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

struct StopButtonStyle;

impl button::StyleSheet for StopButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.8, 0.3, 0.3))),
            border: border::Border {
                radius: 3.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.9, 0.4, 0.4))),
            border: border::Border {
                radius: 3.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

pub struct CompactIndicatorStyle {
    pub is_active: bool,
}

impl button::StyleSheet for CompactIndicatorStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active {
                Color::from_rgb(0.15, 0.3, 0.15)
            } else {
                Color::from_rgb(0.1, 0.1, 0.15)
            })),
            border: border::Border {
                radius: 3.0.into(),
                width: 1.0,
                color: if self.is_active {
                    Color::from_rgb(0.4, 0.8, 0.4)
                } else {
                    Color::from_rgb(0.3, 0.3, 0.4)
                },
            },
            text_color: if self.is_active {
                Color::from_rgb(0.4, 0.8, 0.4)
            } else {
                Color::from_rgb(0.6, 0.6, 0.7)
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active {
                Color::from_rgb(0.2, 0.4, 0.2)
            } else {
                Color::from_rgb(0.15, 0.15, 0.2)
            })),
            border: border::Border {
                radius: 3.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.5, 0.5, 0.6),
            },
            text_color: Color::WHITE,
            ..Default::default()
        }
    }
}

struct SettingsButtonStyle;

impl button::StyleSheet for SettingsButtonStyle {
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

struct TooltipStyle;

impl container::StyleSheet for TooltipStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.08, 0.12))),
            border: border::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.3, 0.3, 0.4),
            },
            text_color: Some(Color::from_rgb(0.9, 0.9, 1.0)),
            ..Default::default()
        }
    }
}
