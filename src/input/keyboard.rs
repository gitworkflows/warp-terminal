//! Keyboard event handling and shortcuts for the terminal application.
//!
//! This module provides keyboard shortcuts for synchronization features and other
//! application functionality.

use crate::model::synchronization::SynchronizationScope;
use crate::Message;
use iced::keyboard::{Event as KeyboardEvent, Key, Modifiers};
use iced::Event;
use uuid::Uuid;

/// Keyboard shortcuts configuration
#[derive(Debug, Clone)]
pub struct KeyboardShortcuts {
    /// Whether to use Command key on macOS (instead of Ctrl)
    pub use_cmd_on_macos: bool,
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self {
            use_cmd_on_macos: cfg!(target_os = "macos"),
        }
    }
}

impl KeyboardShortcuts {
    /// Handle keyboard events and return corresponding messages
    pub fn handle_event(&self, event: &Event, active_pane_id: Option<Uuid>, pane_manager: &crate::model::pane::PaneManager) -> Option<Message> {
        if let Event::Keyboard(KeyboardEvent::KeyPressed { key, modifiers, .. }) = event {
            self.handle_key_press(key, modifiers, active_pane_id, pane_manager)
        } else {
            None
        }
    }

    /// Handle key press events with modifiers
    fn handle_key_press(
        &self,
        key: &Key,
        modifiers: &Modifiers,
        active_pane_id: Option<Uuid>,
        pane_manager: &crate::model::pane::PaneManager,
    ) -> Option<Message> {
        let primary_modifier = if self.use_cmd_on_macos {
            modifiers.command()
        } else {
            modifiers.control()
        };

        match key {
            // Pane Navigation shortcuts (Task 1.2 requirements)
            Key::Character(c) if c.as_str() == "d" && primary_modifier && modifiers.shift() => {
                // Ctrl+Shift+D - Split horizontally
                Some(Message::PaneSplitHorizontal)
            }

            Key::Character(c) if c.as_str() == "D" && primary_modifier && modifiers.shift() => {
                // Ctrl+Shift+Shift+D (effectively Ctrl+Shift+D with caps) - Split vertically
                Some(Message::PaneSplitVertical)
            }

            Key::Character(c) if c.as_str() == "w" && primary_modifier => {
                // Ctrl+W - Close current pane
                Some(Message::PaneClose)
            }

            Key::Named(iced::keyboard::key::Named::Tab)
                if primary_modifier && !modifiers.shift() =>
            {
                // Ctrl+Tab - Focus next pane
                Some(Message::PaneFocusNext)
            }

            Key::Named(iced::keyboard::key::Named::Tab)
                if primary_modifier && modifiers.shift() =>
            {
                // Ctrl+Shift+Tab - Focus previous pane
                Some(Message::PaneFocusPrevious)
            }

            Key::Named(iced::keyboard::key::Named::ArrowUp) if modifiers.alt() => {
                // Alt+Up - Focus pane above
                Some(Message::PaneFocusDirection(
                    crate::app::terminal::Direction::Up,
                ))
            }

            Key::Named(iced::keyboard::key::Named::ArrowDown) if modifiers.alt() => {
                // Alt+Down - Focus pane below
                Some(Message::PaneFocusDirection(
                    crate::app::terminal::Direction::Down,
                ))
            }

            Key::Named(iced::keyboard::key::Named::ArrowLeft) if modifiers.alt() => {
                // Alt+Left - Focus pane to the left
                Some(Message::PaneFocusDirection(
                    crate::app::terminal::Direction::Left,
                ))
            }

            Key::Named(iced::keyboard::key::Named::ArrowRight) if modifiers.alt() => {
                // Alt+Right - Focus pane to the right
                Some(Message::PaneFocusDirection(
                    crate::app::terminal::Direction::Right,
                ))
            }

            // Pane Resizing shortcuts
            Key::Named(iced::keyboard::key::Named::ArrowUp)
                if modifiers.alt() && modifiers.shift() =>
            {
                active_pane_id.and_then(|id| {
                    pane_manager.find_sibling_pane_id(id, crate::model::pane::SplitDirection::Vertical)
                        .map(|sibling_id| Message::PaneResize(id, sibling_id, -10))
                })
            }
            Key::Named(iced::keyboard::key::Named::ArrowDown)
                if modifiers.alt() && modifiers.shift() =>
            {
                active_pane_id.and_then(|id| {
                    pane_manager.find_sibling_pane_id(id, crate::model::pane::SplitDirection::Vertical)
                        .map(|sibling_id| Message::PaneResize(id, sibling_id, 10))
                })
            }
            Key::Named(iced::keyboard::key::Named::ArrowLeft)
                if modifiers.alt() && modifiers.shift() =>
            {
                active_pane_id.and_then(|id| {
                    pane_manager.find_sibling_pane_id(id, crate::model::pane::SplitDirection::Horizontal)
                        .map(|sibling_id| Message::PaneResize(id, sibling_id, -10))
                })
            }
            Key::Named(iced::keyboard::key::Named::ArrowRight)
                if modifiers.alt() && modifiers.shift() =>
            {
                active_pane_id.and_then(|id| {
                    pane_manager.find_sibling_pane_id(id, crate::model::pane::SplitDirection::Horizontal)
                        .map(|sibling_id| Message::PaneResize(id, sibling_id, 10))
                })
            }

            // Command Palette shortcuts
            Key::Character(c) if c.as_str() == "p" && primary_modifier && modifiers.shift() => {
                // Ctrl+Shift+P - Toggle command palette
                Some(Message::CommandPaletteToggle)
            }

            // Existing shortcuts continue below
            // Synchronization shortcuts
            Key::Character(c) if c.as_str() == "i" && primary_modifier && modifiers.alt() => {
                // Ctrl+Alt+I (Cmd+Opt+I on macOS) - Toggle synchronization
                Some(Message::ToggleSynchronizationMode)
            }

            Key::Character(c) if c.as_str() == "t" && primary_modifier && modifiers.alt() => {
                // Ctrl+Alt+T (Cmd+Opt+T on macOS) - Enable current tab sync
                Some(Message::StartSynchronization(
                    SynchronizationScope::CurrentTab,
                ))
            }

            Key::Character(c) if c.as_str() == "a" && primary_modifier && modifiers.alt() => {
                // Ctrl+Alt+A (Cmd+Opt+A on macOS) - Enable all tabs sync
                Some(Message::StartSynchronization(SynchronizationScope::AllTabs))
            }

            Key::Character(c) if c.as_str() == "s" && primary_modifier && modifiers.alt() => {
                // Ctrl+Alt+S (Cmd+Opt+S on macOS) - Stop synchronization
                Some(Message::StopSynchronization)
            }

            Key::Character(c) if c.as_str() == "p" && primary_modifier && modifiers.alt() => {
                // Ctrl+Alt+P (Cmd+Opt+P on macOS) - Pause/Resume synchronization
                Some(Message::ToggleSynchronizationPause)
            }

            // Existing shortcuts
            Key::Character(c) if c.as_str() == "r" && modifiers.control() => {
                // Ctrl+R - Toggle command search
                Some(Message::ToggleCommandSearch)
            }

            Key::Named(iced::keyboard::key::Named::Enter) => {
                // Enter - Execute command (only if not in command search mode)
                Some(Message::ExecuteCommand)
            }

            Key::Named(iced::keyboard::key::Named::Escape) => {
                // Escape - Close command search or cancel synchronization
                Some(Message::HandleEscape)
            }

            // Function keys for quick sync scope switching
            Key::Named(iced::keyboard::key::Named::F1) => {
                // F1 - Toggle to current tab sync
                Some(Message::StartSynchronization(
                    SynchronizationScope::CurrentTab,
                ))
            }

            Key::Named(iced::keyboard::key::Named::F2) => {
                // F2 - Toggle to all tabs sync
                Some(Message::StartSynchronization(SynchronizationScope::AllTabs))
            }

            Key::Named(iced::keyboard::key::Named::F3) => {
                // F3 - Stop synchronization
                Some(Message::StopSynchronization)
            }

            // Navigation keys for command search
            Key::Named(iced::keyboard::key::Named::ArrowUp) if !modifiers.is_empty() => {
                Some(Message::CommandSearchNavigateUp)
            }

            Key::Named(iced::keyboard::key::Named::ArrowDown) if !modifiers.is_empty() => {
                Some(Message::CommandSearchNavigateDown)
            }

            _ => None,
        }
    }

