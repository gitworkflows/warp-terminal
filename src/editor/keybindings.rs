use iced::keyboard::{Key, Modifiers};
use iced::keyboard::key::Named;
use std::collections::HashMap;
use crate::editor::EditorAction;

#[derive(Debug, Clone)]
pub struct KeybindingManager {
    bindings: HashMap<KeyBinding, EditorKeybinding>,
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: Key,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone)]
pub struct EditorKeybinding {
    pub action: EditorAction,
    pub description: String,
    pub enabled: bool,
}

impl Default for KeybindingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeybindingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            enabled: true,
        };
        
        manager.load_default_keybindings();
        manager
    }

    pub fn handle_key_press(&self, key: &Key, modifiers: &Modifiers) -> Option<EditorAction> {
        if !self.enabled {
            return None;
        }

        let binding = KeyBinding { key: key.clone(), modifiers: *modifiers };
        self.bindings.get(&binding).map(|kb| kb.action.clone())
    }

    pub fn add_keybinding(&mut self, key: Key, modifiers: Modifiers, action: EditorAction, description: String) {
        let binding = KeyBinding { key, modifiers };
        self.bindings.insert(binding, EditorKeybinding {
            action,
            description,
            enabled: true,
        });
    }

    pub fn remove_keybinding(&mut self, key: Key, modifiers: Modifiers) {
        let binding = KeyBinding { key, modifiers };
        self.bindings.remove(&binding);
    }

    pub fn get_all_keybindings(&self) -> Vec<(KeyBinding, &EditorKeybinding)> {
        self.bindings.iter().map(|(k, v)| (k.clone(), v)).collect()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn load_default_keybindings(&mut self) {
        // Basic editing
        self.add_keybinding(
            Key::Character("z".into()),
            Modifiers::CTRL,
            EditorAction::Undo,
            "Undo last action".to_string(),
        );

        self.add_keybinding(
            Key::Character("y".into()),
            Modifiers::CTRL,
            EditorAction::Redo,
            "Redo last undone action".to_string(),
        );

        self.add_keybinding(
            Key::Character("z".into()),
            Modifiers::CTRL | Modifiers::SHIFT,
            EditorAction::Redo,
            "Redo last undone action".to_string(),
        );

        // Selection
self.add_keybinding(
            Key::Character("a".into()),
            Modifiers::CTRL,
            EditorAction::SELECT_ALL,
            "Select all text".to_string(),
        );

        // Copy/Cut/Paste would be handled by the system, but we can add custom actions
self.add_keybinding(
            Key::Character("x".into()),
            Modifiers::CTRL,
            EditorAction::CUT,
            "Cut selected text".to_string(),
        );

        self.add_keybinding(
            Key::Character("c".into()),
            Modifiers::CTRL,
            EditorAction::Copy,
            "Copy selected text".to_string(),
        );

        self.add_keybinding(
            Key::Character("v".into()),
            Modifiers::CTRL,
            EditorAction::Paste,
            "Paste text from clipboard".to_string(),
        );

        // Navigation
self.add_keybinding(
            Key::Named(Named::Home),
            Modifiers::empty(),
            EditorAction::MOVE_TO_LINE_START,
            "Move cursor to beginning of line".to_string(),
        );

self.add_keybinding(
            Key::Named(Named::End),
            Modifiers::empty(),
            EditorAction::MOVE_TO_LINE_END,
            "Move cursor to end of line".to_string(),
        );

self.add_keybinding(
            Key::Named(Named::Home),
            Modifiers::CTRL,
            EditorAction::MOVE_TO_DOCUMENT_START,
            "Move cursor to beginning of document".to_string(),
        );

self.add_keybinding(
            Key::Named(Named::End),
            Modifiers::CTRL,
            EditorAction::MOVE_TO_DOCUMENT_END,
            "Move cursor to end of document".to_string(),
        );

        // Line operations
        self.add_keybinding(
            Key::Named(Named::Enter),
            Modifiers::SHIFT,
            EditorAction::InsertNewLine,
            "Insert new line".to_string(),
        );

self.add_keybinding(
            Key::Character("d".into()),
            Modifiers::CTRL,
            EditorAction::DUPLICATE_CURRENT_LINE,
            "Duplicate current line".to_string(),
        );

self.add_keybinding(
            Key::Character("k".into()),
            Modifiers::CTRL,
            EditorAction::DELETE_CURRENT_LINE,
            "Delete current line".to_string(),
        );

        // Indentation
self.add_keybinding(
            Key::Named(Named::Tab),
            Modifiers::empty(),
            EditorAction::INDENT_SELECTION,
            "Indent selected text".to_string(),
        );

self.add_keybinding(
            Key::Named(Named::Tab),
            Modifiers::SHIFT,
            EditorAction::UNINDENT_SELECTION,
            "Unindent selected text".to_string(),
        );

        // Search
        self.add_keybinding(
            Key::Character("f".into()),
            Modifiers::CTRL,
            EditorAction::show_search(),
            "Show search dialog".to_string(),
        );

        self.add_keybinding(
            Key::Character("h".into()),
            Modifiers::CTRL,
            EditorAction::show_replace(),
            "Show find and replace dialog".to_string(),
        );

self.add_keybinding(
            Key::Character("g".into()),
            Modifiers::CTRL,
            EditorAction::GO_TO_LINE,
            "Go to specific line".to_string(),
        );

        // Multiple cursors
self.add_keybinding(
            Key::Character("d".into()),
            Modifiers::ALT,
            EditorAction::ADD_CURSOR_BELOW,
            "Add cursor on line below".to_string(),
        );

self.add_keybinding(
            Key::Character("u".into()),
            Modifiers::ALT,
            EditorAction::ADD_CURSOR_ABOVE,
            "Add cursor on line above".to_string(),
        );

        // Word operations
self.add_keybinding(
            Key::Named(Named::ArrowLeft),
            Modifiers::CTRL,
            EditorAction::MOVE_TO_PREVIOUS_WORD,
            "Move to previous word".to_string(),
        );

self.add_keybinding(
            Key::Named(Named::ArrowRight),
            Modifiers::CTRL,
            EditorAction::MOVE_TO_NEXT_WORD,
            "Move to next word".to_string(),
        );

self.add_keybinding(
            Key::Named(Named::Backspace),
            Modifiers::CTRL,
            EditorAction::DELETE_PREVIOUS_WORD,
            "Delete previous word".to_string(),
        );

self.add_keybinding(
            Key::Named(Named::Delete),
            Modifiers::CTRL,
            EditorAction::DELETE_NEXT_WORD,
            "Delete next word".to_string(),
        );

        // Toggle features
self.add_keybinding(
            Key::Character("/".into()),
            Modifiers::CTRL,
            EditorAction::TOGGLE_COMMENT,
            "Toggle line comment".to_string(),
        );

self.add_keybinding(
            Key::Character("l".into()),
            Modifiers::CTRL,
            EditorAction::TOGGLE_LINE_NUMBERS,
            "Toggle line numbers".to_string(),
        );

        // Save (even though this might be handled elsewhere)
self.add_keybinding(
            Key::Character("s".into()),
            Modifiers::CTRL,
            EditorAction::SAVE,
            "Save current content".to_string(),
        );

        // Format
self.add_keybinding(
            Key::Character("f".into()),
            Modifiers::ALT | Modifiers::SHIFT,
            EditorAction::FORMAT_DOCUMENT,
            "Format entire document".to_string(),
        );

        // Folding
self.add_keybinding(
            Key::Character("-".into()),
            Modifiers::CTRL,
            EditorAction::FOLD_CURRENT_REGION,
            "Fold current region".to_string(),
        );

self.add_keybinding(
            Key::Character("+".into()),
            Modifiers::CTRL,
            EditorAction::UNFOLD_CURRENT_REGION,
            "Unfold current region".to_string(),
        );
    }

    pub fn format_key_binding(&self, binding: &KeyBinding) -> String {
        let mut parts = Vec::new();

        if binding.modifiers.contains(Modifiers::CTRL) {
            parts.push("Ctrl");
        }
        if binding.modifiers.contains(Modifiers::ALT) {
            parts.push("Alt");
        }
        if binding.modifiers.contains(Modifiers::SHIFT) {
            parts.push("Shift");
        }
        if binding.modifiers.contains(Modifiers::LOGO) {
            parts.push("Cmd");
        }

        let key_name = match binding.key {
            Key::Character(ref c) => c.as_str(),
            Key::Named(named) => match named {
                Named::Enter => "Enter",
                Named::Escape => "Esc",
                Named::Backspace => "Backspace",
                Named::Tab => "Tab",
                Named::Space => "Space",
                Named::ArrowLeft => "Left",
                Named::ArrowUp => "Up",
                Named::ArrowRight => "Right",
                Named::ArrowDown => "Down",
                Named::Home => "Home",
                Named::End => "End",
                Named::PageUp => "PageUp",
                Named::PageDown => "PageDown",
                Named::Insert => "Insert",
                Named::Delete => "Delete",
                Named::F1 => "F1",
                Named::F2 => "F2",
                Named::F3 => "F3",
                Named::F4 => "F4",
                Named::F5 => "F5",
                Named::F6 => "F6",
                Named::F7 => "F7",
                Named::F8 => "F8",
                Named::F9 => "F9",
                Named::F10 => "F10",
                Named::F11 => "F11",
                Named::F12 => "F12",
                _ => "Unknown",
            },
            _ => "Unknown",
        };

        parts.push(key_name);
        parts.join("+")
    }
}

