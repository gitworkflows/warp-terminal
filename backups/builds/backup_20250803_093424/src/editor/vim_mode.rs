use iced::keyboard::{Key, Modifiers};
use iced::keyboard::key::Named;
use std::collections::HashMap;
use crate::editor::EditorAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Debug, Clone)]
pub struct VimModeManager {
    current_mode: VimMode,
    keybindings: HashMap<VimMode, HashMap<Key, VimCommand>>,
    pending_keys: Vec<Key>,
    repeat_count: Option<usize>,
    last_command: Option<VimCommand>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub enum VimCommand {
    // Navigation
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToDocumentStart,
    MoveToDocumentEnd,
    
    // Mode changes
    EnterInsertMode,
    EnterInsertModeAfterCursor,
    EnterInsertModeAtLineStart,
    EnterInsertModeAtLineEnd,
    EnterVisualMode,
    EnterCommandMode,
    ExitToNormal,
    
    // Editing
    DeleteChar,
    DeleteLine,
    DeleteWord,
    YankLine,
    YankWord,
    Paste,
    PasteBefore,
    Undo,
    Redo,
    
    // Search
    SearchForward,
    SearchBackward,
    SearchNext,
    SearchPrevious,
    
    // Text objects
    SelectInnerWord,
    SelectAroundWord,
    SelectInnerParagraph,
    SelectAroundParagraph,
    
    // Commands
    Save,
    Quit,
    ForceQuit,
    
    // Editor actions
    EditorAction(EditorAction),
}

impl Default for VimModeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VimModeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_mode: VimMode::Normal,
            keybindings: HashMap::new(),
            pending_keys: Vec::new(),
            repeat_count: None,
            last_command: None,
            enabled: false,
        };
        
        manager.load_default_keybindings();
        manager
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
        self.current_mode = VimMode::Normal;
        self.pending_keys.clear();
        self.repeat_count = None;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn current_mode(&self) -> &VimMode {
        &self.current_mode
    }
    
    pub fn handle_key_press(&mut self, key: &Key, _modifiers: &Modifiers) -> Option<EditorAction> {
        if !self.enabled {
            return None;
        }
        
        // Handle numeric prefixes for repeat counts
        if let Key::Character(c) = key {
            if let Ok(digit) = c.parse::<usize>() {
                if self.current_mode == VimMode::Normal && digit > 0 {
                    self.repeat_count = Some(self.repeat_count.unwrap_or(0) * 10 + digit);
                    return None;
                }
            }
        }
        
        // Get command for current mode and key
        if let Some(mode_bindings) = self.keybindings.get(&self.current_mode) {
            if let Some(command) = mode_bindings.get(key) {
                return self.execute_command(command.clone());
            }
        }
        
        // Reset repeat count if no valid command
        self.repeat_count = None;
        None
    }
    
    fn execute_command(&mut self, command: VimCommand) -> Option<EditorAction> {
        let repeat_count = self.repeat_count.unwrap_or(1);
        self.repeat_count = None;
        self.last_command = Some(command.clone());
        
        match command {
            // Mode changes
            VimCommand::EnterInsertMode => {
                self.current_mode = VimMode::Insert;
                None
            },
            VimCommand::EnterInsertModeAfterCursor => {
                self.current_mode = VimMode::Insert;
                Some(EditorAction::MoveCursor(1))
            },
            VimCommand::EnterInsertModeAtLineStart => {
                self.current_mode = VimMode::Insert;
                Some(EditorAction::MoveCursor(0)) // Move to line start
            },
            VimCommand::EnterInsertModeAtLineEnd => {
                self.current_mode = VimMode::Insert;
                Some(EditorAction::MoveCursor(usize::MAX)) // Move to line end
            },
            VimCommand::EnterVisualMode => {
                self.current_mode = VimMode::Visual;
                None
            },
            VimCommand::EnterCommandMode => {
                self.current_mode = VimMode::Command;
                None
            },
            VimCommand::ExitToNormal => {
                self.current_mode = VimMode::Normal;
                None
            },
            
            // Navigation
            VimCommand::MoveLeft => Some(EditorAction::MoveCursor(0)), // Will be handled specially
            VimCommand::MoveRight => Some(EditorAction::MoveCursor(1)),
            VimCommand::MoveUp => Some(EditorAction::MoveCursor(0)), // Line up
            VimCommand::MoveDown => Some(EditorAction::MoveCursor(0)), // Line down
            VimCommand::MoveWordForward => Some(EditorAction::MoveCursor(0)), // Next word
            VimCommand::MoveWordBackward => Some(EditorAction::MoveCursor(0)), // Previous word
            VimCommand::MoveToLineStart => Some(EditorAction::MoveCursor(0)),
            VimCommand::MoveToLineEnd => Some(EditorAction::MoveCursor(usize::MAX)),
            VimCommand::MoveToDocumentStart => Some(EditorAction::MoveCursor(0)),
            VimCommand::MoveToDocumentEnd => Some(EditorAction::MoveCursor(usize::MAX)),
            
            // Editing
            VimCommand::DeleteChar => Some(EditorAction::Delete(0, repeat_count)),
            VimCommand::DeleteLine => Some(EditorAction::DeleteLine(0)),
            VimCommand::DeleteWord => Some(EditorAction::Delete(0, 0)), // Delete word
            VimCommand::YankLine => Some(EditorAction::CopyLine(0)),
            VimCommand::YankWord => Some(EditorAction::CopySelection),
            VimCommand::Paste => Some(EditorAction::Paste),
            VimCommand::PasteBefore => Some(EditorAction::Paste), // Paste before cursor
            VimCommand::Undo => Some(EditorAction::Undo),
            VimCommand::Redo => Some(EditorAction::Redo),
            
            // Search
            VimCommand::SearchForward => Some(EditorAction::Find(String::new())),
            VimCommand::SearchBackward => Some(EditorAction::Find(String::new())),
            VimCommand::SearchNext => Some(EditorAction::FindNext),
            VimCommand::SearchPrevious => Some(EditorAction::FindPrevious),
            
            // Commands
            VimCommand::Save => Some(EditorAction::SaveSnapshot), // Will be handled specially
            VimCommand::Quit => Some(EditorAction::SaveSnapshot), // Will be handled specially
            VimCommand::ForceQuit => Some(EditorAction::SaveSnapshot), // Will be handled specially
            
            // Editor actions passthrough
            VimCommand::EditorAction(action) => Some(action),
            
            _ => None,
        }
    }
    
    fn load_default_keybindings(&mut self) {
        // Normal mode keybindings
        let mut normal_bindings = HashMap::new();
        
        // Navigation
        normal_bindings.insert(Key::Character("h".into()), VimCommand::MoveLeft);
        normal_bindings.insert(Key::Character("j".into()), VimCommand::MoveDown);
        normal_bindings.insert(Key::Character("k".into()), VimCommand::MoveUp);
        normal_bindings.insert(Key::Character("l".into()), VimCommand::MoveRight);
        normal_bindings.insert(Key::Character("w".into()), VimCommand::MoveWordForward);
        normal_bindings.insert(Key::Character("b".into()), VimCommand::MoveWordBackward);
        normal_bindings.insert(Key::Character("0".into()), VimCommand::MoveToLineStart);
        normal_bindings.insert(Key::Character("$".into()), VimCommand::MoveToLineEnd);
        normal_bindings.insert(Key::Character("g".into()), VimCommand::MoveToDocumentStart); // gg
        normal_bindings.insert(Key::Character("G".into()), VimCommand::MoveToDocumentEnd);
        
        // Mode changes
        normal_bindings.insert(Key::Character("i".into()), VimCommand::EnterInsertMode);
        normal_bindings.insert(Key::Character("a".into()), VimCommand::EnterInsertModeAfterCursor);
        normal_bindings.insert(Key::Character("I".into()), VimCommand::EnterInsertModeAtLineStart);
        normal_bindings.insert(Key::Character("A".into()), VimCommand::EnterInsertModeAtLineEnd);
        normal_bindings.insert(Key::Character("v".into()), VimCommand::EnterVisualMode);
        normal_bindings.insert(Key::Character(":".into()), VimCommand::EnterCommandMode);
        
        // Editing
        normal_bindings.insert(Key::Character("x".into()), VimCommand::DeleteChar);
        normal_bindings.insert(Key::Character("d".into()), VimCommand::DeleteLine); // dd
        normal_bindings.insert(Key::Character("y".into()), VimCommand::YankLine); // yy
        normal_bindings.insert(Key::Character("p".into()), VimCommand::Paste);
        normal_bindings.insert(Key::Character("P".into()), VimCommand::PasteBefore);
        normal_bindings.insert(Key::Character("u".into()), VimCommand::Undo);
        normal_bindings.insert(Key::Character("r".into()), VimCommand::Redo); // Ctrl+R normally
        
        // Search
        normal_bindings.insert(Key::Character("/".into()), VimCommand::SearchForward);
        normal_bindings.insert(Key::Character("?".into()), VimCommand::SearchBackward);
        normal_bindings.insert(Key::Character("n".into()), VimCommand::SearchNext);
        normal_bindings.insert(Key::Character("N".into()), VimCommand::SearchPrevious);
        
        self.keybindings.insert(VimMode::Normal, normal_bindings);
        
        // Insert mode keybindings
        let mut insert_bindings = HashMap::new();
        insert_bindings.insert(Key::Named(Named::Escape), VimCommand::ExitToNormal);
        self.keybindings.insert(VimMode::Insert, insert_bindings);
        
        // Visual mode keybindings
        let mut visual_bindings = HashMap::new();
        visual_bindings.insert(Key::Named(Named::Escape), VimCommand::ExitToNormal);
        visual_bindings.insert(Key::Character("d".into()), VimCommand::DeleteChar);
        visual_bindings.insert(Key::Character("y".into()), VimCommand::YankWord);
        visual_bindings.insert(Key::Character("x".into()), VimCommand::DeleteChar);
        self.keybindings.insert(VimMode::Visual, visual_bindings);
        
        // Command mode keybindings
        let mut command_bindings = HashMap::new();
        command_bindings.insert(Key::Named(Named::Escape), VimCommand::ExitToNormal);
        command_bindings.insert(Key::Named(Named::Enter), VimCommand::ExitToNormal);
        self.keybindings.insert(VimMode::Command, command_bindings);
    }
    
    pub fn get_mode_display(&self) -> &str {
        match self.current_mode {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Visual => "VISUAL",
            VimMode::Command => "COMMAND",
        }
    }
    
    pub fn add_custom_keybinding(&mut self, mode: VimMode, key: Key, command: VimCommand) {
        if let Some(mode_bindings) = self.keybindings.get_mut(&mode) {
            mode_bindings.insert(key, command);
        }
    }
    
    pub fn remove_keybinding(&mut self, mode: VimMode, key: &Key) {
        if let Some(mode_bindings) = self.keybindings.get_mut(&mode) {
            mode_bindings.remove(key);
        }
    }
    
    pub fn repeat_last_command(&mut self) -> Option<EditorAction> {
        if let Some(command) = self.last_command.clone() {
            self.execute_command(command)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_mode_creation() {
        let manager = VimModeManager::new();
        assert_eq!(manager.current_mode(), &VimMode::Normal);
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_mode_switching() {
        let mut manager = VimModeManager::new();
        manager.enable();
        
        // Enter insert mode
        let action = manager.handle_key_press(&Key::Character("i".into()), &Modifiers::empty());
        assert_eq!(manager.current_mode(), &VimMode::Insert);
        
        // Exit to normal mode
        let action = manager.handle_key_press(&Key::Named(Named::Escape), &Modifiers::empty());
        assert_eq!(manager.current_mode(), &VimMode::Normal);
    }

    #[test]
    fn test_repeat_count() {
        let mut manager = VimModeManager::new();
        manager.enable();
        
        // Enter repeat count
        manager.handle_key_press(&Key::Character("3".into()), &Modifiers::empty());
        
        // Execute delete command
        let action = manager.handle_key_press(&Key::Character("x".into()), &Modifiers::empty());
        assert!(action.is_some());
    }
}
