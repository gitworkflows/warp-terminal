use crate::editor::syntax_highlighter::SyntaxHighlighter;
use crate::editor::vim_mode::VimModeManager;
use crate::executor::advanced_commands::AdvancedCommands;
use crate::editor::EditorAction;

pub struct CommandInspector {
    command_executor: AdvancedCommands,
    highlighter: SyntaxHighlighter,
    vim_mode_manager: VimModeManager,
    enabled: bool,
}

impl CommandInspector {
    pub fn new() -> Self {
        Self {
            command_executor: AdvancedCommands::new(),
            highlighter: SyntaxHighlighter::new(),
            vim_mode_manager: VimModeManager::new(),
            enabled: true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn inspect_command(&self, command: &str) -> Vec<EditorAction> {
        // Highlight the syntax
        let highlighted_spans = self.highlighter.highlight(command);

        // Validate syntax
        let syntax_errors = self.highlighter.validate_syntax(command);
        
        // Execute command expansion
        let expanded_command = self.command_executor.expand_alias(command);
        
        // Return actions based on inspection
        let mut actions = Vec::new();

        if highlighted_spans.is_empty() {
            actions.push(EditorAction::ShowMessage("No highlights found".to_string()));
        }

        for error in syntax_errors {
            actions.push(EditorAction::ShowErrorMessage(error.message));
        }

        if expanded_command != command {
            actions.push(EditorAction::ShowMessage(format!("Alias expanded to: {}", expanded_command)));
        }
        
        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_inspection() {
        let inspector = CommandInspector::new();
        let actions = inspector.inspect_command("ls -la /invalid/path");
        
        assert!(!actions.is_empty());
    }
}