// Extended EditorAction enum to include additional actions for keybindings
impl EditorAction {
    // Navigation actions
    pub const MOVE_TO_LINE_START: EditorAction = EditorAction::MoveCursor(0); // Will be handled specially
    pub const MOVE_TO_LINE_END: EditorAction = EditorAction::MoveCursor(0); // Will be handled specially
    pub const MOVE_TO_DOCUMENT_START: EditorAction = EditorAction::MoveCursor(0);
    pub const MOVE_TO_DOCUMENT_END: EditorAction = EditorAction::MoveCursor(usize::MAX);
    
    // Line operations
    pub const DUPLICATE_CURRENT_LINE: EditorAction = EditorAction::DuplicateLine(0);
    pub const DELETE_CURRENT_LINE: EditorAction = EditorAction::DeleteLine(0);
    
    // Selection operations
    pub const SELECT_ALL: EditorAction = EditorAction::SetSelection(0, usize::MAX);
    
    // Clipboard operations (these would need to be handled by the UI layer)
    pub const CUT: EditorAction = EditorAction::SaveSnapshot; // Placeholder
    pub const COPY: EditorAction = EditorAction::SaveSnapshot; // Placeholder
    pub const PASTE: EditorAction = EditorAction::SaveSnapshot; // Placeholder
    
