use std::collections::HashMap;
use crate::utils::editor::{Suggestion, SuggestionType};

#[derive(Debug, Clone)]
pub struct FuzzyMatcher {
    command_index: HashMap<String, Vec<String>>,
    history_scores: HashMap<String, f32>,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        let mut command_index = HashMap::new();
        
        // Pre-populate with common commands
        command_index.insert("git".to_string(), vec![
            "status".to_string(),
            "add".to_string(),
            "commit".to_string(),
            "push".to_string(),
            "pull".to_string(),
            "branch".to_string(),
            "checkout".to_string(),
            "merge".to_string(),
            "log".to_string(),
            "diff".to_string(),
        ]);
        
        command_index.insert("cargo".to_string(), vec![
            "build".to_string(),
            "run".to_string(),
            "test".to_string(),
            "check".to_string(),
            "clippy".to_string(),
            "fmt".to_string(),
            "clean".to_string(),
            "doc".to_string(),
        ]);
        
        Self {
            command_index,
            history_scores: HashMap::new(),
        }
    }
    
    pub fn match_input(&self, input: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        if input.is_empty() {
            return suggestions;
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return suggestions;
        }
        
        let base_command = parts[0];
        
        // Exact matches first
        if let Some(subcommands) = self.command_index.get(base_command) {
            for subcommand in subcommands {
                if parts.len() > 1 {
                    let partial = parts[1];
                    if subcommand.starts_with(partial) {
                        let score = self.calculate_score(base_command, subcommand);
                        suggestions.push(Suggestion {
                            text: format!("{} {}", base_command, subcommand),
                            description: format!("{} - {}", base_command, subcommand),
                            suggestion_type: SuggestionType::Command,
                            score,
                        });
                    }
                } else {
                    let score = self.calculate_score(base_command, subcommand);
                    suggestions.push(Suggestion {
                        text: format!("{} {}", base_command, subcommand),
                        description: format!("{} - {}", base_command, subcommand),
                        suggestion_type: SuggestionType::Command,
                        score,
                    });
                }
            }
        }
        
        // Sort by score (highest first)
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        suggestions
    }
    
    pub fn learn_from_history(&mut self, command: &str) {
        let score = self.history_scores.get(command).unwrap_or(&0.0) + 1.0;
        self.history_scores.insert(command.to_string(), score);
    }
    
    pub fn matches(&self, pattern: &str, text: &str) -> bool {
        if pattern.is_empty() {
            return true;
        }
        
        let pattern_lower = pattern.to_lowercase();
        let text_lower = text.to_lowercase();
        
        // Simple fuzzy matching: check if all characters in pattern appear in text in order
        let mut pattern_chars = pattern_lower.chars().peekable();
        let mut text_chars = text_lower.chars();
        
        while let Some(pattern_char) = pattern_chars.peek() {
            let mut found = false;
            while let Some(text_char) = text_chars.next() {
                if *pattern_char == text_char {
                    pattern_chars.next();
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        
        pattern_chars.peek().is_none()
    }
    
    fn calculate_score(&self, base_command: &str, subcommand: &str) -> f32 {
        let full_command = format!("{} {}", base_command, subcommand);
        let history_bonus = self.history_scores.get(&full_command).unwrap_or(&0.0) * 0.1;
        1.0 + history_bonus
    }
}
