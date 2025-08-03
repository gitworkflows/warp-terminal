use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A command correction suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCorrection {
    /// The original (incorrect) command
    pub original_command: String,
    /// The suggested corrected command
    pub corrected_command: String,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Reason for the correction
    pub reason: CorrectionReason,
    /// Rule that generated this correction
    pub rule: String,
}

/// Types of corrections that can be made
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CorrectionReason {
    /// Command was misspelled
    Misspelling,
    /// Missing required flags or arguments
    MissingFlags,
    /// Permission denied - needs sudo or chmod
    PermissionIssue,
    /// Directory doesn't exist or path issue
    PathIssue,
    /// Command not found but similar command exists
    CommandNotFound,
    /// Wrong argument order or syntax
    ArgumentIssue,
}

/// Command corrections engine
#[derive(Debug, Clone)]
pub struct CommandCorrector {
    /// Correction rules mapped by command name
    rules: HashMap<String, Vec<CorrectionRule>>,
    /// Common command typos
    common_typos: HashMap<String, String>,
    /// Whether the corrector is enabled
    enabled: bool,
}

/// A correction rule for specific commands
#[derive(Debug, Clone)]
pub struct CorrectionRule {
    /// Pattern to match against stderr/error
    pub error_pattern: String,
    /// Function to generate correction
    pub corrector: fn(&str, &str) -> Option<CommandCorrection>,
    /// Confidence level for this rule
    pub confidence: f32,
    /// Name/description of the rule
    pub name: String,
}

impl CommandCorrector {
    /// Create a new command corrector with default rules
    pub fn new() -> Self {
        let mut corrector = Self {
            rules: HashMap::new(),
            common_typos: HashMap::new(),
            enabled: true,
        };
        
        corrector.initialize_default_rules();
        corrector.initialize_common_typos();
        corrector
    }

    /// Enable or disable command corrections
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if corrections are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generate correction suggestions for a failed command
    pub fn suggest_correction(&self, command: &str, stderr: &str, exit_code: i32) -> Option<CommandCorrection> {
        if !self.enabled || exit_code == 0 {
            return None;
        }

        // First check for common typos
        if let Some(correction) = self.check_common_typos(command) {
            return Some(correction);
        }

        // Extract command name
        let command_parts: Vec<&str> = command.trim().split_whitespace().collect();
        if command_parts.is_empty() {
            return None;
        }

        let command_name = command_parts[0];

        // Apply command-specific rules
        if let Some(rules) = self.rules.get(command_name) {
            for rule in rules {
                if stderr.contains(&rule.error_pattern) || rule.error_pattern == "*" {
                    if let Some(correction) = (rule.corrector)(command, stderr) {
                        return Some(correction);
                    }
                }
            }
        }

        // Apply generic rules
        if let Some(generic_rules) = self.rules.get("generic") {
            for rule in generic_rules {
                if stderr.contains(&rule.error_pattern) || rule.error_pattern == "*" {
                    if let Some(correction) = (rule.corrector)(command, stderr) {
                        return Some(correction);
                    }
                }
            }
        }

        None
    }

    /// Check for common command typos
    fn check_common_typos(&self, command: &str) -> Option<CommandCorrection> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command_name = parts[0];
        if let Some(corrected_command_name) = self.common_typos.get(command_name) {
            let corrected_command = command.replacen(command_name, corrected_command_name, 1);
            return Some(CommandCorrection {
                original_command: command.to_string(),
                corrected_command,
                confidence: 0.9,
                reason: CorrectionReason::Misspelling,
                rule: "common_typos".to_string(),
            });
        }

