use std::collections::HashMap;
use crate::model::history::HistoryManager;
use crate::utils::fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone)]
pub struct SuggestionEngine {
    commands: HashMap<String, Vec<CommandCompletion>>,
    fuzzy_matcher: FuzzyMatcher,
    path_completer: PathCompleter,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub command: String,
    pub description: String,
    pub score: f32,
    pub source: SuggestionSource,
    pub suggestion_type: SuggestionType,
    pub insert_text: String,
    pub cursor_offset: usize,
}

#[derive(Debug, Clone)]
pub enum SuggestionSource {
    History,
    StaticCommands,
    PathCompletion,
    OptionCompletion,
    Workflow,
}

#[derive(Debug, Clone)]
pub enum SuggestionType {
    Command,
    Subcommand,
    Option,
    Path,
    Argument,
}

#[derive(Debug, Clone)]
pub struct CommandCompletion {
    pub name: String,
    pub description: String,
    pub options: Vec<OptionCompletion>,
    pub subcommands: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OptionCompletion {
    pub name: String,
    pub short: Option<String>,
    pub description: String,
    pub takes_value: bool,
}

#[derive(Debug, Clone)]
pub struct PathCompleter {
    current_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct DynamicSuggestionContext {
    pub input: String,
    pub cursor_position: usize,
    pub current_word: String,
    pub previous_words: Vec<String>,
    pub is_option: bool,
    pub is_path: bool,
}

impl PathCompleter {
    pub fn new() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")),
        }
    }
    
    pub fn complete_path(&self, prefix: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        let path = if prefix.starts_with('/') {
            std::path::PathBuf::from(prefix)
        } else if prefix.starts_with('~') {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
            std::path::PathBuf::from(prefix.replacen('~', &home, 1))
        } else {
            self.current_dir.join(prefix)
        };
        
        let parent = path.parent().unwrap_or(&self.current_dir);
        let filename = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(&filename.to_string()) {
                    let is_dir = entry.path().is_dir();
                    let display_name = if is_dir { format!("{}/", name) } else { name.clone() };
                    
                    suggestions.push(Suggestion {
                        command: display_name.clone(),
                        description: if is_dir { "Directory".to_string() } else { "File".to_string() },
                        score: 20.0,
                        source: SuggestionSource::PathCompletion,
                        suggestion_type: SuggestionType::Path,
                        insert_text: display_name,
                        cursor_offset: 0,
                    });
                }
            }
        }
        
        suggestions
    }
}

impl SuggestionEngine {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        
        // Load comprehensive command completions
        commands.insert("git".to_string(), vec![
            CommandCompletion {
                name: "status".to_string(),
                description: "Show the working tree status".to_string(),
                options: vec![
                    OptionCompletion {
                        name: "--short".to_string(),
                        short: Some("-s".to_string()),
                        description: "Give the output in the short-format".to_string(),
                        takes_value: false,
                    },
                    OptionCompletion {
                        name: "--branch".to_string(),
                        short: Some("-b".to_string()),
                        description: "Show the branch and tracking info".to_string(),
                        takes_value: false,
                    },
                ],
                subcommands: vec![],
            },
            CommandCompletion {
                name: "add".to_string(),
                description: "Add file contents to the index".to_string(),
                options: vec![
                    OptionCompletion {
                        name: "--all".to_string(),
                        short: Some("-A".to_string()),
                        description: "Add changes from all tracked and untracked files".to_string(),
                        takes_value: false,
                    },
                ],
                subcommands: vec![],
            },
        ]);
        
        commands.insert("cargo".to_string(), vec![
            CommandCompletion {
                name: "build".to_string(),
                description: "Compile the current package".to_string(),
                options: vec![
                    OptionCompletion {
                        name: "--release".to_string(),
                        short: None,
                        description: "Build artifacts in release mode".to_string(),
                        takes_value: false,
                    },
                ],
                subcommands: vec![],
            },
            CommandCompletion {
                name: "run".to_string(),
                description: "Run a binary or example of the local package".to_string(),
                options: vec![],
                subcommands: vec![],
            },
        ]);
        