    /// Get help text for keyboard shortcuts
    pub fn get_help_text(&self) -> String {
        let modifier = if self.use_cmd_on_macos { "Cmd" } else { "Ctrl" };
        let alt = if self.use_cmd_on_macos { "Opt" } else { "Alt" };

        format!(
            "Pane Navigation Shortcuts:\n\
            • {modifier}+Shift+D - Split pane horizontally\n\
            • {modifier}+Shift+D (caps) - Split pane vertically\n\
            • {modifier}+W - Close current pane\n\
            • {modifier}+Tab - Focus next pane\n\
            • {modifier}+Shift+Tab - Focus previous pane\n\
            • {alt}+Arrow Keys - Focus pane in direction\n\
            \n\
            Command Palette:\n\
            • {modifier}+Shift+P - Toggle command palette\n\
            \n\
            Synchronization Shortcuts:\n\
            • {modifier}+{alt}+I - Toggle synchronization mode\n\
            • {modifier}+{alt}+T - Enable current tab sync\n\
            • {modifier}+{alt}+A - Enable all tabs sync\n\
            • {modifier}+{alt}+S - Stop synchronization\n\
            • {modifier}+{alt}+P - Pause/Resume synchronization\n\
            • F1 - Quick: Current tab sync\n\
            • F2 - Quick: All tabs sync\n\
            • F3 - Quick: Stop sync\n\
            \n\
            Other Shortcuts:\n\
            • Ctrl+R - Toggle command search\n\
            • Enter - Execute command\n\
            • Escape - Close panels/Cancel"
        )
    }