        None
    }

    /// Initialize default correction rules
    fn initialize_default_rules(&mut self) {
        // Git rules
        self.add_git_rules();
        
        // Docker rules
        self.add_docker_rules();
        
        // NPM/Yarn rules
        self.add_npm_rules();
        
        // Generic rules first
        self.add_generic_rules();
        
        // System command rules (adds to both generic and specific rules)
        self.add_system_rules();
    }

    /// Add Git correction rules
    fn add_git_rules(&mut self) {
        let mut git_rules = Vec::new();

        // Git push upstream
        git_rules.push(CorrectionRule {
            error_pattern: "no upstream branch".to_string(),
            corrector: |cmd, _stderr| {
                if cmd.trim() == "git push" {
                    Some(CommandCorrection {
                        original_command: cmd.to_string(),
                        corrected_command: "git push --set-upstream origin HEAD".to_string(),
                        confidence: 0.95,
                        reason: CorrectionReason::MissingFlags,
                        rule: "git_push_upstream".to_string(),
                    })
                } else {
                    None
                }
            },
            confidence: 0.95,
            name: "Git push upstream".to_string(),
        });

        // Git branch not found
        git_rules.push(CorrectionRule {
            error_pattern: "did not match any file(s) known to git".to_string(),
            corrector: |cmd, _stderr| {
                if cmd.starts_with("git checkout ") {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let branch_name = parts[2];
                        Some(CommandCorrection {
                            original_command: cmd.to_string(),
                            corrected_command: format!("git checkout -b {}", branch_name),
                            confidence: 0.8,
                            reason: CorrectionReason::MissingFlags,
                            rule: "git_checkout_create_branch".to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            confidence: 0.8,
            name: "Git checkout create branch".to_string(),
        });

        self.rules.insert("git".to_string(), git_rules);
    }

    /// Add Docker correction rules
    fn add_docker_rules(&mut self) {
        let mut docker_rules = Vec::new();

        // Docker permission denied
        docker_rules.push(CorrectionRule {
            error_pattern: "permission denied".to_string(),
            corrector: |cmd, _stderr| {
                if cmd.starts_with("docker ") && !cmd.starts_with("sudo ") {
                    Some(CommandCorrection {
                        original_command: cmd.to_string(),
                        corrected_command: format!("sudo {}", cmd),
                        confidence: 0.9,
                        reason: CorrectionReason::PermissionIssue,
                        rule: "docker_sudo".to_string(),
                    })
                } else {
                    None
                }
            },
            confidence: 0.9,
            name: "Docker sudo".to_string(),
        });

        self.rules.insert("docker".to_string(), docker_rules);
    }

    /// Add NPM/Yarn correction rules
    fn add_npm_rules(&mut self) {
        let mut npm_rules = Vec::new();

        // NPM command not found -> suggest npx
        npm_rules.push(CorrectionRule {
            error_pattern: "command not found".to_string(),
            corrector: |cmd, _stderr| {
                if cmd.starts_with("npm ") {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 2 && !["install", "start", "test", "run", "init"].contains(&parts[1]) {
                        Some(CommandCorrection {
                            original_command: cmd.to_string(),
                            corrected_command: cmd.replace("npm ", "npx "),
                            confidence: 0.8,
                            reason: CorrectionReason::CommandNotFound,
                            rule: "npm_to_npx".to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            confidence: 0.8,
            name: "NPM to NPX".to_string(),
        });

        self.rules.insert("npm".to_string(), npm_rules);
    }

    /// Add system command correction rules
    fn add_system_rules(&mut self) {
        let mut system_rules = Vec::new();

        // Permission denied -> add chmod +x
        system_rules.push(CorrectionRule {
            error_pattern: "Permission denied".to_string(),
            corrector: |cmd, _stderr| {
                if cmd.starts_with("./") {
                    Some(CommandCorrection {
                        original_command: cmd.to_string(),
                        corrected_command: format!("chmod +x {} && {}", cmd, cmd),
                        confidence: 0.9,
                        reason: CorrectionReason::PermissionIssue,
                        rule: "chmod_exec".to_string(),
                    })
                } else {
                    None
                }
            },
            confidence: 0.9,
            name: "Chmod executable".to_string(),
        });

        // Directory not found in cd command
        system_rules.push(CorrectionRule {
            error_pattern: "No such file or directory".to_string(),
            corrector: |cmd, _stderr| {
                if cmd.starts_with("cd ") {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let target_dir = parts[1];
                        // Suggest mkdir first
                        Some(CommandCorrection {
                            original_command: cmd.to_string(),
                            corrected_command: format!("mkdir -p {} && cd {}", target_dir, target_dir),
                            confidence: 0.7,
                            reason: CorrectionReason::PathIssue,
                            rule: "mkdir_cd".to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            confidence: 0.7,
            name: "Mkdir then CD".to_string(),
        });

        // Add system rules to generic rules since they apply to various commands
        if let Some(generic_rules) = self.rules.get_mut("generic") {
            generic_rules.extend(system_rules.clone());
        }
        
        // Also add them specifically for cd command
        self.rules.insert("cd".to_string(), system_rules);
    }

    /// Add generic correction rules that apply to any command
    fn add_generic_rules(&mut self) {
        let mut generic_rules = Vec::new();

        // Command not found
        generic_rules.push(CorrectionRule {
            error_pattern: "command not found".to_string(),
            corrector: |cmd, _stderr| {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if let Some(command_name) = parts.first() {
                    // Try to find similar commands
                    if let Some(suggestion) = find_similar_command(command_name) {
                        return Some(CommandCorrection {
                            original_command: cmd.to_string(),
                            corrected_command: cmd.replacen(command_name, &suggestion, 1),
                            confidence: 0.7,
                            reason: CorrectionReason::CommandNotFound,
                            rule: "similar_command".to_string(),
                        });
                    }
                }
                None
            },
            confidence: 0.7,
            name: "Similar command".to_string(),
        });

        self.rules.insert("generic".to_string(), generic_rules);
    }

    /// Initialize common command typos
    fn initialize_common_typos(&mut self) {
        let typos = [
            ("gti", "git"),
            ("gut", "git"),
            ("got", "git"),
            ("igt", "git"),
            ("ger", "git"),
            ("car", "cat"),
            ("cta", "cat"),
            ("sl", "ls"),
            ("l", "ls"),
            ("ll", "ls -la"),
            ("cd..", "cd .."),
            ("cd.", "cd ."),
            ("cdd", "cd"),
            ("claer", "clear"),
            ("clera", "clear"),
            ("clearn", "clear"),
            ("ecoh", "echo"),
            ("ehco", "echo"),
            ("mkdri", "mkdir"),
            ("mdir", "mkdir"),
            ("rimdir", "rmdir"),
            ("rmder", "rmdir"),
            ("pytohn", "python"),
            ("pyhton", "python"),
            ("pythno", "python"),
            ("ndoe", "node"),
            ("noed", "node"),
            ("nmp", "npm"),
            ("yran", "yarn"),
            ("cagro", "cargo"),
            ("crago", "cargo"),
            ("gacro", "cargo"),
            ("doker", "docker"),
            ("dcoker", "docker"),
            ("docekr", "docker"),
        ];

        for (typo, correction) in &typos {
            self.common_typos.insert(typo.to_string(), correction.to_string());
        }
    }
}

/// Find similar command names (simple implementation)
fn find_similar_command(command: &str) -> Option<String> {
    let common_commands = [
        "ls", "cat", "grep", "find", "ps", "top", "kill", "chmod", "chown",
        "git", "npm", "yarn", "python", "node", "cargo", "rustc", "gcc",
        "make", "cmake", "docker", "kubectl", "ssh", "scp", "curl", "wget",
        "cd", "pwd", "mkdir", "rmdir", "touch", "rm", "cp", "mv", "ln",
        "tar", "zip", "unzip", "gzip", "gunzip", "head", "tail", "sort",
        "uniq", "cut", "awk", "sed", "tr", "wc", "diff", "patch", "nano",
        "vim", "emacs", "less", "more", "which", "whereis", "locate",
    ];

    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for &candidate in &common_commands {
        let distance = levenshtein_distance(command, candidate);
        // Allow up to 2 character differences or half the command length, whichever is larger
        let max_distance = std::cmp::max(2, command.len() / 2);
        if distance < best_distance && distance <= max_distance && distance > 0 {
            best_distance = distance;
            best_match = Some(candidate.to_string());
        }
    }

    best_match
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

impl Default for CommandCorrector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_typos() {
        let corrector = CommandCorrector::new();
        
        let correction = corrector.suggest_correction("gti status", "", 127);
        assert!(correction.is_some());
        let correction = correction.unwrap();
        assert_eq!(correction.corrected_command, "git status");
        assert_eq!(correction.reason, CorrectionReason::Misspelling);
    }

    #[test]
    fn test_git_push_upstream() {
        let corrector = CommandCorrector::new();
        
        let correction = corrector.suggest_correction(
            "git push", 
            "fatal: The current branch has no upstream branch", 
            1
        );
        assert!(correction.is_some());
        let correction = correction.unwrap();
        assert_eq!(correction.corrected_command, "git push --set-upstream origin HEAD");
        assert_eq!(correction.reason, CorrectionReason::MissingFlags);
    }

    #[test]
    fn test_chmod_permission() {
        let corrector = CommandCorrector::new();
        
        let correction = corrector.suggest_correction(
            "./script.sh", 
            "Permission denied", 
            126
        );
        assert!(correction.is_some());
        let correction = correction.unwrap();
        assert_eq!(correction.corrected_command, "chmod +x ./script.sh && ./script.sh");
        assert_eq!(correction.reason, CorrectionReason::PermissionIssue);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("git", "gti"), 2);
        assert_eq!(levenshtein_distance("cat", "car"), 1);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "test"), 4);
    }

    #[test]
    fn test_similar_command() {
        assert_eq!(find_similar_command("gti"), Some("git".to_string()));
        assert_eq!(find_similar_command("cta"), Some("cat".to_string()));
        assert_eq!(find_similar_command("xyz123"), None);
    }

    #[test]
    fn test_disabled_corrector() {
        let mut corrector = CommandCorrector::new();
        corrector.set_enabled(false);
        
        let correction = corrector.suggest_correction("gti status", "", 127);
        assert!(correction.is_none());
    }
}
