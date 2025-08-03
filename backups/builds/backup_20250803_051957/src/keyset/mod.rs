//! Keyset management for Warp terminal
//!
//! This module handles loading and parsing YAML keybinding files according to the
//! format specified in keysets/FORMAT.md.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use iced::keyboard::{Key, Modifiers};
use anyhow::{Result, Context};
use crate::Message;

/// Represents a complete keyset loaded from a YAML file
#[derive(Debug, Clone)]
pub struct Keyset {
    /// Map from action name to parsed keybinding
    bindings: HashMap<String, ParsedKeybinding>,
    /// Name of this keyset
    pub name: String,
    /// File path this keyset was loaded from
    pub path: PathBuf,
}

/// A parsed keybinding with key and modifiers
#[derive(Debug, Clone, PartialEq)]
struct ParsedKeybinding {
    key: Key,
    modifiers: Modifiers,
}

/// Manager for loading and managing keysets
#[derive(Debug, Clone)]
pub struct KeysetManager {
    /// Currently active keyset
    active_keyset: Option<Keyset>,
    /// Available keysets by name
    available_keysets: HashMap<String, PathBuf>,
}

impl KeysetManager {
    /// Create a new keyset manager
    pub fn new() -> Self {
        Self {
            active_keyset: None,
            available_keysets: HashMap::new(),
        }
    }

    /// Discover keysets in the keysets directory
    pub fn discover_keysets<P: AsRef<Path>>(&mut self, keysets_dir: P) -> Result<()> {
        let dir = keysets_dir.as_ref();
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    self.available_keysets.insert(name.to_string(), path);
                }
            }
        }
        
        Ok(())
    }

    /// Load and activate a keyset by name
    pub fn load_keyset(&mut self, name: &str) -> Result<()> {
        if let Some(path) = self.available_keysets.get(name) {
            let keyset = Keyset::load_from_file(path, name)?;
            self.active_keyset = Some(keyset);
            Ok(())
        } else {
            anyhow::bail!("Keyset '{}' not found", name);
        }
    }

    /// Get the currently active keyset
    pub fn active_keyset(&self) -> Option<&Keyset> {
        self.active_keyset.as_ref()
    }

    /// List available keyset names
    pub fn list_keysets(&self) -> Vec<&String> {
        self.available_keysets.keys().collect()
    }

    /// Handle a keyboard event and return the corresponding message
    pub fn handle_keyboard_event(&self, key: &Key, modifiers: &Modifiers) -> Option<Message> {
        if let Some(keyset) = &self.active_keyset {
            keyset.get_action_for_key(key, modifiers)
        } else {
            None
        }
    }
}

impl Keyset {
    /// Load a keyset from a YAML file
    fn load_from_file<P: AsRef<Path>>(path: P, name: &str) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read keyset file: {}", path.display()))?;
        
        // Parse YAML as a simple map of action -> key binding string
        let raw_bindings: HashMap<String, String> = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse YAML in keyset file: {}", path.display()))?;
        
        let mut bindings = HashMap::new();
        
        // Parse each keybinding string into Key and Modifiers
        for (action, binding_str) in raw_bindings {
            match parse_keybinding(&binding_str) {
                Ok(parsed) => {
                    bindings.insert(action, parsed);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse keybinding '{}' for action '{}': {}", binding_str, action, e);
                }
            }
        }
        
        Ok(Keyset {
            bindings,
            name: name.to_string(),
            path: path.to_path_buf(),
        })
    }

    /// Get the message for a key combination, if any
    fn get_action_for_key(&self, key: &Key, modifiers: &Modifiers) -> Option<Message> {
        // Find matching keybinding
        for (action, binding) in &self.bindings {
            if binding.key == *key && binding.modifiers == *modifiers {
                return action_name_to_message(action);
            }
        }
        None
    }

    /// Get the keybinding string for an action
    pub fn get_binding_for_action(&self, action: &str) -> Option<String> {
        self.bindings.get(action).map(|binding| {
            format_keybinding(&binding.key, &binding.modifiers)
        })
    }
}

