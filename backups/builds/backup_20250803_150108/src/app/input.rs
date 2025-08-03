use crate::utils::editor::Suggestion;
use crate::utils::syntax::SyntaxTree;
use iced::Theme;
use iced::widget::text_input;
use iced::{Element};

#[derive(Debug, Default)]
pub struct EnhancedInput {
    pub input_value: String,
    pub suggestions: Vec<Suggestion>,
    pub syntax_tree: Option<SyntaxTree>,
}

impl EnhancedInput {
    pub fn view(&self, _theme: &Theme) -> Element<crate::Message> {
        let input = text_input("Enter command...", &self.input_value)
            .on_input(crate::Message::InputChanged);
        
        input.into()
    }
}

