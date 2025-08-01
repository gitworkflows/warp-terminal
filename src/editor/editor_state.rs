
use std::collections::VecDeque;
use std::time::{SystemTime, Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorState {
    pub content: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub scroll_offset: usize,
    pub is_multiline: bool,
    pub line_numbers_visible: bool,
    pub syntax_highlighting_enabled: bool,
    
    // History for undo/redo
    history: VecDeque<EditorSnapshot>,
    future: VecDeque<EditorSnapshot>,
    last_save_time: SystemTime,
    max_history_size: usize,
    
    // Editor metrics
    pub total_lines: usize,
    pub current_line: usize,
    pub current_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorSnapshot {
    pub content: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub enum EditorAction {
    // Basic editing
    Insert(String),
    Delete(usize, usize), // start, length
    Replace(usize, usize, String), // start, length, new_text
    
    // Cursor movement
    MoveCursor(usize),
    SetSelection(usize, usize),
    ClearSelection,
    
    // Line operations
    InsertNewLine,
    DeleteLine(usize),
    DuplicateLine(usize),
    MoveLine(usize, usize), // from, to
    
    // Indentation
    Indent(usize), // line number
    Unindent(usize), // line number
    AutoIndent,
    
    // Search and replace
    Find(String),
    ReplaceAll(String, String),
    
    // History
    Undo,
    Redo,
    SaveSnapshot,
    
    // Multi-cursor operations
    AddCursor(usize),
    RemoveCursor(usize),
    
    // Folding
    FoldRegion(usize, usize),
    UnfoldRegion(usize, usize),
    
    // Messages and notifications
    ShowMessage(String),
    ShowErrorMessage(String),
    ShowWarningMessage(String),
    
    // Search navigation
    FindNext,
    FindPrevious,
    
    // Copy operations
    Copy,
    CopyLine(usize),
    CopySelection,
    Paste,
    
    // Command operations
    ExecuteCommand(String),
    ValidateCommand(String),
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            selection_start: None,
            selection_end: None,
            scroll_offset: 0,
            is_multiline: true,
            line_numbers_visible: true,
            syntax_highlighting_enabled: true,
            history: VecDeque::new(),
            future: VecDeque::new(),
            last_save_time: SystemTime::now(),
            max_history_size: 100,
            total_lines: 1,
            current_line: 1,
            current_column: 1,
        }
    }

    pub fn with_content(content: impl Into<String>) -> Self {
        let mut state = Self::new();
        state.set_content(content.into());
        state
    }

    pub fn set_content(&mut self, content: String) {
        self.save_snapshot();
        self.content = content;
        self.cursor_position = self.cursor_position.min(self.content.len());
        self.update_metrics();
    }

    pub fn insert_text(&mut self, text: &str) {
        self.save_snapshot();
        
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            // Replace selection
            let (start, end) = (start.min(end), start.max(end));
            self.content.replace_range(start..end, text);
            self.cursor_position = start + text.len();
            self.clear_selection();
        } else {
            // Insert at cursor
            self.content.insert_str(self.cursor_position, text);
            self.cursor_position += text.len();
        }
        
        self.update_metrics();
    }

    pub fn delete_range(&mut self, start: usize, end: usize) {
        if start >= end || end > self.content.len() {
            return;
        }
        
        self.save_snapshot();
        self.content.replace_range(start..end, "");
        self.cursor_position = start.min(self.content.len());
        self.clear_selection();
        self.update_metrics();
    }

    pub fn delete_current_selection(&mut self) -> bool {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start, end) = (start.min(end), start.max(end));
            self.delete_range(start, end);
            true
        } else {
            false
        }
    }

    pub fn backspace(&mut self) {
        if self.delete_current_selection() {
            return;
        }
        
        if self.cursor_position > 0 {
            self.save_snapshot();
            let char_boundary = self.find_prev_char_boundary(self.cursor_position);
            self.content.replace_range(char_boundary..self.cursor_position, "");
            self.cursor_position = char_boundary;
            self.update_metrics();
        }
    }

    pub fn delete(&mut self) {
        if self.delete_current_selection() {
            return;
        }
        
        if self.cursor_position < self.content.len() {
            self.save_snapshot();
            let char_boundary = self.find_next_char_boundary(self.cursor_position);
            self.content.replace_range(self.cursor_position..char_boundary, "");
            self.update_metrics();
        }
    }

    pub fn move_cursor(&mut self, position: usize) {
        self.cursor_position = position.min(self.content.len());
        self.update_cursor_position();
    }

    pub fn move_cursor_left(&mut self, extend_selection: bool) {
        if self.cursor_position > 0 {
            let new_pos = self.find_prev_char_boundary(self.cursor_position);
            
            if extend_selection {
                self.extend_selection_to(new_pos);
            } else {
                self.clear_selection();
            }
            
            self.cursor_position = new_pos;
            self.update_cursor_position();
        }
    }

    pub fn move_cursor_right(&mut self, extend_selection: bool) {
        if self.cursor_position < self.content.len() {
            let new_pos = self.find_next_char_boundary(self.cursor_position);
            
            if extend_selection {
                self.extend_selection_to(new_pos);
            } else {
                self.clear_selection();
            }
            
            self.cursor_position = new_pos;
            self.update_cursor_position();
        }
    }

    pub fn move_cursor_up(&mut self, extend_selection: bool) {
        let current_line_start = self.find_line_start(self.cursor_position);
        if current_line_start > 0 {
            let prev_line_start = self.find_line_start(current_line_start - 1);
            let prev_line_end = current_line_start - 1;
            let column_offset = self.cursor_position - current_line_start;
            let prev_line_length = prev_line_end - prev_line_start;
            
            let new_pos = prev_line_start + column_offset.min(prev_line_length);
            
            if extend_selection {
                self.extend_selection_to(new_pos);
            } else {
                self.clear_selection();
            }
            
            self.cursor_position = new_pos;
            self.update_cursor_position();
        }
    }

    pub fn move_cursor_down(&mut self, extend_selection: bool) {
        let current_line_start = self.find_line_start(self.cursor_position);
        let current_line_end = self.find_line_end(self.cursor_position);
        
        if current_line_end < self.content.len() {
            let next_line_start = current_line_end + 1;
            let next_line_end = self.find_line_end(next_line_start);
            let column_offset = self.cursor_position - current_line_start;
            let next_line_length = next_line_end - next_line_start;
            
            let new_pos = next_line_start + column_offset.min(next_line_length);
            
            if extend_selection {
                self.extend_selection_to(new_pos);
            } else {
                self.clear_selection();
            }
            
            self.cursor_position = new_pos;
            self.update_cursor_position();
        }
    }

    pub fn move_cursor_to_line_start(&mut self, extend_selection: bool) {
        let line_start = self.find_line_start(self.cursor_position);
        
        if extend_selection {
            self.extend_selection_to(line_start);
        } else {
            self.clear_selection();
        }
        
        self.cursor_position = line_start;
        self.update_cursor_position();
    }

    pub fn move_cursor_to_line_end(&mut self, extend_selection: bool) {
        let line_end = self.find_line_end(self.cursor_position);
        
        if extend_selection {
            self.extend_selection_to(line_end);
        } else {
            self.clear_selection();
        }
        
        self.cursor_position = line_end;
        self.update_cursor_position();
    }

    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.selection_end = Some(self.content.len());
    }

    pub fn select_word(&mut self) {
        let word_start = self.find_word_boundary_left(self.cursor_position);
        let word_end = self.find_word_boundary_right(self.cursor_position);
        
        self.selection_start = Some(word_start);
        self.selection_end = Some(word_end);
        self.cursor_position = word_end;
    }

    pub fn select_line(&mut self) {
        let line_start = self.find_line_start(self.cursor_position);
        let line_end = self.find_line_end(self.cursor_position);
        
        self.selection_start = Some(line_start);
        self.selection_end = Some(line_end);
    }

    pub fn get_selected_text(&self) -> Option<String> {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start, end) = (start.min(end), start.max(end));
            Some(self.content[start..end].to_string())
        } else {
            None
        }
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_end.is_some()
            && self.selection_start != self.selection_end
    }

    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.history.pop_back() {
            // Save current state to future
            self.future.push_back(self.create_snapshot());
            
            // Restore previous state
            self.restore_snapshot(snapshot);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.future.pop_back() {
            // Save current state to history
            self.history.push_back(self.create_snapshot());
            
            // Restore future state
            self.restore_snapshot(snapshot);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn insert_newline(&mut self) {
        self.insert_text("\n");
        
        // Auto-indentation
        if self.syntax_highlighting_enabled {
            let current_line_start = self.find_line_start(self.cursor_position - 1);
            let prev_line = &self.content[current_line_start..self.cursor_position - 1];
            let indentation = self.calculate_indentation(prev_line);
            
            if !indentation.is_empty() {
                self.insert_text(&indentation);
            }
        }
    }

    pub fn get_lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }

    pub fn get_line(&self, line_number: usize) -> Option<&str> {
        self.content.lines().nth(line_number)
    }

    pub fn get_current_line_text(&self) -> &str {
        let line_start = self.find_line_start(self.cursor_position);
        let line_end = self.find_line_end(self.cursor_position);
        &self.content[line_start..line_end]
    }

    // Private helper methods

    fn save_snapshot(&mut self) {
        // Don't save if it's too soon since last save
        if self.last_save_time.elapsed().unwrap_or(Duration::ZERO) < Duration::from_millis(500) {
            return;
        }
        
        let snapshot = self.create_snapshot();
        self.history.push_back(snapshot);
        
        // Clear future when making new changes
        self.future.clear();
        
        // Limit history size
        if self.history.len() > self.max_history_size {
            self.history.pop_front();
        }
        
        self.last_save_time = SystemTime::now();
    }

    fn create_snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            content: self.content.clone(),
            cursor_position: self.cursor_position,
            selection_start: self.selection_start,
            selection_end: self.selection_end,
            timestamp: SystemTime::now(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.content = snapshot.content;
        self.cursor_position = snapshot.cursor_position;
        self.selection_start = snapshot.selection_start;
        self.selection_end = snapshot.selection_end;
        self.update_metrics();
    }

    fn extend_selection_to(&mut self, position: usize) {
        if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor_position);
        }
        self.selection_end = Some(position);
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }

    fn update_metrics(&mut self) {
        self.total_lines = self.content.lines().count().max(1);
        self.update_cursor_position();
    }

    fn update_cursor_position(&mut self) {
        let before_cursor = &self.content[..self.cursor_position];
        self.current_line = before_cursor.matches('\n').count() + 1;
        
        if let Some(last_newline) = before_cursor.rfind('\n') {
            self.current_column = self.cursor_position - last_newline;
        } else {
            self.current_column = self.cursor_position + 1;
        }
    }

    fn find_prev_char_boundary(&self, mut pos: usize) -> usize {
        while pos > 0 && !self.content.is_char_boundary(pos) {
            pos -= 1;
        }
        if pos > 0 {
            pos -= 1;
            while pos > 0 && !self.content.is_char_boundary(pos) {
                pos -= 1;
            }
        }
        pos
    }

    fn find_next_char_boundary(&self, mut pos: usize) -> usize {
        let len = self.content.len();
        if pos < len {
            pos += 1;
            while pos < len && !self.content.is_char_boundary(pos) {
                pos += 1;
            }
        }
        pos
    }

    pub fn find_line_start(&self, pos: usize) -> usize {
        self.content[..pos].rfind('\n').map_or(0, |i| i + 1)
    }

    fn find_line_end(&self, pos: usize) -> usize {
        self.content[pos..].find('\n').map_or(self.content.len(), |i| pos + i)
    }

    fn find_word_boundary_left(&self, mut pos: usize) -> usize {
        while pos > 0 {
            let ch = self.content.chars().nth(pos - 1);
            if let Some(c) = ch {
                if c.is_alphanumeric() || c == '_' {
                    pos -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        pos
    }

    fn find_word_boundary_right(&self, mut pos: usize) -> usize {
        let chars: Vec<char> = self.content.chars().collect();
        while pos < chars.len() {
            let ch = chars[pos];
            if ch.is_alphanumeric() || ch == '_' {
                pos += 1;
            } else {
                break;
            }
        }
        pos
    }

    fn calculate_indentation(&self, line: &str) -> String {
        let mut indent = String::new();
        for ch in line.chars() {
            if ch == ' ' || ch == '\t' {
                indent.push(ch);
            } else {
                break;
            }
        }
        indent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_editing() {
        let mut state = EditorState::new();
        
        state.insert_text("Hello");
        assert_eq!(state.content, "Hello");
        assert_eq!(state.cursor_position, 5);
        
        state.insert_text(" World");
        assert_eq!(state.content, "Hello World");
        assert_eq!(state.cursor_position, 11);
    }

    #[test]
    fn test_undo_redo() {
        let mut state = EditorState::new();
        
        // First save initial state
        state.save_snapshot();
        state.insert_text("Hello");
        
        // Force save another snapshot by waiting
        state.last_save_time = std::time::SystemTime::now() - std::time::Duration::from_secs(1);
        state.save_snapshot();
        state.insert_text(" World");
        
        assert!(state.undo());
        assert_eq!(state.content, "Hello");
        
        assert!(state.redo());
        assert_eq!(state.content, "Hello World");
    }

    #[test]
    fn test_selection() {
        let mut state = EditorState::with_content("Hello World");
        
        state.cursor_position = 0;
        state.selection_start = Some(0);
        state.selection_end = Some(5);
        
        assert_eq!(state.get_selected_text(), Some("Hello".to_string()));
        assert!(state.has_selection());
        
        state.insert_text("Hi");
        assert_eq!(state.content, "Hi World");
        assert!(!state.has_selection());
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = EditorState::with_content("Line 1\nLine 2\nLine 3");
        
        state.cursor_position = 0;
        state.move_cursor_down(false);
        assert_eq!(state.cursor_position, 7); // Start of "Line 2"
        
        state.move_cursor_up(false);
        assert_eq!(state.cursor_position, 0); // Back to start
    }
}
