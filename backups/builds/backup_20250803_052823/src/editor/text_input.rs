use crate::editor::{EditorAction, EditorState};
use crate::editor::vim_mode::{VimModeManager, VimMode};
use crate::editor::syntax_highlighter::{SyntaxHighlighter, HighlightedSpan};
use crate::inspector::CommandInspector;
use iced::keyboard::{Key, Modifiers};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TextInputHandler {
    pub editor_state: EditorState,
    pub vim_manager: VimModeManager,
    pub syntax_highlighter: SyntaxHighlighter,
    pub command_inspector: CommandInspector,
    pub multi_cursors: Vec<usize>,
    pub auto_complete_enabled: bool,
    pub auto_indent_enabled: bool,
    pub bracket_matching_enabled: bool,
    pub word_wrap_enabled: bool,
    pub current_completions: Vec<String>,
    pub completion_index: Option<usize>,
}

impl Default for TextInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInputHandler {
    pub fn new() -> Self {
        Self {
            editor_state: EditorState::new(),
            vim_manager: VimModeManager::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            command_inspector: CommandInspector::new(),
            multi_cursors: Vec::new(),
            auto_complete_enabled: true,
            auto_indent_enabled: true,
            bracket_matching_enabled: true,
            word_wrap_enabled: false,
            current_completions: Vec::new(),
            completion_index: None,
        }
    }

    pub fn handle_key_input(&mut self, key: &Key, modifiers: &Modifiers) -> Vec<EditorAction> {
        let mut actions = Vec::new();

        // Handle Vim mode if enabled
        if self.vim_manager.is_enabled() {
            if let Some(vim_action) = self.vim_manager.handle_key_press(key, modifiers) {
                actions.push(vim_action);
            }
            
            // In insert mode, allow normal text input
            if self.vim_manager.current_mode() != &VimMode::Insert {
                return actions;
            }
        }

        // Handle special key combinations
        match (key, *modifiers) {
            // Auto-completion
            (Key::Named(iced::keyboard::key::Named::Tab), Modifiers::empty()) => {
                if !self.current_completions.is_empty() {
                    actions.push(self.apply_completion());
                } else {
                    actions.extend(self.trigger_auto_complete());
                }
            },

            // Multi-cursor operations
            (Key::Character(c), Modifiers::ALT) if c == "d" => {
                actions.push(EditorAction::AddCursor(self.editor_state.cursor_position));
            },

            // Bracket auto-completion
            (Key::Character(c), Modifiers::empty()) if self.is_opening_bracket(c) => {
                if self.bracket_matching_enabled {
                    actions.extend(self.handle_bracket_input(c));
                } else {
                    actions.push(EditorAction::Insert(c.to_string()));
                }
            },

            // Regular character input
            (Key::Character(c), _) => {
                actions.push(EditorAction::Insert(c.to_string()));
                
                // Trigger real-time validation
                if self.command_inspector.enabled {
                    actions.extend(self.validate_current_line());
                }
            },

            // Enter key - smart indentation
            (Key::Named(iced::keyboard::key::Named::Enter), _) => {
                if self.auto_indent_enabled {
                    actions.extend(self.smart_new_line());
                } else {
                    actions.push(EditorAction::InsertNewLine);
                }
            },

            // Backspace with smart deletion
            (Key::Named(iced::keyboard::key::Named::Backspace), Modifiers::empty()) => {
                actions.extend(self.smart_backspace());
            },

            // Word-wise deletion
            (Key::Named(iced::keyboard::key::Named::Backspace), Modifiers::CTRL) => {
                actions.push(EditorAction::Delete(
                    self.find_word_start(),
                    self.editor_state.cursor_position - self.find_word_start()
                ));
            },

            _ => {}
        }

        // Update multi-cursors if any exist
        if !self.multi_cursors.is_empty() {
            actions.extend(self.apply_to_multi_cursors(&actions.clone()));
        }

        actions
    }

    pub fn get_highlighted_content(&self) -> Vec<HighlightedSpan> {
        if self.editor_state.syntax_highlighting_enabled {
            self.syntax_highlighter.highlight_command_line(&self.editor_state.content)
        } else {
            Vec::new()
        }
    }

    pub fn set_vim_mode(&mut self, enabled: bool) {
        if enabled {
            self.vim_manager.enable();
        } else {
            self.vim_manager.disable();
        }
    }

    pub fn is_vim_enabled(&self) -> bool {
        self.vim_manager.is_enabled()
    }

    pub fn get_current_vim_mode(&self) -> String {
        if self.vim_manager.is_enabled() {
            self.vim_manager.get_mode_display().to_string()
        } else {
            "NORMAL".to_string()
        }
    }

