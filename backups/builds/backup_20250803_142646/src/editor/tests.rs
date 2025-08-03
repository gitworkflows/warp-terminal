#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_editor_with_defaults() {
        let editor = ModernTextEditor::new();
        assert_eq!(editor.font_size, 14);
        assert_eq!(editor.multi_cursors.len(), 0);
        assert!(!editor.ai_suggestions_enabled);
    }

    #[test]
    fn test_update_editor_with_input_changed() {
        let mut editor = ModernTextEditor::new();
        editor.update(EditorMessage::InputChanged("Hello, World!".to_string()));
        assert_eq!(editor.state.get_content(), "Hello, World!");
    }

    #[test]
    fn test_multi_cursor_editing() {
        let mut editor = ModernTextEditor::new();
        editor.multi_cursors.push(CursorState {
            position: 0,
            line: 0,
            column: 0,
            selection_start: None,
            selection_end: None,
        });
        editor.update(EditorMessage::KeyPressed(KeyCode::A, Modifiers::empty()));
        assert_eq!(editor.get_content(), "a");
    }
}
