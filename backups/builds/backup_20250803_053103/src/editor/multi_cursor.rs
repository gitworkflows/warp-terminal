use crate::editor::{EditorAction, EditorState};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct MultiCursorManager {
    cursors: BTreeSet<usize>,
    primary_cursor: usize,
    selection_ranges: Vec<(usize, usize)>, // (start, end) pairs
    enabled: bool,
}

impl Default for MultiCursorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiCursorManager {
    pub fn new() -> Self {
        Self {
            cursors: BTreeSet::new(),
            primary_cursor: 0,
            selection_ranges: Vec::new(),
            enabled: true,
        }
    }

    pub fn add_cursor(&mut self, position: usize) {
        if self.enabled {
            self.cursors.insert(position);
        }
    }

    pub fn remove_cursor(&mut self, position: usize) {
        self.cursors.remove(&position);
        if self.cursors.is_empty() {
            self.primary_cursor = position;
        }
    }

    pub fn clear_cursors(&mut self) {
        self.cursors.clear();
        self.selection_ranges.clear();
    }

    pub fn get_all_positions(&self) -> Vec<usize> {
        let mut positions = vec![self.primary_cursor];
        positions.extend(self.cursors.iter().copied());
        positions.sort();
        positions
    }

    pub fn set_primary_cursor(&mut self, position: usize) {
        self.primary_cursor = position;
    }

    pub fn get_primary_cursor(&self) -> usize {
        self.primary_cursor
    }

    pub fn has_multiple_cursors(&self) -> bool {
        !self.cursors.is_empty()
    }

    pub fn cursor_count(&self) -> usize {
        self.cursors.len() + 1 // +1 for primary cursor
    }

    /// Add cursor at each occurrence of selected text
    pub fn add_cursors_for_selection(&mut self, editor_state: &EditorState, selected_text: &str) {
        if selected_text.is_empty() {
            return;
        }

        let content = &editor_state.content;
        let mut start_pos = 0;

        while let Some(pos) = content[start_pos..].find(selected_text) {
            let absolute_pos = start_pos + pos;
            self.add_cursor(absolute_pos);
            start_pos = absolute_pos + selected_text.len();
        }
    }

    /// Add cursor above current position
    pub fn add_cursor_above(&mut self, editor_state: &EditorState) {
        let current_line_start = editor_state.find_line_start(self.primary_cursor);
        if current_line_start > 0 {
            let prev_line_start = editor_state.find_line_start(current_line_start - 1);
            let column_offset = self.primary_cursor - current_line_start;
            let prev_line_end = current_line_start - 1;
            let prev_line_length = prev_line_end - prev_line_start;
            
            let new_cursor_pos = prev_line_start + column_offset.min(prev_line_length);
            self.add_cursor(new_cursor_pos);
        }
    }

    /// Add cursor below current position
    pub fn add_cursor_below(&mut self, editor_state: &EditorState) {
        let current_line_start = editor_state.find_line_start(self.primary_cursor);
        let current_line_end = self.find_line_end(editor_state, self.primary_cursor);
        
        if current_line_end < editor_state.content.len() {
            let next_line_start = current_line_end + 1;
            let column_offset = self.primary_cursor - current_line_start;
            let next_line_end = self.find_line_end(editor_state, next_line_start);
            let next_line_length = next_line_end - next_line_start;
            
            let new_cursor_pos = next_line_start + column_offset.min(next_line_length);
            self.add_cursor(new_cursor_pos);
        }
    }

    /// Apply action to all cursors
    pub fn apply_action_to_all(&self, action: &EditorAction, editor_state: &EditorState) -> Vec<EditorAction> {
        if !self.has_multiple_cursors() {
            return vec![action.clone()];
        }

        let mut actions = Vec::new();
        let positions = self.get_all_positions();

        match action {
            EditorAction::Insert(text) => {
                // Apply insertions in reverse order to maintain position accuracy
                for &pos in positions.iter().rev() {
                    actions.push(EditorAction::Replace(pos, 0, text.clone()));
                }
            },
            
            EditorAction::Delete(_, length) => {
                // Apply deletions in reverse order
                for &pos in positions.iter().rev() {
                    actions.push(EditorAction::Delete(pos, *length));
                }
            },
            
            EditorAction::Replace(_, length, replacement) => {
                // Apply replacements in reverse order
                for &pos in positions.iter().rev() {
                    actions.push(EditorAction::Replace(pos, *length, replacement.clone()));
                }
            },
            
            _ => {
                // For other actions, apply to primary cursor only
                actions.push(action.clone());
            }
        }

        actions
    }