    // Private helper methods

    fn is_opening_bracket(&self, c: &str) -> bool {
        matches!(c, "(" | "{" | "[" | "\"" | "'")
    }

    fn get_closing_bracket(&self, c: &str) -> &str {
        match c {
            "(" => ")",
            "{" => "}",
            "[" => "]",
            "\"" => "\"",
            "'" => "'",
            _ => "",
        }
    }

    fn handle_bracket_input(&mut self, bracket: &str) -> Vec<EditorAction> {
        let mut actions = Vec::new();
        let closing = self.get_closing_bracket(bracket);
        
        // Insert opening bracket
        actions.push(EditorAction::Insert(bracket.to_string()));
        
        // Insert closing bracket if not a quote or if quote is not already closed
        if bracket != "\"" && bracket != "'" || !self.is_quote_already_closed(bracket) {
            actions.push(EditorAction::Insert(closing.to_string()));
            // Move cursor back between brackets
            actions.push(EditorAction::MoveCursor(self.editor_state.cursor_position));
        }
        
        actions
    }

    fn is_quote_already_closed(&self, quote: &str) -> bool {
        let content = &self.editor_state.content;
        let before_cursor = &content[..self.editor_state.cursor_position];
        let after_cursor = &content[self.editor_state.cursor_position..];
        
        let quotes_before = before_cursor.matches(quote).count();
        let quotes_after = after_cursor.matches(quote).count();
        
        // If there's an odd number of quotes before cursor and at least one after,
        // we're likely inside a quote
        quotes_before % 2 == 1 && quotes_after > 0
    }

    fn smart_new_line(&mut self) -> Vec<EditorAction> {
        let mut actions = Vec::new();
        actions.push(EditorAction::InsertNewLine);
        
        // Calculate indentation based on previous line
        let current_line_text = self.editor_state.get_current_line_text();
        let indentation = self.calculate_indentation(current_line_text);
        
        if !indentation.is_empty() {
            actions.push(EditorAction::Insert(indentation));
        }
        
        actions
    }

    fn calculate_indentation(&self, line: &str) -> String {
        let mut indent = String::new();
        
        // Copy existing indentation
        for ch in line.chars() {
            if ch == ' ' || ch == '\t' {
                indent.push(ch);
            } else {
                break;
            }
        }
        
        // Add extra indentation for certain patterns
        let trimmed = line.trim();
        if trimmed.ends_with('{') || trimmed.ends_with(':') || trimmed.ends_with('(') {
            indent.push_str("    "); // Add 4 spaces
        }
        
        indent
    }

    fn smart_backspace(&mut self) -> Vec<EditorAction> {
        let mut actions = Vec::new();
        
        // Check if we're deleting a bracket pair
        if self.bracket_matching_enabled && self.editor_state.cursor_position > 0 {
            let content = &self.editor_state.content;
            let before_cursor = content.chars().nth(self.editor_state.cursor_position - 1);
            let at_cursor = content.chars().nth(self.editor_state.cursor_position);
            
            if let (Some(before), Some(at)) = (before_cursor, at_cursor) {
                let before_str = before.to_string();
                let at_str = at.to_string();
                
                if self.is_opening_bracket(&before_str) && 
                   at_str == self.get_closing_bracket(&before_str) {
                    // Delete both brackets
                    actions.push(EditorAction::Delete(
                        self.editor_state.cursor_position - 1, 
                        2
                    ));
                    return actions;
                }
            }
        }
        
        // Regular backspace
        if self.editor_state.cursor_position > 0 {
            actions.push(EditorAction::Delete(
                self.editor_state.cursor_position - 1, 
                1
            ));
        }
        
        actions
    }

    fn trigger_auto_complete(&mut self) -> Vec<EditorAction> {
        let mut actions = Vec::new();
        
        if !self.auto_complete_enabled {
            return actions;
        }
        
        // Get current word for completion
        let current_word = self.get_current_word();
        if current_word.is_empty() {
            return actions;
        }
        
        // Get completions from command inspector
        let current_dir = std::env::current_dir().unwrap_or_default();
        self.current_completions = self.command_inspector
            .command_executor
            .get_completions(&current_word, &current_dir);
        
        if !self.current_completions.is_empty() {
            self.completion_index = Some(0);
            actions.push(EditorAction::ShowMessage(
                format!("Found {} completions", self.current_completions.len())
            ));
        }
        
        actions
    }