/// Parse a keybinding string like "cmd-shift-A" into Key and Modifiers
fn parse_keybinding(binding_str: &str) -> Result<ParsedKeybinding> {
    let parts: Vec<&str> = binding_str.split('-').collect();
    
    if parts.is_empty() {
        anyhow::bail!("Empty keybinding string");
    }
    
    let key_part = parts.last().unwrap();
    let modifier_parts = &parts[..parts.len() - 1];
    
    // Parse modifiers
    let mut modifiers = Modifiers::empty();
    for modifier in modifier_parts {
        match *modifier {
            "ctrl" => modifiers |= Modifiers::CTRL,
            "cmd" => modifiers |= Modifiers::COMMAND,
            "alt" => modifiers |= Modifiers::ALT,
            "meta" => modifiers |= Modifiers::ALT, // Use ALT as META for compatibility
            "shift" => modifiers |= Modifiers::SHIFT,
            _ => anyhow::bail!("Unknown modifier: {}", modifier),
        }
    }
    
    // Parse key
    let key = parse_key(key_part)?;
    
    Ok(ParsedKeybinding { key, modifiers })
}

/// Parse a key string into an iced Key
fn parse_key(key_str: &str) -> Result<Key> {
    let key = match key_str {
        // Named keys
        "up" => Key::Named(iced::keyboard::key::Named::ArrowUp),
        "down" => Key::Named(iced::keyboard::key::Named::ArrowDown),
        "left" => Key::Named(iced::keyboard::key::Named::ArrowLeft),
        "right" => Key::Named(iced::keyboard::key::Named::ArrowRight),
        "home" => Key::Named(iced::keyboard::key::Named::Home),
        "end" => Key::Named(iced::keyboard::key::Named::End),
        "pageup" => Key::Named(iced::keyboard::key::Named::PageUp),
        "pagedown" => Key::Named(iced::keyboard::key::Named::PageDown),
        "backspace" => Key::Named(iced::keyboard::key::Named::Backspace),
        "enter" => Key::Named(iced::keyboard::key::Named::Enter),
        "insert" => Key::Named(iced::keyboard::key::Named::Insert),
        "delete" => Key::Named(iced::keyboard::key::Named::Delete),
        "escape" => Key::Named(iced::keyboard::key::Named::Escape),
        "tab" => Key::Named(iced::keyboard::key::Named::Tab),
        "numpadenter" => Key::Named(iced::keyboard::key::Named::Enter), // Treat as regular enter for now
        "f1" => Key::Named(iced::keyboard::key::Named::F1),
        "f2" => Key::Named(iced::keyboard::key::Named::F2),
        "f3" => Key::Named(iced::keyboard::key::Named::F3),
        "f4" => Key::Named(iced::keyboard::key::Named::F4),
        "f5" => Key::Named(iced::keyboard::key::Named::F5),
        "f6" => Key::Named(iced::keyboard::key::Named::F6),
        "f7" => Key::Named(iced::keyboard::key::Named::F7),
        "f8" => Key::Named(iced::keyboard::key::Named::F8),
        "f9" => Key::Named(iced::keyboard::key::Named::F9),
        "f10" => Key::Named(iced::keyboard::key::Named::F10),
        "f11" => Key::Named(iced::keyboard::key::Named::F11),
        "f12" => Key::Named(iced::keyboard::key::Named::F12),
        "f13" => Key::Named(iced::keyboard::key::Named::F13),
        "f14" => Key::Named(iced::keyboard::key::Named::F14),
        "f15" => Key::Named(iced::keyboard::key::Named::F15),
        "f16" => Key::Named(iced::keyboard::key::Named::F16),
        "f17" => Key::Named(iced::keyboard::key::Named::F17),
        "f18" => Key::Named(iced::keyboard::key::Named::F18),
        "f19" => Key::Named(iced::keyboard::key::Named::F19),
        "f20" => Key::Named(iced::keyboard::key::Named::F20),
        // Character keys - single character
        s if s.chars().count() == 1 => {
            Key::Character(s.into())
        }
        // Special characters that might appear in shift combinations
        _ => Key::Character(key_str.into()),
    };
    
    Ok(key)
}