    // Indentation
    pub const INDENT_SELECTION: EditorAction = EditorAction::AutoIndent;
    pub const UNINDENT_SELECTION: EditorAction = EditorAction::AutoIndent; // Will be handled specially
    
    // Search operations - using functions instead of constants since we need String::new()
    pub fn show_search() -> EditorAction {
        EditorAction::Find(String::new())
    }
    
    pub fn show_replace() -> EditorAction {
        EditorAction::Replace(0, 0, String::new()) // Fixed arguments: start, length, replacement
    }
    pub const GO_TO_LINE: EditorAction = EditorAction::MoveCursor(0);
    
    // Multiple cursor operations
    pub const ADD_CURSOR_BELOW: EditorAction = EditorAction::AddCursor(0);
    pub const ADD_CURSOR_ABOVE: EditorAction = EditorAction::AddCursor(0);
    
    // Word operations
    pub const MOVE_TO_PREVIOUS_WORD: EditorAction = EditorAction::MoveCursor(0);
    pub const MOVE_TO_NEXT_WORD: EditorAction = EditorAction::MoveCursor(0);
    pub const DELETE_PREVIOUS_WORD: EditorAction = EditorAction::Delete(0, 0);
    pub const DELETE_NEXT_WORD: EditorAction = EditorAction::Delete(0, 0);
    