    fn apply_completion(&mut self) -> EditorAction {
        if let (Some(index), false) = (self.completion_index, self.current_completions.is_empty()) {
            let completion = &self.current_completions[index];
            let current_word = self.get_current_word();
            
            // Replace current word with completion
            let word_start = self.find_word_start();
            let replacement = completion[current_word.len()..].to_string();
            
            self.completion_index = Some((index + 1) % self.current_completions.len());
            
            EditorAction::Replace(word_start + current_word.len(), 0, replacement)
        } else {
            EditorAction::ShowMessage("No completions available".to_string())
        }
    }

    fn get_current_word(&self) -> String {
        let content = &self.editor_state.content;
        let cursor = self.editor_state.cursor_position;
        
        let word_start = content[..cursor]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        content[word_start..cursor].to_string()
    }

    fn find_word_start(&self) -> usize {
        let content = &self.editor_state.content;
        let cursor = self.editor_state.cursor_position;
        
        content[..cursor]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0)
    }

    fn validate_current_line(&self) -> Vec<EditorAction> {
        let current_line = self.editor_state.get_current_line_text();
        self.command_inspector.inspect_command(current_line)
    }

    fn apply_to_multi_cursors(&self, actions: &[EditorAction]) -> Vec<EditorAction> {
        let mut multi_actions = Vec::new();
        
        for &cursor_pos in &self.multi_cursors {
            for action in actions {
                match action {
                    EditorAction::Insert(text) => {
                        multi_actions.push(EditorAction::Replace(cursor_pos, 0, text.clone()));
                    },
                    EditorAction::Delete(start, length) => {
                        let adjusted_start = cursor_pos + start - self.editor_state.cursor_position;
                        multi_actions.push(EditorAction::Delete(adjusted_start, *length));
                    },
                    _ => {} // Other actions don't apply to multi-cursors
                }
            }
        }
        
        multi_actions
    }

    pub fn add_cursor(&mut self, position: usize) {
        if !self.multi_cursors.contains(&position) {
            self.multi_cursors.push(position);
        }
    }

    pub fn clear_multi_cursors(&mut self) {
        self.multi_cursors.clear();
    }

    pub fn get_cursor_positions(&self) -> Vec<usize> {
        let mut positions = vec![self.editor_state.cursor_position];
        positions.extend(&self.multi_cursors);
        positions
    }

    pub fn set_content(&mut self, content: String) {
        self.editor_state.set_content(content);
        self.clear_multi_cursors();
        self.current_completions.clear();
        self.completion_index = None;
    }

    pub fn get_content(&self) -> &str {
        &self.editor_state.content
    }

    pub fn get_cursor_position(&self) -> usize {
        self.editor_state.cursor_position
    }

    pub fn set_cursor_position(&mut self, position: usize) {
        self.editor_state.move_cursor(position);
    }

    // Feature toggles
    pub fn toggle_auto_complete(&mut self) {
        self.auto_complete_enabled = !self.auto_complete_enabled;
    }

    pub fn toggle_auto_indent(&mut self) {
        self.auto_indent_enabled = !self.auto_indent_enabled;
    }

    pub fn toggle_bracket_matching(&mut self) {
        self.bracket_matching_enabled = !self.bracket_matching_enabled;
    }

    pub fn toggle_word_wrap(&mut self) {
        self.word_wrap_enabled = !self.word_wrap_enabled;
    }

    pub fn toggle_syntax_highlighting(&mut self) {
        self.editor_state.syntax_highlighting_enabled = !self.editor_state.syntax_highlighting_enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_handler_creation() {
        let handler = TextInputHandler::new();
        assert!(handler.auto_complete_enabled);
        assert!(handler.auto_indent_enabled);
        assert!(handler.bracket_matching_enabled);
    }

    #[test]
    fn test_bracket_matching() {
        let mut handler = TextInputHandler::new();
        let actions = handler.handle_key_input(
            &Key::Character("(".into()), 
            &Modifiers::empty()
        );
        
        // Should insert both opening and closing bracket
        assert_eq!(actions.len(), 3); // Insert (, Insert ), MoveCursor
    }

    #[test]
    fn test_vim_mode_integration() {
        let mut handler = TextInputHandler::new();
        handler.set_vim_mode(true);
        
        assert!(handler.is_vim_enabled());
        assert_eq!(handler.get_current_vim_mode(), "NORMAL");
    }

    #[test]
    fn test_multi_cursor() {
        let mut handler = TextInputHandler::new();
        handler.add_cursor(10);
        handler.add_cursor(20);
        
        let positions = handler.get_cursor_positions();
        assert_eq!(positions.len(), 3); // Main cursor + 2 additional
    }
}
