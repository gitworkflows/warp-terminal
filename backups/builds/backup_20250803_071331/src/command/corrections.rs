use std::collections::HashMap;
// use crate::executor::advanced_commands::AdvancedCommands;
use crate::command::history::CommandHistory;

#[derive(Debug, Clone)]
pub struct CommandCorrections {
    common_typos: HashMap<String, String>,
    command_history: CommandHistory,
    enabled: bool,
    confidence_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct Correction {
    pub original: String,
    pub suggested: String,
    pub confidence: f32,
    pub correction_type: CorrectionType,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub enum CorrectionType {
    Typo,
    MissingFlag,
    WrongFlag,
    MissingArgument,
    CommandNotFound,
    PathNotFound,
}

impl Default for CommandCorrections {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandCorrections {
    pub fn new() -> Self {
        let mut corrections = Self {
            common_typos: HashMap::new(),
            command_history: CommandHistory::new(),
            enabled: true,
            confidence_threshold: 0.7,
        };
        
        corrections.load_common_typos();
        corrections
    }

    pub fn analyze_command(&self, command: &str) -> Vec<Correction> {
        if !self.enabled {
            return Vec::new();
        }

        let mut corrections = Vec::new();
        
        // Check for typos in command name
        corrections.extend(self.check_command_typos(command));
        
        // Check for missing or wrong flags
        corrections.extend(self.check_flag_issues(command));
        
        // Check for missing arguments
        corrections.extend(self.check_missing_arguments(command));
        
        // Check for path issues
        corrections.extend(self.check_path_issues(command));
        
        // Sort by confidence score
        corrections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // Filter by confidence threshold
        corrections.retain(|c| c.confidence >= self.confidence_threshold);
        
        corrections
    }

    pub fn get_best_correction(&self, command: &str) -> Option<Correction> {
        self.analyze_command(command).into_iter().next()
    }

    pub fn learn_from_correction(&mut self, original: &str, corrected: &str, was_helpful: bool) {
        if was_helpful {
            // Add to common typos if it's a simple substitution
            let parts_orig: Vec<&str> = original.split_whitespace().collect();
            let parts_corr: Vec<&str> = corrected.split_whitespace().collect();
            
            if parts_orig.len() == parts_corr.len() {
                for (orig, corr) in parts_orig.iter().zip(parts_corr.iter()) {
                    if orig != corr && self.levenshtein_distance(orig, corr) <= 2 {
                        self.common_typos.insert(orig.to_string(), corr.to_string());
                    }
                }
            }
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    // Private methods

    fn load_common_typos(&mut self) {
        // Common command typos
        let typos = vec![
            ("ls", "sl"), ("cd", "dc"), ("pwd", "pdw"),
            ("mkdir", "mkdri"), ("rmdir", "rmdir"), ("rm", "mr"),
            ("cp", "pc"), ("mv", "vm"), ("cat", "tac"),
            ("grep", "gerp"), ("find", "fnid"), ("sort", "srot"),
            ("head", "haed"), ("tail", "tial"), ("less", "lses"),
            ("more", "mroe"), ("which", "whihc"), ("whoami", "whaoim"),
            ("history", "histroy"), ("alias", "alais"), ("echo", "ecoh"),
            ("export", "exprot"), ("source", "sourec"), ("chmod", "chmode"),
            ("chown", "chwon"), ("ps", "sp"), ("kill", "kil"),
            ("jobs", "jbos"), ("fg", "gf"), ("bg", "gb"),
            ("git", "gti"), ("cargo", "cagro"), ("npm", "mnp"),
            ("python", "pythno"), ("node", "noed"), ("docker", "dokcer"),
            ("kubectl", "kubetcl"), ("ssh", "shs"), ("scp", "pcs"),
            ("rsync", "rscyn"), ("curl", "culr"), ("wget", "wgte"),
        ];

        for (correct, typo) in typos {
            self.common_typos.insert(typo.to_string(), correct.to_string());
        }
    }

    fn check_command_typos(&self, command: &str) -> Vec<Correction> {
        let mut corrections = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if let Some(first_word) = parts.first() {
            // Check direct typo mapping
            if let Some(correction) = self.common_typos.get(*first_word) {
                let corrected_command = command.replacen(first_word, correction, 1);
                corrections.push(Correction {
                    original: command.to_string(),
                    suggested: corrected_command,
                    confidence: 0.9,
                    correction_type: CorrectionType::Typo,
                    explanation: format!("Did you mean '{}'?", correction),
                });
            }
            
            // Check if command exists in PATH
            if !self.command_exists(first_word) {
                // Find similar commands
                if let Some(similar) = self.find_similar_command(first_word) {
                    let corrected_command = command.replacen(first_word, &similar, 1);
                    corrections.push(Correction {
                        original: command.to_string(),
                        suggested: corrected_command,
                        confidence: self.calculate_similarity_confidence(first_word, &similar),
                        correction_type: CorrectionType::CommandNotFound,
                        explanation: format!("Command '{}' not found. Did you mean '{}'?", first_word, similar),
                    });
                }
            }
        }
        
        corrections
    }

    fn check_flag_issues(&self, command: &str) -> Vec<Correction> {
        let mut corrections = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if let Some(cmd) = parts.first() {
            match *cmd {
                "ls" => {
                    if !command.contains("-l") && !command.contains("-a") {
                        corrections.push(Correction {
                            original: command.to_string(),
                            suggested: format!("{} -la", command),
                            confidence: 0.6,
                            correction_type: CorrectionType::MissingFlag,
                            explanation: "Consider adding -la for detailed listing".to_string(),
                        });
                    }
                },
                "rm" => {
                    if command.contains(" -r") && !command.contains(" -f") {
                        corrections.push(Correction {
                            original: command.to_string(),
                            suggested: command.replace(" -r", " -rf"),
                            confidence: 0.7,
                            correction_type: CorrectionType::MissingFlag,
                            explanation: "Consider adding -f flag for force removal".to_string(),
                        });
                    }
                },
                "git" => {
                    if parts.len() > 1 {
                        match parts[1] {
                            "commit" => {
                                if !command.contains("-m") {
                                    corrections.push(Correction {
                                        original: command.to_string(),
                                        suggested: format!("{} -m \"\"", command),
                                        confidence: 0.8,
                                        correction_type: CorrectionType::MissingFlag,
                                        explanation: "Missing commit message (-m flag)".to_string(),
                                    });
                                }
                            },
                            "push" => {
                                if parts.len() == 2 {
                                    corrections.push(Correction {
                                        original: command.to_string(),
                                        suggested: format!("{} origin main", command),
                                        confidence: 0.6,
                                        correction_type: CorrectionType::MissingArgument,
                                        explanation: "Consider specifying remote and branch".to_string(),
                                    });
                                }
                            },
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }
        
        corrections
    }

    fn check_missing_arguments(&self, command: &str) -> Vec<Correction> {
        let mut corrections = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if let Some(cmd) = parts.first() {
            match *cmd {
                "cd" => {
                    if parts.len() == 1 {
                        corrections.push(Correction {
                            original: command.to_string(),
                            suggested: "cd ~".to_string(),
                            confidence: 0.5,
                            correction_type: CorrectionType::MissingArgument,
                            explanation: "cd without arguments goes to home directory".to_string(),
                        });
                    }
                },
                "grep" => {
                    if parts.len() < 3 {
                        corrections.push(Correction {
                            original: command.to_string(),
                            suggested: format!("{} pattern file", command),
                            confidence: 0.7,
                            correction_type: CorrectionType::MissingArgument,
                            explanation: "grep requires pattern and file arguments".to_string(),
                        });
                    }
                },
                "find" => {
                    if parts.len() < 2 {
                        corrections.push(Correction {
                            original: command.to_string(),
                            suggested: format!("{} . -name \"*\"", command),
                            confidence: 0.6,
                            correction_type: CorrectionType::MissingArgument,
                            explanation: "find requires path and search criteria".to_string(),
                        });
                    }
                },
                _ => {}
            }
        }
        
        corrections
    }

    fn check_path_issues(&self, command: &str) -> Vec<Correction> {
        let mut corrections = Vec::new();
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        // Check for potential path arguments
        for (i, part) in parts.iter().enumerate() {
            if i == 0 { continue; } // Skip command name
            
            // Skip flags
            if part.starts_with('-') { continue; }
            
            // Check if it looks like a path
            if part.contains('/') || part.starts_with('~') || part.starts_with('.') {
                if !std::path::Path::new(part).exists() {
                    // Try to find similar existing paths
                    if let Some(similar_path) = self.find_similar_path(part) {
                        let corrected_command = command.replace(part, &similar_path);
                        corrections.push(Correction {
                            original: command.to_string(),
                            suggested: corrected_command,
                            confidence: 0.8,
                            correction_type: CorrectionType::PathNotFound,
                            explanation: format!("Path '{}' not found. Did you mean '{}'?", part, similar_path),
                        });
                    }
                }
            }
        }
        
        corrections
    }

    fn command_exists(&self, command: &str) -> bool {
        which::which(command).is_ok()
    }

    fn find_similar_command(&self, command: &str) -> Option<String> {
        let common_commands = vec![
            "ls", "cd", "pwd", "mkdir", "rmdir", "rm", "cp", "mv", "cat",
            "grep", "find", "sort", "head", "tail", "less", "more", "which",
            "whoami", "history", "alias", "echo", "export", "source", "chmod",
            "chown", "ps", "kill", "jobs", "fg", "bg", "git", "cargo", "npm",
            "python", "node", "docker", "kubectl", "ssh", "scp", "rsync",
            "curl", "wget",
        ];

        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for cmd in &common_commands {
            let distance = self.levenshtein_distance(command, cmd);
            if distance < best_distance && distance <= 2 {
                best_distance = distance;
                best_match = Some(cmd.to_string());
            }
        }

        best_match
    }

    fn find_similar_path(&self, path: &str) -> Option<String> {
        let parent = std::path::Path::new(path).parent()?;
        let filename = std::path::Path::new(path).file_name()?.to_str()?;

        if let Ok(entries) = std::fs::read_dir(parent) {
            let mut best_match = None;
            let mut best_distance = usize::MAX;

            for entry in entries.filter_map(Result::ok) {
                if let Some(entry_name) = entry.file_name().to_str() {
                    let distance = self.levenshtein_distance(filename, entry_name);
                    if distance < best_distance && distance <= 2 {
                        best_distance = distance;
                        best_match = Some(parent.join(entry_name).to_string_lossy().to_string());
                    }
                }
            }

            return best_match;
        }

        None
    }

    fn calculate_similarity_confidence(&self, original: &str, suggested: &str) -> f32 {
        let distance = self.levenshtein_distance(original, suggested);
        let max_len = original.len().max(suggested.len());
        
        if max_len == 0 {
            return 1.0;
        }
        
        1.0 - (distance as f32 / max_len as f32)
    }

    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                    .min(matrix[i + 1][j] + 1)
                    .min(matrix[i][j] + cost);
            }
        }

        matrix[len1][len2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_typo_detection() {
        let corrections = CommandCorrections::new();
        let result = corrections.analyze_command("gti status");
        
        assert!(!result.is_empty());
        assert_eq!(result[0].suggested, "git status");
        assert!(matches!(result[0].correction_type, CorrectionType::Typo));
    }

    #[test]
    fn test_missing_flag_detection() {
        let corrections = CommandCorrections::new();
        let result = corrections.analyze_command("git commit");
        
        assert!(!result.is_empty());
        assert!(result[0].suggested.contains("-m"));
        assert!(matches!(result[0].correction_type, CorrectionType::MissingFlag));
    }

    #[test]
    fn test_confidence_threshold() {
        let mut corrections = CommandCorrections::new();
        corrections.set_confidence_threshold(0.9);
        
        let result = corrections.analyze_command("ls");
        // Should have fewer suggestions with higher threshold
        assert!(result.iter().all(|c| c.confidence >= 0.9));
    }

    #[test]
    fn test_learning_from_correction() {
        let mut corrections = CommandCorrections::new();
        corrections.learn_from_correction("mycommand", "my_command", true);
        
        let result = corrections.analyze_command("mycommand");
        // Should now suggest the learned correction
        assert!(result.iter().any(|c| c.suggested.contains("my_command")));
    }
}