    /// Update cursor positions after text modification
    pub fn update_positions_after_edit(&mut self, edit_position: usize, length_change: i32) {
        // Update primary cursor
        if self.primary_cursor >= edit_position {
            if length_change >= 0 {
                self.primary_cursor += length_change as usize;
            } else {
                let decrease = (-length_change) as usize;
                if self.primary_cursor >= edit_position + decrease {
                    self.primary_cursor -= decrease;
                } else {
                    self.primary_cursor = edit_position;
                }
            }
        }

        // Update all other cursors
        let mut updated_cursors = BTreeSet::new();
        for &cursor in &self.cursors {
            let updated_pos = if cursor >= edit_position {
                if length_change >= 0 {
                    cursor + length_change as usize
                } else {
                    let decrease = (-length_change) as usize;
                    if cursor >= edit_position + decrease {
                        cursor - decrease
                    } else {
                        edit_position
                    }
                }
            } else {
                cursor
            };
            updated_cursors.insert(updated_pos);
        }
        self.cursors = updated_cursors;
    }

    /// Select word at each cursor position
    pub fn select_word_at_cursors(&mut self, editor_state: &EditorState) -> Vec<EditorAction> {
        let mut actions = Vec::new();
        
        for &cursor_pos in &self.get_all_positions() {
            let word_start = self.find_word_start_at(editor_state, cursor_pos);
            let word_end = self.find_word_end_at(editor_state, cursor_pos);
            
            if word_start < word_end {
                self.selection_ranges.push((word_start, word_end));
                actions.push(EditorAction::SetSelection(word_start, word_end));
            }
        }
        
        actions
    }

    /// Find next occurrence of word under cursor and add cursor there
    pub fn find_and_add_next_occurrence(&mut self, editor_state: &EditorState) -> Option<EditorAction> {
        let word_start = self.find_word_start_at(editor_state, self.primary_cursor);
        let word_end = self.find_word_end_at(editor_state, self.primary_cursor);
        
        if word_start >= word_end {
            return None;
        }
        
        let word = &editor_state.content[word_start..word_end];
        let search_start = word_end;
        
        if let Some(next_pos) = editor_state.content[search_start..].find(word) {
            let absolute_pos = search_start + next_pos;
            self.add_cursor(absolute_pos);
            Some(EditorAction::MoveCursor(absolute_pos))
        } else {
            None
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.clear_cursors();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    // Helper methods
    
    fn find_line_end(&self, editor_state: &EditorState, pos: usize) -> usize {
        editor_state.content[pos..].find('\n').map_or(editor_state.content.len(), |i| pos + i)
    }

    fn find_word_start_at(&self, editor_state: &EditorState, pos: usize) -> usize {
        let content = &editor_state.content;
        let mut start = pos;
        
        while start > 0 {
            let ch = content.chars().nth(start - 1);
            if let Some(c) = ch {
                if c.is_alphanumeric() || c == '_' {
                    start -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        start
    }

    fn find_word_end_at(&self, editor_state: &EditorState, pos: usize) -> usize {
        let content = &editor_state.content;
        let chars: Vec<char> = content.chars().collect();
        let mut end = pos;
        
        while end < chars.len() {
            let ch = chars[end];
            if ch.is_alphanumeric() || ch == '_' {
                end += 1;
            } else {
                break;
            }
        }
        
        // Convert back to byte position
        content.char_indices().nth(end).map_or(content.len(), |(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_cursor_creation() {
        let mut manager = MultiCursorManager::new();
        assert!(!manager.has_multiple_cursors());
        assert_eq!(manager.cursor_count(), 1);
        
        manager.add_cursor(10);
        assert!(manager.has_multiple_cursors());
        assert_eq!(manager.cursor_count(), 2);
    }

    #[test]
    fn test_cursor_positions() {
        let mut manager = MultiCursorManager::new();
        manager.set_primary_cursor(5);
        manager.add_cursor(10);
        manager.add_cursor(15);
        
        let positions = manager.get_all_positions();
        assert_eq!(positions, vec![5, 10, 15]);
    }

    #[test]
    fn test_position_updates_after_edit() {
        let mut manager = MultiCursorManager::new();
        manager.set_primary_cursor(10);
        manager.add_cursor(20);
        manager.add_cursor(30);
        
        // Insert 5 characters at position 5
        manager.update_positions_after_edit(5, 5);
        
        let positions = manager.get_all_positions();
        assert_eq!(positions, vec![15, 25, 35]);
    }

    #[test]
    fn test_clear_cursors() {
        let mut manager = MultiCursorManager::new();
        manager.add_cursor(10);
        manager.add_cursor(20);
        
        assert!(manager.has_multiple_cursors());
        
        manager.clear_cursors();
        assert!(!manager.has_multiple_cursors());
    }
}
