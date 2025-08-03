use iced::{Element, Color};
use iced::widget::text;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SyntaxHighlighter {
    keywords: HashMap<String, Color>,
    operators: HashMap<String, Color>,
    strings_color: Color,
    comments_color: Color,
    numbers_color: Color,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        let mut keywords = HashMap::new();
        
        // Shell keywords
        keywords.insert("if".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("then".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("else".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("elif".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("fi".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("for".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("while".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("do".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("done".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("case".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("esac".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        keywords.insert("function".to_string(), Color::from_rgb(0.8, 0.4, 0.8));
        
        // Common commands
        keywords.insert("ls".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("cd".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("pwd".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("mkdir".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("rm".to_string(), Color::from_rgb(0.8, 0.4, 0.4));
        keywords.insert("cp".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("mv".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("cat".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("grep".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("find".to_string(), Color::from_rgb(0.4, 0.8, 0.4));
        keywords.insert("git".to_string(), Color::from_rgb(0.8, 0.6, 0.2));
        keywords.insert("cargo".to_string(), Color::from_rgb(0.8, 0.6, 0.2));
        keywords.insert("npm".to_string(), Color::from_rgb(0.8, 0.6, 0.2));
        keywords.insert("docker".to_string(), Color::from_rgb(0.2, 0.6, 0.8));
        
        let mut operators = HashMap::new();
        operators.insert("|".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        operators.insert("&&".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        operators.insert("||".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        operators.insert(">".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        operators.insert(">>".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        operators.insert("<".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        operators.insert("&".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        operators.insert(";".to_string(), Color::from_rgb(0.6, 0.6, 0.6));
        
        Self {
            keywords,
            operators,
            strings_color: Color::from_rgb(0.8, 0.8, 0.4),
            comments_color: Color::from_rgb(0.5, 0.5, 0.5),
            numbers_color: Color::from_rgb(0.4, 0.6, 0.8),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub color: Option<Color>,
    pub is_bold: bool,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut chars = input.chars().peekable();
        let mut in_string = false;
        let mut string_char = '"';
        let mut in_comment = false;
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' if !in_comment => {
                    if in_string && ch == string_char {
                        current_token.push(ch);
                        tokens.push(Token {
                            text: current_token.clone(),
                            color: Some(self.strings_color),
                            is_bold: false,
                        });
                        current_token.clear();
                        in_string = false;
                    } else if !in_string {
                        if !current_token.is_empty() {
                            self.push_token(&mut tokens, &current_token);
                            current_token.clear();
                        }
                        current_token.push(ch);
                        in_string = true;
                        string_char = ch;
                    } else {
                        current_token.push(ch);
                    }
                }
                '#' if !in_string => {
                    if !current_token.is_empty() {
                        self.push_token(&mut tokens, &current_token);
                        current_token.clear();
                    }
                    in_comment = true;
                    current_token.push(ch);
                }
                ' ' | '\t' if !in_string && !in_comment => {
                    if !current_token.is_empty() {
                        self.push_token(&mut tokens, &current_token);
                        current_token.clear();
                    }
                    tokens.push(Token {
                        text: ch.to_string(),
                        color: None,
                        is_bold: false,
                    });
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }
        
        if !current_token.is_empty() {
            if in_comment {
                tokens.push(Token {
                    text: current_token,
                    color: Some(self.comments_color),
                    is_bold: false,
                });
            } else {
                self.push_token(&mut tokens, &current_token);
            }
        }
        
        tokens
    }
    
    fn push_token(&self, tokens: &mut Vec<Token>, text: &str) {
        let color = if let Some(&color) = self.keywords.get(text) {
            Some(color)
        } else if self.operators.contains_key(text) {
            Some(self.operators[text])
        } else if text.chars().all(|c| c.is_ascii_digit() || c == '.') {
            Some(self.numbers_color)
        } else if text.starts_with('-') && text.len() > 1 {
            // Command flags
            Some(Color::from_rgb(0.6, 0.8, 0.8))
        } else {
            None
        };
        
        tokens.push(Token {
            text: text.to_string(),
            color,
            is_bold: false,
        });
    }
    
    pub fn highlight_text<'a>(&self, input: &'a str) -> Element<'a, crate::Message> {
        let _tokens = self.tokenize(input);
        
        // For now, return simple colored text
        // In a full implementation, you would use Rich text widget
        text(input).into()
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxTree {
    pub root: SyntaxNode,
}

#[derive(Debug, Clone)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub text: String,
    pub children: Vec<SyntaxNode>,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxKind {
    Command,
    Argument,
    Flag,
    String,
    Number,
    Operator,
    Pipe,
    Redirect,
    Comment,
    Variable,
    Path,
    Error,
}

impl SyntaxTree {
    pub fn parse(input: &str) -> Self {
        let mut parser = Parser::new(input);
        let root = parser.parse();
        
        Self { root }
    }
}

#[allow(dead_code)]
struct Parser {
    input: String,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            pos: 0,
        }
    }
    
    fn parse(&mut self) -> SyntaxNode {
        // Simple parsing implementation
        // In a real implementation, this would be much more sophisticated
        SyntaxNode {
            kind: SyntaxKind::Command,
            text: self.input.clone(),
            children: Vec::new(),
            span: (0, self.input.len()),
        }
    }
}