    // Toggle operations
    pub const TOGGLE_COMMENT: EditorAction = EditorAction::SaveSnapshot; // Placeholder
    pub const TOGGLE_LINE_NUMBERS: EditorAction = EditorAction::SaveSnapshot; // Placeholder
    
    // File operations
    pub const SAVE: EditorAction = EditorAction::SaveSnapshot;
    pub const FORMAT_DOCUMENT: EditorAction = EditorAction::SaveSnapshot; // Placeholder
    
    // Folding operations
    pub const FOLD_CURRENT_REGION: EditorAction = EditorAction::FoldRegion(0, 0);
    pub const UNFOLD_CURRENT_REGION: EditorAction = EditorAction::UnfoldRegion(0, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybinding_manager_creation() {
        let manager = KeybindingManager::new();
        assert!(manager.is_enabled());
        assert!(!manager.get_all_keybindings().is_empty());
    }

    #[test]
    fn test_handle_key_press() {
        let manager = KeybindingManager::new();
        
        // Test Ctrl+Z for undo
        let action = manager.handle_key_press(&Key::Character("z".into()), &Modifiers::CTRL);
        assert!(matches!(action, Some(EditorAction::Undo)));
        
        // Test unknown keybinding
        let action = manager.handle_key_press(&Key::Character("q".into()), &(Modifiers::ALT | Modifiers::SHIFT));
        assert!(action.is_none());
    }

    #[test]
    fn test_add_remove_keybinding() {
        let mut manager = KeybindingManager::new();
        
        // Add custom keybinding
        manager.add_keybinding(
            Key::Character("t".into()),
            Modifiers::CTRL | Modifiers::SHIFT,
            EditorAction::SaveSnapshot,
            "Test action".to_string(),
        );
        
        let action = manager.handle_key_press(&Key::Character("t".into()), &(Modifiers::CTRL | Modifiers::SHIFT));
        assert!(matches!(action, Some(EditorAction::SaveSnapshot)));
        
        // Remove keybinding
        manager.remove_keybinding(Key::Character("t".into()), Modifiers::CTRL | Modifiers::SHIFT);
        let action = manager.handle_key_press(&Key::Character("t".into()), &(Modifiers::CTRL | Modifiers::SHIFT));
        assert!(action.is_none());
    }

    #[test]
    fn test_format_key_binding() {
        let manager = KeybindingManager::new();
        
        let binding = KeyBinding {
            key: Key::Character("s".into()),
            modifiers: Modifiers::CTRL,
        };
        
        assert_eq!(manager.format_key_binding(&binding), "Ctrl+s");
        
        let binding = KeyBinding {
            key: Key::Character("f".into()),
            modifiers: Modifiers::CTRL | Modifiers::SHIFT,
        };
        
        assert_eq!(manager.format_key_binding(&binding), "Ctrl+Shift+f");
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = KeybindingManager::new();
        
        // Should work when enabled
        let action = manager.handle_key_press(&Key::Character("z".into()), &Modifiers::CTRL);
        assert!(action.is_some());
        
        // Disable and test
        manager.disable();
        let action = manager.handle_key_press(&Key::Character("z".into()), &Modifiers::CTRL);
        assert!(action.is_none());
        
        // Re-enable
        manager.enable();
        let action = manager.handle_key_press(&Key::Character("z".into()), &Modifiers::CTRL);
        assert!(action.is_some());
    }
}