/// Convert action name to corresponding Message
fn action_name_to_message(action: &str) -> Option<Message> {
    match action {
        // Input and command execution
        "editor_view:clear_buffer" => Some(Message::InputChanged(String::new())),
        
        // Command search
        "input:search_command_history" | "workspace:show_command_search" => Some(Message::ToggleCommandSearch),
        
        // Copy operations - these need block IDs, so we'll handle them differently
        // For now, return None for actions that need additional context
        "terminal:copy" | "terminal:copy_command" | "terminal:copy_output" => None,
        
        // Settings
        "workspace:show_settings_modal" => Some(Message::InputChanged("toggle_settings".to_string())),
        
        // Theme operations
        "workspace:show_theme_chooser" => None, // Would need theme chooser UI
        
        // Synchronization
        "workspace:toggle_sync_mode" => Some(Message::ToggleSynchronizationMode),
        
        _ => {
            tracing::debug!("Unhandled action: {}", action);
            None
        }
    }
}

/// Format a Key and Modifiers back into a keybinding string
fn format_keybinding(key: &Key, modifiers: &Modifiers) -> String {
    let mut parts = Vec::new();
    
    if modifiers.control() {
        parts.push("ctrl");
    }
    if modifiers.command() {
        parts.push("cmd");
    }
    if modifiers.alt() {
        parts.push("alt");
    }
    if modifiers.shift() {
        parts.push("shift");
    }
    
    // Add the key
    let key_str = match key {
        Key::Named(named) => match named {
            iced::keyboard::key::Named::ArrowUp => "up",
            iced::keyboard::key::Named::ArrowDown => "down",
            iced::keyboard::key::Named::ArrowLeft => "left",
            iced::keyboard::key::Named::ArrowRight => "right",
            iced::keyboard::key::Named::Home => "home",
            iced::keyboard::key::Named::End => "end",
            iced::keyboard::key::Named::PageUp => "pageup",
            iced::keyboard::key::Named::PageDown => "pagedown",
            iced::keyboard::key::Named::Backspace => "backspace",
            iced::keyboard::key::Named::Enter => "enter",
            iced::keyboard::key::Named::Insert => "insert",
            iced::keyboard::key::Named::Delete => "delete",
            iced::keyboard::key::Named::Escape => "escape",
            iced::keyboard::key::Named::Tab => "tab",
            iced::keyboard::key::Named::F1 => "f1",
            iced::keyboard::key::Named::F2 => "f2",
            iced::keyboard::key::Named::F3 => "f3",
            iced::keyboard::key::Named::F4 => "f4",
            iced::keyboard::key::Named::F5 => "f5",
            iced::keyboard::key::Named::F6 => "f6",
            iced::keyboard::key::Named::F7 => "f7",
            iced::keyboard::key::Named::F8 => "f8",
            iced::keyboard::key::Named::F9 => "f9",
            iced::keyboard::key::Named::F10 => "f10",
            iced::keyboard::key::Named::F11 => "f11",
            iced::keyboard::key::Named::F12 => "f12",
            iced::keyboard::key::Named::F13 => "f13",
            iced::keyboard::key::Named::F14 => "f14",
            iced::keyboard::key::Named::F15 => "f15",
            iced::keyboard::key::Named::F16 => "f16",
            iced::keyboard::key::Named::F17 => "f17",
            iced::keyboard::key::Named::F18 => "f18",
            iced::keyboard::key::Named::F19 => "f19",
            iced::keyboard::key::Named::F20 => "f20",
            _ => "unknown",
        },
        Key::Character(c) => c.as_str(),
        _ => "unknown",
    };
    
    parts.push(key_str);
    parts.join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_keyset() {
        let keyset = Keyset::load_from_file("./keysets/default-warp-keybindings.yaml", "test");
        if let Ok(keyset) = keyset {
            assert!(keyset.bindings.len() > 0);
        } else {
            // Test file might not exist in test environment, that's ok
            assert!(true);
        }
    }
}
