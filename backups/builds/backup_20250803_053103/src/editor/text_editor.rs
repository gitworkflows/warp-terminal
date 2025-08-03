use iced::{
    widget::{canvas, container, text, row, column, scrollable, Space},
    Element, Length, Color, Font, Point, Size, Rectangle, Background, border,
    keyboard::{key::{Key, Named}, Modifiers},
    mouse::Cursor,
    Renderer,
    Theme,
    alignment,
};
use iced::widget::canvas::{Cache, Canvas, Geometry, Path, Stroke, Text};
use iced::widget::text::{LineHeight, Shaping};

use crate::editor::{
    EditorState, EditorAction, SyntaxHighlighter, KeybindingManager,
    CompletionEngine, CompletionItem,
};


// Additional structs for modern text editing features
#[derive(Debug, Clone)]
pub struct CursorState {
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct CollaborationCursor {
    pub user_id: String,
    pub user_name: String,
    pub position: usize,
    pub color: Color,
    pub last_updated: std::time::Instant,
}

#[derive(Debug)]
pub struct ModernTextEditor {
    state: EditorState,
    syntax_highlighter: SyntaxHighlighter,
    keybinding_manager: KeybindingManager,
    completion_engine: CompletionEngine,
    
    // UI state
    font: Font,
    font_size: u16,
    line_height: f32,
    char_width: f32,
    
    // Visual settings
    show_line_numbers: bool,
    #[allow(dead_code)]
    show_whitespace: bool,
    highlight_current_line: bool,
    show_minimap: bool,
    #[allow(dead_code)]
    show_folding: bool,
    wrap_lines: bool,
    
    // Advanced features
    multi_cursors: Vec<CursorState>,
    bracket_matching: bool,
    #[allow(dead_code)]
    auto_indent: bool,
    #[allow(dead_code)]
    smart_indent: bool,
    auto_closing_brackets: bool,
    
    // Interaction state
    is_focused: bool,
    mouse_position: Point,
    last_click_time: std::time::Instant,
    click_count: u8,
    
    // Performance
    cache: Cache,
    needs_redraw: bool,
    #[allow(dead_code)]
    visible_range: (usize, usize),
    
    // Completion state
    completion_visible: bool,
    completion_items: Vec<CompletionItem>,
    selected_completion: usize,
    ai_suggestions_enabled: bool,
    
    // Search and replace
    search_query: String,
    search_results: Vec<SearchResult>,
    current_search_index: usize,
    #[allow(dead_code)]
    replace_mode: bool,
    
    // Code folding
    folded_regions: std::collections::HashSet<usize>,
    
    // Live collaboration
    collaboration_cursors: std::collections::HashMap<String, CollaborationCursor>,
}

#[derive(Debug, Clone)]
pub enum EditorMessage {
    InputChanged(String),
    KeyPressed(Key, Modifiers),
    MouseClicked(Point),
    MouseMoved(Point),
    ScrollChanged(f32),
    CompletionSelected(usize),
    ActionRequested(EditorAction),
}

impl Default for ModernTextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl ModernTextEditor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            keybinding_manager: KeybindingManager::new(),
            completion_engine: CompletionEngine::new(),
            
            font: Font::MONOSPACE,
            font_size: 14,
            line_height: 20.0,
            char_width: 8.5,
            
            show_line_numbers: true,
            show_whitespace: false,
            highlight_current_line: true,
            show_minimap: false,
            show_folding: true,
            wrap_lines: false,
            
            multi_cursors: Vec::new(),
            bracket_matching: true,
            auto_indent: true,
            smart_indent: true,
            auto_closing_brackets: true,
            
            is_focused: false,
            mouse_position: Point::ORIGIN,
            last_click_time: std::time::Instant::now(),
            click_count: 0,
            
            cache: Cache::new(),
            needs_redraw: true,
            visible_range: (0, 0),
            