    /// Check if a key combination is reserved for synchronization
    pub fn is_sync_shortcut(&self, key: &Key, modifiers: &Modifiers) -> bool {
        let primary_modifier = if self.use_cmd_on_macos {
            modifiers.command()
        } else {
            modifiers.control()
        };

        match key {
            Key::Character(c) if primary_modifier && modifiers.alt() => {
                matches!(c.as_str(), "i" | "t" | "a" | "s" | "p")
            }
            Key::Named(iced::keyboard::key::Named::F1)
            | Key::Named(iced::keyboard::key::Named::F2)
            | Key::Named(iced::keyboard::key::Named::F3) => true,
            _ => false,
        }
    }
}

/// Keyboard shortcut hint display component
pub fn shortcut_hints(
    shortcuts: &KeyboardShortcuts,
    font_size: u16,
) -> iced::Element<'static, Message> {
    use iced::widget::{column, container, text};
    use iced::{Alignment, Color, Length};

    let help_text = shortcuts.get_help_text();

    container(
        column![
            text("Keyboard Shortcuts")
                .size(font_size + 2)
                .style(Color::from_rgb(0.9, 0.9, 1.0)),
            text(help_text)
                .size(font_size - 2)
                .style(Color::from_rgb(0.7, 0.7, 0.8))
        ]
        .spacing(8)
        .align_items(Alignment::Start),
    )
    .padding(16)
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_shortcuts_creation() {
        let shortcuts = KeyboardShortcuts::default();
        assert_eq!(shortcuts.use_cmd_on_macos, cfg!(target_os = "macos"));
    }

    #[test]
    fn test_sync_shortcut_detection() {
        let shortcuts = KeyboardShortcuts::default();

        // Test sync shortcuts
        let modifiers = Modifiers::CTRL | Modifiers::ALT;
        assert!(shortcuts.is_sync_shortcut(&Key::Character("i".into()), &modifiers));
        assert!(shortcuts.is_sync_shortcut(&Key::Character("t".into()), &modifiers));
        assert!(shortcuts.is_sync_shortcut(&Key::Character("a".into()), &modifiers));

        // Test function keys
        assert!(shortcuts.is_sync_shortcut(
            &Key::Named(iced::keyboard::key::Named::F1),
            &Modifiers::empty()
        ));

        // Test non-sync shortcuts
        assert!(!shortcuts.is_sync_shortcut(&Key::Character("r".into()), &Modifiers::CTRL));
    }

    #[test]
    fn test_help_text_generation() {
        let shortcuts = KeyboardShortcuts::default();
        let help_text = shortcuts.get_help_text();

        assert!(help_text.contains("Toggle synchronization mode"));
        assert!(help_text.contains("Enable current tab sync"));
        assert!(help_text.contains("Stop synchronization"));
    }
}
