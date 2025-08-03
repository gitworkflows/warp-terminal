use iced::{Element, Color, Length};
use iced::widget::{column, row, text, text_input, container};

use crate::Message;

#[derive(Debug, Clone)]
pub struct EnhancedTextInput {
    pub value: String,
    pub cursor_position: usize,
    pub suggestions: Vec<Suggestion>,
    pub syntax_tree: Option<SyntaxTree>,
    pub show_suggestions: bool,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub description: String,
    pub suggestion_type: SuggestionType,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub enum SuggestionType {
    Command,
    Flag,
    File,
    Directory,
    History,
    Builtin,
}

#[derive(Debug, Clone)]
pub struct SyntaxTree {
    pub tokens: Vec<Token>,
    pub errors: Vec<SyntaxError>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Command,
    Flag,
    Argument,
    String,
    Number,
    Operator,
    Whitespace,
    Comment,
    Error,
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

impl Default for EnhancedTextInput {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            suggestions: Vec::new(),
            syntax_tree: None,
            show_suggestions: false,
        }
    }
}

impl EnhancedTextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = value;
        self.update_syntax_tree();
        self.update_suggestions();
        self
    }

    pub fn view(&self, placeholder: &str) -> Element<Message> {
        let mut content = column![];

        // Main text input
        let input = text_input(placeholder, &self.value)
            .on_input(Message::InputChanged)
            .on_submit(Message::ExecuteCommand)
            .size(16)
            .padding(10)
            .width(Length::Fill);

        content = content.push(input);

        // Show syntax errors if any
        if let Some(ref syntax_tree) = self.syntax_tree {
            if !syntax_tree.errors.is_empty() {
                let error_text = syntax_tree.errors
                    .iter()
                    .map(|err| err.message.clone())
                    .collect::<Vec<_>>()
                    .join("; ");

let error_display = text(format!("⚠️ {}", error_text))
                    .size(12)
                    .style(Color::from_rgb(0.8, 0.4, 0.4));

                content = content.push(error_display);
            }
        }

        // Show suggestions if enabled
        if self.show_suggestions && !self.suggestions.is_empty() {
            content = content.push(self.suggestions_view());
        }

        container(content)
            .width(Length::Fill)
            .into()
    }

    fn suggestions_view(&self) -> Element<Message> {
        let mut suggestions_column = column![].spacing(2);

        for (_i, suggestion) in self.suggestions.iter().take(5).enumerate() {
            let icon = match suggestion.suggestion_type {
                SuggestionType::Command => "⚡",
                SuggestionType::Flag => "🏁",
                SuggestionType::File => "📄",
                SuggestionType::Directory => "📁",
                SuggestionType::History => "🕒",
                SuggestionType::Builtin => "🔧",
            };

            let suggestion_row = row![
                text(icon).size(14),
                text(&suggestion.text).size(14),
text(&suggestion.description)
                    .size(12)
                    .style(Color::from_rgb(0.7, 0.7, 0.7))
            ]
            .spacing(8);

            let suggestion_container = container(suggestion_row)
                .padding(5);

            suggestions_column = suggestions_column.push(suggestion_container);
        }

        container(suggestions_column)
            .padding(5)
            .into()
    }

    pub fn update_syntax_tree(&mut self) {
        self.syntax_tree = Some(self.parse_syntax(&self.value));
    }

    pub fn update_suggestions(&mut self) {
        self.suggestions = self.generate_suggestions(&self.value);
        self.show_suggestions = !self.suggestions.is_empty();
    }

    fn parse_syntax(&self, input: &str) -> SyntaxTree {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        let mut current_pos = 0;

        // Simple tokenization - can be enhanced with a proper parser
        let words: Vec<&str> = input.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            let start = current_pos;
            let end = start + word.len();

            let token_type = if i == 0 {
                // First word is usually a command
                if self.is_valid_command(word) {
                    TokenType::Command
                } else {
                    errors.push(SyntaxError {
                        message: format!("Unknown command: {}", word),
                        start,
                        end,
                    });
                    TokenType::Error
                }
            } else if word.starts_with('-') {
                TokenType::Flag
            } else if word.parse::<i32>().is_ok() || word.parse::<f64>().is_ok() {
                TokenType::Number
            } else if word.starts_with('"') && word.ends_with('"') {
                TokenType::String
            } else {
                TokenType::Argument
            };

            tokens.push(Token {
                text: word.to_string(),
                token_type,
                start,
                end,
            });

            current_pos = end + 1; // +1 for the space
        }

        SyntaxTree { tokens, errors }
    }

    fn generate_suggestions(&self, input: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Get the current word being typed
        let words: Vec<&str> = input.split_whitespace().collect();
        let current_word = words.last().unwrap_or(&"");

        // Command suggestions
        let commands = vec![
            ("ls", "List directory contents"),
            ("cd", "Change directory"),
            ("pwd", "Print working directory"),
            ("cat", "Display file contents"),
            ("grep", "Search text patterns"),
            ("find", "Find files and directories"),
            ("git", "Version control system"),
            ("npm", "Node package manager"),
            ("cargo", "Rust package manager"),
            ("python", "Python interpreter"),
            ("node", "Node.js interpreter"),
            ("vim", "Text editor"),
            ("nano", "Text editor"),
            ("clear", "Clear terminal"),
            ("history", "Show command history"),
        ];

        if words.is_empty() || words.len() == 1 {
            // Suggest commands
            for (cmd, desc) in commands {
                if cmd.starts_with(current_word) {
                    suggestions.push(Suggestion {
                        text: cmd.to_string(),
                        description: desc.to_string(),
                        suggestion_type: SuggestionType::Command,
                        score: self.calculate_fuzzy_score(current_word, cmd),
                    });
                }
            }
        } else {
            // Suggest flags and arguments based on the command
            let command = words[0];
            match command {
                "ls" => {
                    let ls_flags = vec![
                        ("-l", "Use long listing format"),
                        ("-a", "Show hidden files"),
                        ("-h", "Human readable sizes"),
                        ("-t", "Sort by modification time"),
                        ("-r", "Reverse order"),
                    ];
                    
                    for (flag, desc) in ls_flags {
                        if flag.starts_with(current_word) {
                            suggestions.push(Suggestion {
                                text: flag.to_string(),
                                description: desc.to_string(),
                                suggestion_type: SuggestionType::Flag,
                                score: self.calculate_fuzzy_score(current_word, flag),
                            });
                        }
                    }
                }
                "git" => {
                    let git_commands = vec![
                        ("status", "Show working tree status"),
                        ("add", "Add files to staging area"),
                        ("commit", "Record changes to repository"),
                        ("push", "Upload changes to remote"),
                        ("pull", "Download changes from remote"),
                        ("clone", "Clone a repository"),
                        ("branch", "List, create, or delete branches"),
                        ("checkout", "Switch branches or restore files"),
                        ("merge", "Join development histories"),
                        ("log", "Show commit logs"),
                    ];
                    
                    for (subcmd, desc) in git_commands {
                        if subcmd.starts_with(current_word) {
                            suggestions.push(Suggestion {
                                text: subcmd.to_string(),
                                description: desc.to_string(),
                                suggestion_type: SuggestionType::Command,
                                score: self.calculate_fuzzy_score(current_word, subcmd),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        // Sort suggestions by score
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        suggestions
    }

    fn calculate_fuzzy_score(&self, query: &str, target: &str) -> f32 {
        if query.is_empty() {
            return 1.0;
        }
        
        if target.starts_with(query) {
            return 0.9 + (query.len() as f32 / target.len() as f32) * 0.1;
        }
        
        if target.contains(query) {
            return 0.5 + (query.len() as f32 / target.len() as f32) * 0.3;
        }
        
        // Simple character matching
        let mut score = 0.0;
        let mut query_chars = query.chars().peekable();
        
        for target_char in target.chars() {
            if let Some(&query_char) = query_chars.peek() {
                if target_char.to_lowercase().eq(query_char.to_lowercase()) {
                    score += 1.0;
                    query_chars.next();
                }
            }
        }
        
        score / target.len() as f32
    }

    fn is_valid_command(&self, command: &str) -> bool {
        // Simple command validation - can be enhanced with actual PATH lookup
        matches!(command, 
            "ls" | "cd" | "pwd" | "cat" | "grep" | "find" | "git" | "npm" | 
            "cargo" | "python" | "node" | "vim" | "nano" | "clear" | "history" |
            "echo" | "cp" | "mv" | "rm" | "mkdir" | "rmdir" | "chmod" | "chown" |
            "ps" | "kill" | "top" | "htop" | "df" | "du" | "free" | "uname" |
            "curl" | "wget" | "ssh" | "scp" | "rsync" | "tar" | "zip" | "unzip"
        )
    }
}