            completion_visible: false,
            completion_items: Vec::new(),
            selected_completion: 0,
            ai_suggestions_enabled: false,
            
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            replace_mode: false,
            
            folded_regions: std::collections::HashSet::new(),
            
            collaboration_cursors: std::collections::HashMap::new(),
        }
    }

    pub fn with_content(content: impl Into<String>) -> Self {
        let mut editor = Self::new();
        editor.state = EditorState::with_content(content);
        editor.invalidate_cache();
        editor
    }

    pub fn update(&mut self, message: EditorMessage) {
        match message {
            EditorMessage::InputChanged(text) => {
                self.state.set_content(text);
                self.update_completions();
                self.invalidate_cache();
            }
            
            EditorMessage::KeyPressed(key, modifiers) => {
            if let Some(action) = self.keybinding_manager.handle_key_press(&key, &modifiers) {
                    self.perform_action(action);
                } else {
                    // Handle regular key input
                    self.handle_key_input(key, modifiers);
                }
                self.invalidate_cache();
            }
            
            EditorMessage::MouseClicked(point) => {
                self.handle_mouse_click(point);
                self.invalidate_cache();
            }
            
            EditorMessage::MouseMoved(point) => {
                self.mouse_position = point;
            }
            
            EditorMessage::ScrollChanged(offset) => {
                self.state.scroll_offset = offset as usize;
                self.invalidate_cache();
            }
            
            EditorMessage::CompletionSelected(index) => {
                if index < self.completion_items.len() {
                    self.apply_completion(&self.completion_items[index].clone());
                    self.hide_completions();
                }
                self.invalidate_cache();
            }
            
            EditorMessage::ActionRequested(action) => {
                self.perform_action(action);
                self.invalidate_cache();
            }
        }
    }

    pub fn view(&self) -> Element<'_, EditorMessage> {
        let editor_canvas = Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut content = column![editor_canvas].spacing(0);

        // Add completion popup if visible
        if self.completion_visible && !self.completion_items.is_empty() {
            let completion_popup = self.create_completion_popup();
            content = content.push(completion_popup);
        }

        container(content)
            .style(move |_theme: &Theme| container::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.05, 0.05, 0.09))),
                border: border::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color::from_rgb(0.25, 0.25, 0.35),
                },
                text_color: Some(Color::WHITE),
                ..Default::default()
            })
            .padding(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // Canvas implementation
    pub fn draw_canvas(&self, renderer: &Renderer, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Draw background
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb(0.05, 0.05, 0.09));

            // Calculate visible area
            let visible_lines = (bounds.height / self.line_height) as usize + 1;
            let start_line = self.state.scroll_offset;
            let end_line = (start_line + visible_lines).min(self.state.total_lines);

            // Draw line numbers if enabled
            let line_number_width = if self.show_line_numbers {
                self.draw_line_numbers(frame, start_line, end_line, bounds)
            } else {
                0.0
            };

            // Draw text content
            let text_area = Rectangle {
                x: line_number_width + 10.0,
                y: bounds.y,
                width: bounds.width - line_number_width - 20.0,
                height: bounds.height,
            };

            self.draw_text_content(frame, text_area, start_line, end_line);

            // Draw cursor
            if self.is_focused {
                self.draw_cursor(frame, text_area, line_number_width);
            }

            // Draw selection
            if self.state.has_selection() {
                self.draw_selection(frame, text_area, line_number_width);
            }

            // Highlight current line
            if self.highlight_current_line && self.is_focused {
                self.draw_current_line_highlight(frame, text_area);
            }

            // Draw syntax errors
            self.draw_syntax_errors(frame, text_area);
        });

        vec![geometry]
    }

    // Private methods

    fn perform_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::Undo => {
                self.state.undo();
            }
            EditorAction::Redo => {
                self.state.redo();
            }
            EditorAction::Insert(text) => {
                self.state.insert_text(&text);
            }
            EditorAction::Delete(start, length) => {
                self.state.delete_range(start, start + length);
            }
            EditorAction::MoveCursor(position) => {
                self.state.move_cursor(position);
            }
            EditorAction::SetSelection(start, end) => {
                self.state.selection_start = Some(start);
                self.state.selection_end = Some(end);
            }
            EditorAction::ClearSelection => {
                self.state.selection_start = None;
                self.state.selection_end = None;
            }
            EditorAction::InsertNewLine => {
                self.state.insert_newline();
            }
            _ => {
                // Handle other actions as needed
            }
        }
    }

    fn handle_key_input(&mut self, key: Key, modifiers: Modifiers) {
        match key {
            Key::Named(Named::Enter) => {
                if self.completion_visible {
                    self.apply_selected_completion();
                } else {
                    self.state.insert_newline();
                }
            }
            Key::Named(Named::Tab) => {
                if self.completion_visible {
                    self.apply_selected_completion();
                } else {
                    self.state.insert_text("    "); // 4 spaces
                }
            }
            Key::Named(Named::Escape) => {
                if self.completion_visible {
                    self.hide_completions();
                } else {
                    self.state.clear_selection();
                }
            }
            Key::Named(Named::Backspace) => {
                self.state.backspace();
                self.update_completions();
            }
            Key::Named(Named::Delete) => {
                self.state.delete();
            }
            Key::Named(Named::ArrowLeft) => {
                self.state.move_cursor_left(modifiers.contains(Modifiers::SHIFT));
            }
            Key::Named(Named::ArrowRight) => {
                self.state.move_cursor_right(modifiers.contains(Modifiers::SHIFT));
            }
            Key::Named(Named::ArrowUp) => {
                if self.completion_visible {
                    self.selected_completion = self.selected_completion.saturating_sub(1);
                } else {
                    self.state.move_cursor_up(modifiers.contains(Modifiers::SHIFT));
                }
            }
            Key::Named(Named::ArrowDown) => {
                if self.completion_visible {
                    self.selected_completion = (self.selected_completion + 1).min(self.completion_items.len().saturating_sub(1));
                } else {
                    self.state.move_cursor_down(modifiers.contains(Modifiers::SHIFT));
                }
            }
            Key::Named(Named::Home) => {
                self.state.move_cursor_to_line_start(modifiers.contains(Modifiers::SHIFT));
            }
            Key::Named(Named::End) => {
                self.state.move_cursor_to_line_end(modifiers.contains(Modifiers::SHIFT));
            }
            Key::Character(c) => {
                self.state.insert_text(&c.to_string());
                self.update_completions();
            }
            _ => {
                // Other keys are ignored for now
            }
        }
    }

    fn handle_mouse_click(&mut self, point: Point) {
        let now = std::time::Instant::now();
        let time_since_last_click = now.duration_since(self.last_click_time);
        
        if time_since_last_click < std::time::Duration::from_millis(500) {
            self.click_count += 1;
        } else {
            self.click_count = 1;
        }
        
        self.last_click_time = now;
        self.is_focused = true;

        // Convert point to text position
        let position = self.point_to_position(point);
        
        match self.click_count {
            1 => {
                // Single click - move cursor
                self.state.move_cursor(position);
                self.state.clear_selection();
            }
            2 => {
                // Double click - select word
                self.state.move_cursor(position);
                self.state.select_word();
            }
            3 => {
                // Triple click - select line
                self.state.move_cursor(position);
                self.state.select_line();
            }
            _ => {}
        }
    }

    fn point_to_position(&self, point: Point) -> usize {
        let line_number_width = if self.show_line_numbers { 50.0 } else { 0.0 };
        let text_x = point.x - line_number_width - 10.0;
        let text_y = point.y;
        
        let line = ((text_y / self.line_height) as usize + self.state.scroll_offset).min(self.state.total_lines - 1);
        let column = (text_x / self.char_width) as usize;
        
        // Convert line/column to absolute position
        let lines = self.state.get_lines();
        let mut position = 0;
        
        for (i, line_text) in lines.iter().enumerate() {
            if i == line {
                position += column.min(line_text.len());
                break;
            }
            position += line_text.len() + 1; // +1 for newline
        }
        
        position.min(self.state.content.len())
    }

    fn update_completions(&mut self) {
        let current_line = self.state.get_current_line_text();
        let cursor_column = self.state.current_column.saturating_sub(1);
        
        if let Some(word_start) = self.find_word_start(current_line, cursor_column) {
            let partial_word = &current_line[word_start..cursor_column];
            
            if partial_word.len() >= 2 {
                self.completion_items = self.completion_engine.get_completions(partial_word);
                self.completion_visible = !self.completion_items.is_empty();
                self.selected_completion = 0;
            } else {
                self.hide_completions();
            }
        } else {
            self.hide_completions();
        }
    }

    fn find_word_start(&self, line: &str, column: usize) -> Option<usize> {
        let chars: Vec<char> = line.chars().collect();
        let mut pos = column;
        
        while pos > 0 {
            let ch = chars.get(pos - 1)?;
            if ch.is_alphanumeric() || *ch == '_' {
                pos -= 1;
            } else {
                break;
            }
        }
        
        if pos < column {
            Some(pos)
        } else {
            None
        }
    }

    fn apply_selected_completion(&mut self) {
        if let Some(item) = self.completion_items.get(self.selected_completion).cloned() {
            self.apply_completion(&item);
        }
        self.hide_completions();
    }

    fn apply_completion(&mut self, item: &CompletionItem) {
        let current_line = self.state.get_current_line_text().to_string();
        let cursor_column = self.state.current_column.saturating_sub(1);
        
        if let Some(word_start) = self.find_word_start(&current_line, cursor_column) {
            let line_start = self.state.find_line_start(self.state.cursor_position);
            let absolute_word_start = line_start + word_start;
            let absolute_cursor = line_start + cursor_column;
            
            // Replace the partial word with the completion
            self.state.delete_range(absolute_word_start, absolute_cursor);
            self.state.cursor_position = absolute_word_start;
            self.state.insert_text(&item.text);
        }
    }

    fn hide_completions(&mut self) {
        self.completion_visible = false;
        self.completion_items.clear();
        self.selected_completion = 0;
    }

    fn create_completion_popup(&self) -> Element<'_, EditorMessage> {
        let mut items = column![].spacing(2);
        
        for (i, item) in self.completion_items.iter().enumerate() {
            let is_selected = i == self.selected_completion;
            let item_style = if is_selected {
                move |_theme: &Theme| container::Appearance {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.4, 0.8))),
                    border: border::Border::default(),
                    text_color: Some(Color::WHITE),
                    ..Default::default()
                }
            } else {
                move |_theme: &Theme| container::Appearance {
                    background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.15))),
                    border: border::Border::default(),
                    text_color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
                    ..Default::default()
                }
            };

            let item_widget = container(
                row![
                    text(&item.text).size(self.font_size).font(self.font),
                    Space::with_width(Length::Fill),
                    text(&item.detail.as_deref().unwrap_or(""))
                        .size(self.font_size - 2)
                        .style(Color::from_rgb(0.6, 0.6, 0.7)),
                ]
                .spacing(8)
                .align_items(iced::Alignment::Center)
            )
            .style(item_style)
            .padding([4, 8])
            .width(Length::Fill);

            items = items.push(item_widget);
        }

        container(
            scrollable(items)
                .height(Length::Fixed(200.0))
        )
        .style(move |_theme: &Theme| container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.08, 0.12))),
            border: border::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.3, 0.3, 0.4),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        })
        .padding(2)
        .width(Length::Fixed(300.0))
        .into()
    }

    fn draw_line_numbers(&self, frame: &mut iced::widget::canvas::Frame, start_line: usize, end_line: usize, bounds: Rectangle) -> f32 {
        let width = 50.0;
        
        // Draw line number background
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(width, bounds.height),
            Color::from_rgb(0.08, 0.08, 0.12)
        );
        
        // Draw line numbers
        for line_idx in start_line..end_line {
            let line_number = line_idx + 1;
            let y = (line_idx - start_line) as f32 * self.line_height;
            
            let is_current_line = line_number == self.state.current_line;
            let color = if is_current_line {
                Color::from_rgb(0.9, 0.9, 1.0)
            } else {
                Color::from_rgb(0.5, 0.5, 0.6)
            };
            
            frame.fill_text(Text {
                content: format!("{:>3}", line_number),
                position: Point::new(width - 10.0, y + 2.0),
                color,
                size: iced::Pixels((self.font_size - 1) as f32),
                line_height: LineHeight::Absolute(iced::Pixels(self.line_height)),
                shaping: Shaping::Basic,
                font: self.font,
                horizontal_alignment: alignment::Horizontal::Right,
                vertical_alignment: alignment::Vertical::Top,
            });
        }
        
        // Draw separator line
        frame.stroke(&Path::line(
            Point::new(width, 0.0),
            Point::new(width, bounds.height)
        ), Stroke::default().with_width(1.0).with_color(Color::from_rgb(0.2, 0.2, 0.3)));
        
        width
    }

    fn draw_text_content(&self, frame: &mut iced::widget::canvas::Frame, text_area: Rectangle, start_line: usize, end_line: usize) {
        let lines = self.state.get_lines();
        let highlighted_spans = self.syntax_highlighter.highlight_command_line(&self.state.content);
        
        for line_idx in start_line..end_line {
            if let Some(line_text) = lines.get(line_idx) {
                let y = (line_idx - start_line) as f32 * self.line_height;
                
                // Find spans for this line
                let line_start_pos = lines.iter().take(line_idx).map(|l| l.len() + 1).sum::<usize>();
                let line_end_pos = line_start_pos + line_text.len();
                
                let line_spans: Vec<_> = highlighted_spans.iter()
                    .filter(|span| span.range.start < line_end_pos && span.range.end > line_start_pos)
                    .collect();
                
                if line_spans.is_empty() {
                    // No highlighting, draw plain text
                    frame.fill_text(Text {
                        content: line_text.to_string(),
                        position: Point::new(text_area.x, text_area.y + y + 2.0),
                        color: Color::from_rgb(0.9, 0.9, 0.9),
                        size: iced::Pixels(self.font_size as f32),
                        line_height: LineHeight::Absolute(iced::Pixels(self.line_height)),
                        shaping: Shaping::Basic,
                        font: self.font,
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Top,
                    });
                } else {
                    // Draw highlighted text
                    let mut x_offset = 0.0;
                    let mut last_end = line_start_pos;
                    
                    for span in line_spans {
                        let span_start = span.range.start.max(line_start_pos) - line_start_pos;
                        let span_end = span.range.end.min(line_end_pos) - line_start_pos;
                        
                        // Draw text before this span
                        if span_start > last_end - line_start_pos {
                            let before_text = &line_text[last_end - line_start_pos..span_start];
                            frame.fill_text(Text {
                                content: before_text.to_string(),
                                position: Point::new(text_area.x + x_offset, text_area.y + y + 2.0),
                                color: Color::from_rgb(0.9, 0.9, 0.9),
                                size: iced::Pixels(self.font_size as f32),
                                line_height: LineHeight::Absolute(iced::Pixels(self.line_height)),
                                shaping: Shaping::Basic,
                                font: self.font,
                                horizontal_alignment: alignment::Horizontal::Left,
                                vertical_alignment: alignment::Vertical::Top,
                            });
                            x_offset += before_text.len() as f32 * self.char_width;
                        }
                        
                        // Draw highlighted span
                        let span_text = &line_text[span_start..span_end];
                        frame.fill_text(Text {
                            content: span_text.to_string(),
                            position: Point::new(text_area.x + x_offset, text_area.y + y + 2.0),
                            color: span.color,
                            size: iced::Pixels(self.font_size as f32),
                            line_height: LineHeight::Absolute(iced::Pixels(self.line_height)),
                            shaping: Shaping::Basic,
                            font: self.font,
                            horizontal_alignment: alignment::Horizontal::Left,
                            vertical_alignment: alignment::Vertical::Top,
                        });
                        x_offset += span_text.len() as f32 * self.char_width;
                        
                        last_end = span.range.end;
                    }
                    
                    // Draw remaining text
                    if last_end < line_end_pos {
                        let remaining_text = &line_text[last_end - line_start_pos..];
                        frame.fill_text(Text {
                            content: remaining_text.to_string(),
                            position: Point::new(text_area.x + x_offset, text_area.y + y + 2.0),
                            color: Color::from_rgb(0.9, 0.9, 0.9),
                            size: iced::Pixels(self.font_size as f32),
                            font: self.font,
                            horizontal_alignment: alignment::Horizontal::Left,
                            vertical_alignment: alignment::Vertical::Top,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    fn draw_cursor(&self, frame: &mut iced::widget::canvas::Frame, text_area: Rectangle, _line_number_width: f32) {
        let cursor_line = self.state.current_line.saturating_sub(1);
        let cursor_column = self.state.current_column.saturating_sub(1);
        
        if cursor_line >= self.state.scroll_offset 
            && cursor_line < self.state.scroll_offset + (text_area.height / self.line_height) as usize {
            
            let x = text_area.x + cursor_column as f32 * self.char_width;
            let y = text_area.y + (cursor_line - self.state.scroll_offset) as f32 * self.line_height;
            
            // Draw cursor line
            frame.stroke(&Path::line(
                Point::new(x, y),
                Point::new(x, y + self.line_height - 2.0)
            ), Stroke::default().with_width(2.0).with_color(Color::from_rgb(0.2, 0.6, 1.0)));
        }
    }

    fn draw_selection(&self, frame: &mut iced::widget::canvas::Frame, text_area: Rectangle, _line_number_width: f32) {
        if let (Some(start), Some(end)) = (self.state.selection_start, self.state.selection_end) {
            let (start, end) = (start.min(end), start.max(end));
            
            // Convert positions to line/column coordinates
            let lines = self.state.get_lines();
            let mut pos = 0;
            let mut start_line = 0;
            let mut start_column = 0;
            let mut end_line = 0;
            let mut end_column = 0;
            
            for (line_idx, line_text) in lines.iter().enumerate() {
                if pos + line_text.len() >= start && start_line == 0 {
                    start_line = line_idx;
                    start_column = start - pos;
                }
                if pos + line_text.len() >= end && end_line == 0 {
                    end_line = line_idx;
                    end_column = end - pos;
                    break;
                }
                pos += line_text.len() + 1; // +1 for newline
            }
            
            // Draw selection rectangles
            for line in start_line..=end_line {
                if line >= self.state.scroll_offset 
                    && line < self.state.scroll_offset + (text_area.height / self.line_height) as usize {
                    
                    let y = text_area.y + (line - self.state.scroll_offset) as f32 * self.line_height;
                    
                    let (sel_start, sel_end) = if line == start_line && line == end_line {
                        (start_column, end_column)
                    } else if line == start_line {
                        (start_column, lines[line].len())
                    } else if line == end_line {
                        (0, end_column)
                    } else {
                        (0, lines[line].len())
                    };
                    
                    let x1 = text_area.x + sel_start as f32 * self.char_width;
                    let x2 = text_area.x + sel_end as f32 * self.char_width;
                    
                    frame.fill_rectangle(
                        Point::new(x1, y),
                        Size::new(x2 - x1, self.line_height - 2.0),
                        Color::from_rgba(0.2, 0.4, 0.8, 0.3)
                    );
                }
            }
        }
    }

    fn draw_current_line_highlight(&self, frame: &mut iced::widget::canvas::Frame, text_area: Rectangle) {
        let current_line = self.state.current_line.saturating_sub(1);
        
        if current_line >= self.state.scroll_offset 
            && current_line < self.state.scroll_offset + (text_area.height / self.line_height) as usize {
            
            let y = text_area.y + (current_line - self.state.scroll_offset) as f32 * self.line_height;
            
            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(text_area.width + 60.0, self.line_height),
                Color::from_rgba(0.15, 0.15, 0.25, 0.5)
            );
        }
    }

    fn draw_syntax_errors(&self, frame: &mut iced::widget::canvas::Frame, text_area: Rectangle) {
        let errors = self.syntax_highlighter.validate_syntax(&self.state.content);
        
        for _error in errors {
            // Convert error range to visual position and draw underline
            // This would need more complex logic to handle multi-line errors
            let line = 0; // Simplified for now
            let y = text_area.y + line as f32 * self.line_height + self.line_height - 2.0;
            
            frame.stroke(&Path::line(
                Point::new(text_area.x, y),
                Point::new(text_area.x + 100.0, y) // Simplified width
            ), Stroke::default().with_width(2.0).with_color(Color::from_rgb(1.0, 0.3, 0.3)));
        }
    }

    fn invalidate_cache(&mut self) {
        self.cache.clear();
        self.needs_redraw = true;
    }

    pub fn set_font_size(&mut self, size: u16) {
        self.font_size = size;
        self.line_height = size as f32 * 1.4;
        self.char_width = size as f32 * 0.6;
        self.invalidate_cache();
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        self.syntax_highlighter.set_theme(theme_name);
        self.invalidate_cache();
    }

    pub fn get_content(&self) -> &str {
        &self.state.content
    }

    pub fn can_undo(&self) -> bool {
        self.state.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.state.can_redo()
    }
    
    // Modern text editing features
    
    /// Enable/disable AI-powered suggestions
    pub fn set_ai_suggestions_enabled(&mut self, enabled: bool) {
        self.ai_suggestions_enabled = enabled;
    }
    
    /// Add a new cursor at the specified position for multi-cursor editing
    pub fn add_cursor(&mut self, position: usize) {
        let lines = self.state.get_lines();
        let mut line = 0;
        let mut column = 0;
        let mut pos = 0;
        
        for (i, line_text) in lines.iter().enumerate() {
            if pos + line_text.len() >= position {
                line = i;
                column = position - pos;
                break;
            }
            pos += line_text.len() + 1;
        }
        
        self.multi_cursors.push(CursorState {
            position,
            line,
            column,
            selection_start: None,
            selection_end: None,
        });
    }
    
    /// Remove all multi-cursors
    pub fn clear_multi_cursors(&mut self) {
        self.multi_cursors.clear();
    }
    
    /// Search for text and return results
    pub fn search(&mut self, query: &str) -> Vec<SearchResult> {
        self.search_query = query.to_string();
        self.search_results.clear();
        
        if query.is_empty() {
            return self.search_results.clone();
        }
        
        let content = &self.state.content;
        let lines = self.state.get_lines();
        let mut start = 0;
        
        while let Some(found) = content[start..].find(query) {
            let absolute_pos = start + found;
            let end_pos = absolute_pos + query.len();
            
            // Find line and column
            let mut line = 0;
            let mut pos = 0;
            for (i, line_text) in lines.iter().enumerate() {
                if pos + line_text.len() >= absolute_pos {
                    line = i;
                    break;
                }
                pos += line_text.len() + 1;
            }
            
            let column = absolute_pos - pos;
            
            self.search_results.push(SearchResult {
                start: absolute_pos,
                end: end_pos,
                line,
                column,
                text: query.to_string(),
            });
            
            start = absolute_pos + 1;
        }
        
        self.current_search_index = 0;
        self.search_results.clone()
    }
    
    /// Go to next search result
    pub fn next_search_result(&mut self) -> Option<&SearchResult> {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
            let result = &self.search_results[self.current_search_index];
            self.state.move_cursor(result.start);
            self.state.selection_start = Some(result.start);
            self.state.selection_end = Some(result.end);
            Some(result)
        } else {
            None
        }
    }
    
    /// Go to previous search result
    pub fn prev_search_result(&mut self) -> Option<&SearchResult> {
        if !self.search_results.is_empty() {
            if self.current_search_index == 0 {
                self.current_search_index = self.search_results.len() - 1;
            } else {
                self.current_search_index -= 1;
            }
            let result = &self.search_results[self.current_search_index];
            self.state.move_cursor(result.start);
            self.state.selection_start = Some(result.start);
            self.state.selection_end = Some(result.end);
            Some(result)
        } else {
            None
        }
    }
    
    /// Replace current selection with text
    pub fn replace_current(&mut self, replacement: &str) -> bool {
        if let (Some(start), Some(end)) = (self.state.selection_start, self.state.selection_end) {
            self.state.delete_range(start, end);
            self.state.cursor_position = start;
            self.state.insert_text(replacement);
            true
        } else {
            false
        }
    }
    
    /// Replace all occurrences of search query with replacement
    pub fn replace_all(&mut self, replacement: &str) -> usize {
        let mut count = 0;
        // Work backwards to avoid position shifts
        for result in self.search_results.iter().rev() {
            self.state.delete_range(result.start, result.end);
            self.state.cursor_position = result.start;
            self.state.insert_text(replacement);
            count += 1;
        }
        self.search_results.clear();
        count
    }
    
    /// Toggle folding for a line range
    pub fn toggle_fold(&mut self, start_line: usize, _end_line: usize) {
        if self.folded_regions.contains(&start_line) {
            self.folded_regions.remove(&start_line);
        } else {
            self.folded_regions.insert(start_line);
        }
        self.invalidate_cache();
    }
    
    /// Check if a line is folded
    pub fn is_folded(&self, line: usize) -> bool {
        self.folded_regions.contains(&line)
    }
    
    /// Add collaboration cursor for live editing
    pub fn add_collaboration_cursor(&mut self, user_id: String, user_name: String, position: usize, color: Color) {
        self.collaboration_cursors.insert(user_id.clone(), CollaborationCursor {
            user_id,
            user_name,
            position,
            color,
            last_updated: std::time::Instant::now(),
        });
        self.invalidate_cache();
    }
    
    /// Remove collaboration cursor
    pub fn remove_collaboration_cursor(&mut self, user_id: &str) {
        self.collaboration_cursors.remove(user_id);
        self.invalidate_cache();
    }
    
    /// Enable/disable line wrapping
    pub fn set_line_wrapping(&mut self, enabled: bool) {
        self.wrap_lines = enabled;
        self.invalidate_cache();
    }
    
    /// Enable/disable minimap
    pub fn set_minimap_enabled(&mut self, enabled: bool) {
        self.show_minimap = enabled;
        self.invalidate_cache();
    }
    
    /// Enable/disable bracket matching
    pub fn set_bracket_matching(&mut self, enabled: bool) {
        self.bracket_matching = enabled;
        self.invalidate_cache();
    }
    
    /// Enable/disable auto-closing brackets
    pub fn set_auto_closing_brackets(&mut self, enabled: bool) {
        self.auto_closing_brackets = enabled;
    }
    
    /// Get current search query
    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }
    
    /// Get number of search results
    pub fn search_result_count(&self) -> usize {
        self.search_results.len()
    }
    
    /// Get current search result index (0-based)
    pub fn current_search_index(&self) -> usize {
        self.current_search_index
    }
}

impl<Message> canvas::Program<Message> for ModernTextEditor {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        self.draw_canvas(renderer, bounds, cursor)
    }
}

// Note: key_to_char function is no longer needed as we handle Character variants directly