        Self {
            commands,
            fuzzy_matcher: FuzzyMatcher::new(),
            path_completer: PathCompleter::new(),
        }
    }

    /// Generate real-time suggestions based on current input context
    pub fn suggest_dynamic(&self, context: &DynamicSuggestionContext, history: &HistoryManager) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Parse input to understand context
        let parsed_context = self.parse_input_context(&context.input, context.cursor_position);
        
        // Generate suggestions based on context
        match parsed_context {
            InputContext::CommandStart(prefix) => {
                suggestions.extend(self.suggest_commands(&prefix, history));
                suggestions.extend(self.suggest_from_history(&prefix, history));
            },
            InputContext::Subcommand(command, prefix) => {
                suggestions.extend(self.suggest_subcommands(&command, &prefix));
            },
            InputContext::Option(command, prefix) => {
                suggestions.extend(self.suggest_options(&command, &prefix));
            },
            InputContext::Path(prefix) => {
                suggestions.extend(self.suggest_paths(&prefix));
            },
            InputContext::Argument(command, arg_pos, prefix) => {
                suggestions.extend(self.suggest_arguments(&command, arg_pos, &prefix));
            },
        }
        
        // Sort by relevance score
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to top 10 suggestions for performance
        suggestions.truncate(10);
        
        suggestions
    }

    /// Legacy suggest method for backward compatibility
    pub fn suggest(&self, input: &str) -> Vec<Suggestion> {
        let context = DynamicSuggestionContext {
            input: input.to_string(),
            cursor_position: input.len(),
            current_word: self.get_current_word(input, input.len()),
            previous_words: self.get_previous_words(input, input.len()),
            is_option: input.trim_end().ends_with('-'),
            is_path: self.looks_like_path(input),
        };
        
        // Create a dummy history manager for compatibility
        let history = HistoryManager::new();
        self.suggest_dynamic(&context, &history)
    }

    /// Suggest commands based on prefix matching
    fn suggest_commands(&self, prefix: &str, history: &HistoryManager) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Static command suggestions
        for (command, _completions) in &self.commands {
            if command.starts_with(prefix) || self.fuzzy_matcher.matches(prefix, command) {
                let score = self.calculate_command_score(prefix, command, history);
                suggestions.push(Suggestion {
                    command: command.clone(),
                    description: format!("Run {} command", command),
                    score,
                    source: SuggestionSource::StaticCommands,
                    suggestion_type: SuggestionType::Command,
                    insert_text: command.clone(),
                    cursor_offset: 0,
                });
            }
        }
        
        suggestions
    }

    /// Suggest from command history
    fn suggest_from_history(&self, prefix: &str, history: &HistoryManager) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Get recent commands that match prefix
        let matches = history.search_with_prefix(prefix);
        
        for (i, entry) in matches.iter().take(5).enumerate() {
            let score = 50.0 - (i as f32 * 5.0); // Recent commands get higher scores
            suggestions.push(Suggestion {
                command: entry.command.clone(),
                description: format!("From history: {}", self.format_relative_time(entry.timestamp)),
                score,
                source: SuggestionSource::History,
                suggestion_type: SuggestionType::Command,
                insert_text: entry.command.clone(),
                cursor_offset: 0,
            });
        }
        
        suggestions
    }

    /// Suggest subcommands for a given command
    fn suggest_subcommands(&self, command: &str, prefix: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        if let Some(completions) = self.commands.get(command) {
            for completion in completions {
                if completion.name.starts_with(prefix) {
                    suggestions.push(Suggestion {
                        command: format!("{} {}", command, completion.name),
                        description: completion.description.clone(),
                        score: 40.0,
                        source: SuggestionSource::StaticCommands,
                        suggestion_type: SuggestionType::Subcommand,
                        insert_text: completion.name.clone(),
                        cursor_offset: 0,
                    });
                }
            }
        }
        
        suggestions
    }

    /// Suggest options for a given command
    fn suggest_options(&self, command: &str, prefix: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        if let Some(completions) = self.commands.get(command) {
            for completion in completions {
                for option in &completion.options {
                    if option.name.starts_with(prefix) {
                        suggestions.push(Suggestion {
                            command: format!("{} {}", command, option.name),
                            description: option.description.clone(),
                            score: 30.0,
                            source: SuggestionSource::OptionCompletion,
                            suggestion_type: SuggestionType::Option,
                            insert_text: option.name.clone(),
                            cursor_offset: 0,
                        });
                    }
                    
                    // Also suggest short options
                    if let Some(ref short) = option.short {
                        if short.starts_with(prefix) {
                            suggestions.push(Suggestion {
                                command: format!("{} {}", command, short),
                                description: format!("{} (short for {})", option.description, option.name),
                                score: 25.0,
                                source: SuggestionSource::OptionCompletion,
                                suggestion_type: SuggestionType::Option,
                                insert_text: short.clone(),
                                cursor_offset: 0,
                            });
                        }
                    }
                }
            }
        }
        
        suggestions
    }

    /// Suggest file and directory paths
    fn suggest_paths(&self, prefix: &str) -> Vec<Suggestion> {
        self.path_completer.complete_path(prefix)
    }

    /// Suggest command arguments
    fn suggest_arguments(&self, _command: &str, _position: usize, _prefix: &str) -> Vec<Suggestion> {
        // This would be expanded based on specific command argument patterns
        vec![]
    }

    // Helper methods
    
    fn parse_input_context(&self, input: &str, cursor_pos: usize) -> InputContext {
        let trimmed = input.trim();
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        
        if words.is_empty() {
            return InputContext::CommandStart("".to_string());
        }
        
        let current_word = self.get_current_word(input, cursor_pos);
        
        // Determine context based on position and content
        if words.len() == 1 && !trimmed.ends_with(' ') {
            InputContext::CommandStart(current_word)
        } else if current_word.starts_with('-') {
            InputContext::Option(words[0].to_string(), current_word)
        } else if self.looks_like_path(&current_word) {
            InputContext::Path(current_word)
        } else if words.len() >= 2 {
            if let Some(completions) = self.commands.get(words[0]) {
                let has_matching_subcommand = completions.iter().any(|c| c.name == words[1]);
                if !has_matching_subcommand && words.len() == 2 {
                    InputContext::Subcommand(words[0].to_string(), current_word)
                } else {
                    InputContext::Argument(words[0].to_string(), words.len() - 1, current_word)
                }
            } else {
                InputContext::Argument(words[0].to_string(), words.len() - 1, current_word)
            }
        } else {
            InputContext::CommandStart(current_word)
        }
    }
    
    fn get_current_word(&self, input: &str, cursor_pos: usize) -> String {
        let chars: Vec<char> = input.chars().collect();
        let mut start = cursor_pos;
        let mut end = cursor_pos;
        
        // Find word boundaries
        while start > 0 && chars[start - 1] != ' ' {
            start -= 1;
        }
        while end < chars.len() && chars[end] != ' ' {
            end += 1;
        }
        
        chars[start..end].iter().collect()
    }
    
    fn get_previous_words(&self, input: &str, cursor_pos: usize) -> Vec<String> {
        let before_cursor = &input[..cursor_pos];
        before_cursor
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
    
    fn looks_like_path(&self, text: &str) -> bool {
        text.contains('/') || text.starts_with('~') || text.starts_with('.')
    }
    
    fn calculate_command_score(&self, prefix: &str, command: &str, history: &HistoryManager) -> f32 {
        let mut score = 0.0;
        
        // Exact prefix match gets highest score
        if command.starts_with(prefix) {
            score += 60.0;
            
            // Exact match gets bonus
            if command == prefix {
                score += 20.0;
            }
        }
        
        // Fuzzy match gets lower score
        if self.fuzzy_matcher.matches(prefix, command) {
            score += 30.0;
        }
        
        // Bonus for commands in history
        let recent_commands = history.get_recent_commands(50);
        for entry in recent_commands {
            if entry.command.starts_with(command) {
                score += 10.0;
                break;
            }
        }
        
        score
    }
    
    fn format_relative_time(&self, timestamp: u64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let diff = now.saturating_sub(timestamp);
        
        match diff {
            0..=60 => "just now".to_string(),
            61..=3600 => format!("{}m ago", diff / 60),
            3601..=86400 => format!("{}h ago", diff / 3600),
            _ => format!("{}d ago", diff / 86400),
        }
    }
}

#[derive(Debug, Clone)]
enum InputContext {
    CommandStart(String),
    Subcommand(String, String),
    Option(String, String),
    Path(String),
    Argument(String, usize, String),
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}
