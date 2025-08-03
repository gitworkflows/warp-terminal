use iced::Color;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct SyntaxHighlighter {
    rules: Vec<HighlightRule>,
    themes: HashMap<String, SyntaxTheme>,
    current_theme: String,
}

#[derive(Debug, Clone)]
pub struct HighlightRule {
    pub pattern: regex::Regex,
    pub token_type: TokenType,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Shell tokens
    Command,
    Subcommand,
    Flag,
    Option,
    Argument,
    Path,
    String,
    Number,
    Variable,
    Comment,
    Operator,
    Pipe,
    Redirect,
    Background,
    
    // Generic tokens
    Keyword,
    Identifier,
    Literal,
    Punctuation,
    Error,
    Default,
}

#[derive(Debug, Clone)]
pub struct HighlightedSpan {
    pub range: Range<usize>,
    pub token_type: TokenType,
    pub color: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub name: String,
    pub colors: HashMap<TokenType, TokenStyle>,
}

#[derive(Debug, Clone)]
pub struct TokenStyle {
    pub color: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let mut highlighter = Self {
            rules: Vec::new(),
            themes: HashMap::new(),
            current_theme: "dark".to_string(),
        };
        
        highlighter.load_default_rules();
        highlighter.load_default_themes();
        
        highlighter
    }

    pub fn highlight(&self, text: &str) -> Vec<HighlightedSpan> {
        let mut spans = Vec::new();
        let mut covered_ranges = Vec::new();
        
        // Apply highlighting rules in priority order
        let mut rules = self.rules.clone();
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for rule in &rules {
            for mat in rule.pattern.find_iter(text) {
                let range = mat.start()..mat.end();
                
                // Check if this range overlaps with already covered ranges
                if !self.overlaps_with_covered(&range, &covered_ranges) {
                    let theme = self.get_current_theme();
                    let default_style = TokenStyle::default();
                    let style = theme.colors.get(&rule.token_type)
                        .unwrap_or(&default_style);
                    
                    spans.push(HighlightedSpan {
                        range: range.clone(),
                        token_type: rule.token_type.clone(),
                        color: style.color,
                        bold: style.bold,
                        italic: style.italic,
                        underline: style.underline,
                    });
                    
                    covered_ranges.push(range);
                }
            }
        }
        
        // Sort spans by start position
        spans.sort_by(|a, b| a.range.start.cmp(&b.range.start));
        
        spans
    }

    pub fn highlight_command_line(&self, text: &str) -> Vec<HighlightedSpan> {
        let mut spans = Vec::new();
        
        // Parse shell command structure
        let tokens = self.tokenize_shell_command(text);
        
        for token in tokens {
            let theme = self.get_current_theme();
            let default_style = TokenStyle::default();
            let style = theme.colors.get(&token.token_type)
                .unwrap_or(&default_style);
            
            spans.push(HighlightedSpan {
                range: token.range,
                token_type: token.token_type,
                color: style.color,
                bold: style.bold,
                italic: style.italic,
                underline: style.underline,
            });
        }
        
        spans
    }

    pub fn set_theme(&mut self, theme_name: &str) {
        if self.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
        }
    }

    pub fn get_available_themes(&self) -> Vec<&String> {
        self.themes.keys().collect()
    }

    pub fn validate_syntax(&self, text: &str) -> Vec<SyntaxError> {
        let mut errors = Vec::new();
        
        // Basic shell syntax validation
        let mut paren_count = 0;
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut string_char = '\0';
        let mut escape_next = false;
        
        for (i, ch) in text.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            if ch == '\\' {
                escape_next = true;
                continue;
            }
            
            if in_string {
                if ch == string_char {
                    in_string = false;
                }
                continue;
            }
            
            match ch {
                '"' | '\'' => {
                    in_string = true;
                    string_char = ch;
                },
                '(' => paren_count += 1,
                ')' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        errors.push(SyntaxError {
                            range: i..i+1,
                            message: "Unmatched closing parenthesis".to_string(),
                            error_type: SyntaxErrorType::UnmatchedDelimiter,
                        });
                    }
                },
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count < 0 {
                        errors.push(SyntaxError {
                            range: i..i+1,
                            message: "Unmatched closing brace".to_string(),
                            error_type: SyntaxErrorType::UnmatchedDelimiter,
                        });
                    }
                },
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count < 0 {
                        errors.push(SyntaxError {
                            range: i..i+1,
                            message: "Unmatched closing bracket".to_string(),
                            error_type: SyntaxErrorType::UnmatchedDelimiter,
                        });
                    }
                },
                _ => {}
            }
        }
        
        // Check for unclosed delimiters
        if paren_count > 0 {
            errors.push(SyntaxError {
                range: text.len()..text.len(),
                message: format!("{} unclosed parenthesis", paren_count),
                error_type: SyntaxErrorType::UnmatchedDelimiter,
            });
        }
        
        if brace_count > 0 {
            errors.push(SyntaxError {
                range: text.len()..text.len(),
                message: format!("{} unclosed brace", brace_count),
                error_type: SyntaxErrorType::UnmatchedDelimiter,
            });
        }
        
        if bracket_count > 0 {
            errors.push(SyntaxError {
                range: text.len()..text.len(),
                message: format!("{} unclosed bracket", bracket_count),
                error_type: SyntaxErrorType::UnmatchedDelimiter,
            });
        }
        
        if in_string {
            errors.push(SyntaxError {
                range: text.len()..text.len(),
                message: "Unclosed string".to_string(),
                error_type: SyntaxErrorType::UnterminatedString,
            });
        }
        
        errors
    }

    // Private methods

    fn load_default_rules(&mut self) {
        use regex::Regex;
        
        // Shell command patterns
        self.rules.extend(vec![
            // Comments
            HighlightRule {
                pattern: Regex::new(r"#.*$").unwrap(),
                token_type: TokenType::Comment,
                priority: 90,
            },
            
            // Strings
            HighlightRule {
                pattern: Regex::new(r#""([^"\\]|\\.)*""#).unwrap(),
                token_type: TokenType::String,
                priority: 85,
            },
            HighlightRule {
                pattern: Regex::new(r"'([^'\\]|\\.)*'").unwrap(),
                token_type: TokenType::String,
                priority: 85,
            },
            
            // Variables
            HighlightRule {
                pattern: Regex::new(r"\$\{[^}]+\}").unwrap(),
                token_type: TokenType::Variable,
                priority: 80,
            },
            HighlightRule {
                pattern: Regex::new(r"\$[A-Za-z_][A-Za-z0-9_]*").unwrap(),
                token_type: TokenType::Variable,
                priority: 80,
            },
            
            // Numbers
            HighlightRule {
                pattern: Regex::new(r"\b\d+\b").unwrap(),
                token_type: TokenType::Number,
                priority: 75,
            },
            
            // Pipes and redirects
            HighlightRule {
                pattern: Regex::new(r"\|").unwrap(),
                token_type: TokenType::Pipe,
                priority: 70,
            },
            HighlightRule {
                pattern: Regex::new(r">>?|<").unwrap(),
                token_type: TokenType::Redirect,
                priority: 70,
            },
            
            // Background processes
            HighlightRule {
                pattern: Regex::new(r"&").unwrap(),
                token_type: TokenType::Background,
                priority: 70,
            },
            
            // Flags and options
            HighlightRule {
                pattern: Regex::new(r"--[a-zA-Z0-9-]+").unwrap(),
                token_type: TokenType::Flag,
                priority: 65,
            },
            HighlightRule {
                pattern: Regex::new(r"-[a-zA-Z0-9]+").unwrap(),
                token_type: TokenType::Option,
                priority: 65,
            },
            
            // File paths
            HighlightRule {
                pattern: Regex::new(r"[~/]?[a-zA-Z0-9_./\-]+/[a-zA-Z0-9_./\-]*").unwrap(),
                token_type: TokenType::Path,
                priority: 60,
            },
            
            // Common commands (at word boundaries)
            HighlightRule {
                pattern: Regex::new(r"\b(ls|cd|pwd|mkdir|rmdir|rm|cp|mv|cat|less|more|head|tail|grep|find|sort|uniq|wc|awk|sed|cut|tr|xargs|which|whereis|locate|file|stat|chmod|chown|chgrp|tar|gzip|gunzip|zip|unzip|curl|wget|ssh|scp|rsync|git|cargo|npm|node|python|python3|pip|pip3|docker|kubectl|helm|terraform|ansible|make|cmake|gcc|g++|clang|java|javac|go|rustc|ruby|perl|php|bash|zsh|fish|sh|sudo|su|ps|top|htop|kill|killall|jobs|fg|bg|nohup|screen|tmux|vim|nvim|emacs|nano|code|open|pbcopy|pbpaste|echo|printf|date|cal|uptime|whoami|id|groups|history|alias|unalias|export|unset|source|type|man|info|help)\b").unwrap(),
                token_type: TokenType::Command,
                priority: 55,
            },
            
            // Operators
            HighlightRule {
                pattern: Regex::new(r"[=+\-*/]").unwrap(),
                token_type: TokenType::Operator,
                priority: 50,
            },
        ]);
    }

    fn load_default_themes(&mut self) {
        // Dark theme
        let mut dark_colors = HashMap::new();
        dark_colors.insert(TokenType::Command, TokenStyle {
            color: Color::from_rgb(0.4, 0.8, 1.0), // Light blue
            bold: true,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::Subcommand, TokenStyle {
            color: Color::from_rgb(0.6, 0.9, 0.6), // Light green
            bold: false,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::Flag, TokenStyle {
            color: Color::from_rgb(1.0, 0.8, 0.4), // Orange
            bold: false,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::Option, TokenStyle {
            color: Color::from_rgb(1.0, 0.8, 0.4), // Orange
            bold: false,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::String, TokenStyle {
            color: Color::from_rgb(0.8, 1.0, 0.8), // Light green
            bold: false,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::Number, TokenStyle {
            color: Color::from_rgb(1.0, 0.6, 0.8), // Pink
            bold: false,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::Variable, TokenStyle {
            color: Color::from_rgb(0.8, 0.6, 1.0), // Purple
            bold: false,
            italic: true,
            underline: false,
        });
        dark_colors.insert(TokenType::Comment, TokenStyle {
            color: Color::from_rgb(0.5, 0.5, 0.5), // Gray
            bold: false,
            italic: true,
            underline: false,
        });
        dark_colors.insert(TokenType::Path, TokenStyle {
            color: Color::from_rgb(0.6, 0.8, 1.0), // Light blue
            bold: false,
            italic: false,
            underline: true,
        });
        dark_colors.insert(TokenType::Pipe, TokenStyle {
            color: Color::from_rgb(1.0, 1.0, 0.4), // Yellow
            bold: true,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::Redirect, TokenStyle {
            color: Color::from_rgb(1.0, 0.4, 0.4), // Red
            bold: true,
            italic: false,
            underline: false,
        });
        dark_colors.insert(TokenType::Error, TokenStyle {
            color: Color::from_rgb(1.0, 0.3, 0.3), // Red
            bold: true,
            italic: false,
            underline: true,
        });
        dark_colors.insert(TokenType::Default, TokenStyle {
            color: Color::from_rgb(0.9, 0.9, 0.9), // Light gray
            bold: false,
            italic: false,
            underline: false,
        });
        
        self.themes.insert("dark".to_string(), SyntaxTheme {
            name: "Dark".to_string(),
            colors: dark_colors,
        });
        
        // Light theme
        let mut light_colors = HashMap::new();
        light_colors.insert(TokenType::Command, TokenStyle {
            color: Color::from_rgb(0.0, 0.3, 0.8), // Dark blue
            bold: true,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::Subcommand, TokenStyle {
            color: Color::from_rgb(0.0, 0.6, 0.0), // Dark green
            bold: false,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::Flag, TokenStyle {
            color: Color::from_rgb(0.8, 0.4, 0.0), // Dark orange
            bold: false,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::Option, TokenStyle {
            color: Color::from_rgb(0.8, 0.4, 0.0), // Dark orange
            bold: false,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::String, TokenStyle {
            color: Color::from_rgb(0.0, 0.5, 0.0), // Dark green
            bold: false,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::Number, TokenStyle {
            color: Color::from_rgb(0.8, 0.0, 0.4), // Dark pink
            bold: false,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::Variable, TokenStyle {
            color: Color::from_rgb(0.4, 0.0, 0.8), // Dark purple
            bold: false,
            italic: true,
            underline: false,
        });
        light_colors.insert(TokenType::Comment, TokenStyle {
            color: Color::from_rgb(0.4, 0.4, 0.4), // Dark gray
            bold: false,
            italic: true,
            underline: false,
        });
        light_colors.insert(TokenType::Path, TokenStyle {
            color: Color::from_rgb(0.0, 0.2, 0.6), // Dark blue
            bold: false,
            italic: false,
            underline: true,
        });
        light_colors.insert(TokenType::Pipe, TokenStyle {
            color: Color::from_rgb(0.8, 0.6, 0.0), // Dark yellow
            bold: true,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::Redirect, TokenStyle {
            color: Color::from_rgb(0.8, 0.0, 0.0), // Dark red
            bold: true,
            italic: false,
            underline: false,
        });
        light_colors.insert(TokenType::Error, TokenStyle {
            color: Color::from_rgb(0.8, 0.0, 0.0), // Dark red
            bold: true,
            italic: false,
            underline: true,
        });
        light_colors.insert(TokenType::Default, TokenStyle {
            color: Color::from_rgb(0.1, 0.1, 0.1), // Dark gray
            bold: false,
            italic: false,
            underline: false,
        });
        
        self.themes.insert("light".to_string(), SyntaxTheme {
            name: "Light".to_string(),
            colors: light_colors,
        });
    }

    fn get_current_theme(&self) -> &SyntaxTheme {
        self.themes.get(&self.current_theme).unwrap()
    }

    fn overlaps_with_covered(&self, range: &Range<usize>, covered: &[Range<usize>]) -> bool {
        covered.iter().any(|covered_range| {
            range.start < covered_range.end && range.end > covered_range.start
        })
    }

    fn tokenize_shell_command(&self, text: &str) -> Vec<ShellToken> {
        let mut tokens = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_pos = 0;
        
        for (i, word) in words.iter().enumerate() {
            // Find the actual position in the original string
            let word_start = text[current_pos..].find(word).unwrap_or(0) + current_pos;
            let word_end = word_start + word.len();
            
            let token_type = if i == 0 {
                TokenType::Command
            } else if word.starts_with("--") {
                TokenType::Flag
            } else if word.starts_with('-') {
                TokenType::Option
            } else if word.contains('/') || word.starts_with('~') || word.starts_with('.') {
                TokenType::Path
            } else if word.parse::<f64>().is_ok() {
                TokenType::Number
            } else if word.starts_with('"') || word.starts_with('\'') {
                TokenType::String
            } else if word.starts_with('$') {
                TokenType::Variable
            } else {
                TokenType::Argument
            };
            
            tokens.push(ShellToken {
                range: word_start..word_end,
                token_type,
            });
            
            current_pos = word_end;
        }
        
        tokens
    }
}

#[derive(Debug, Clone)]
struct ShellToken {
    range: Range<usize>,
    token_type: TokenType,
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub range: Range<usize>,
    pub message: String,
    pub error_type: SyntaxErrorType,
}

#[derive(Debug, Clone)]
pub enum SyntaxErrorType {
    UnmatchedDelimiter,
    UnterminatedString,
    InvalidSyntax,
    UnknownCommand,
}

impl Default for TokenStyle {
    fn default() -> Self {
        Self {
            color: Color::from_rgb(0.9, 0.9, 0.9),
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let spans = highlighter.highlight("ls -la /home/user");
        
        assert!(!spans.is_empty());
        // Should highlight 'ls' as command, '-la' as option, '/home/user' as path
    }

    #[test]
    fn test_syntax_validation() {
        let highlighter = SyntaxHighlighter::new();
        let errors = highlighter.validate_syntax("echo \"unclosed string");
        
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0].error_type, SyntaxErrorType::UnterminatedString));
    }

    #[test]
    fn test_theme_switching() {
        let mut highlighter = SyntaxHighlighter::new();
        
        assert_eq!(highlighter.current_theme, "dark");
        
        highlighter.set_theme("light");
        assert_eq!(highlighter.current_theme, "light");
        
        // Test invalid theme
        highlighter.set_theme("nonexistent");
        assert_eq!(highlighter.current_theme, "light"); // Should remain unchanged
    }
}
