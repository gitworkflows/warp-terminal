use std::path::PathBuf;
use std::collections::HashMap;
use crate::executor::command_executor::ExecutionResult;

#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    pub command: String,
    pub description: String,
    pub suggestion_type: SuggestionType,
}

#[derive(Debug, Clone)]
pub enum SuggestionType {
    Git,
    Build,
    Package,
    Python,
    Docker,
    System,
}

#[derive(Debug, Clone)]
pub struct ExpandedCommand {
    pub original: String,
    pub expanded: String,
    pub command_type: CommandType,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Git,
    Rust,
    NodeJS,
    Python,
    Docker,
    Kubernetes,
    FileSystem,
    TextProcessing,
    Alias,
    System,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CommandValidation {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub position: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub message: String,
}

impl CommandSuggestion {
    pub fn new(command: &str, description: &str, suggestion_type: SuggestionType) -> Self {
        Self {
            command: command.to_string(),
            description: description.to_string(),
            suggestion_type,
        }
    }
}

/// Advanced command utilities for enhanced terminal functionality
pub struct AdvancedCommands {
    aliases: HashMap<String, String>,
}

impl AdvancedCommands {
    pub fn new() -> Self {
        let mut aliases = HashMap::new();
        
        // Set up common aliases
        aliases.insert("l".to_string(), "ls -la".to_string());
        aliases.insert("ll".to_string(), "ls -l".to_string());
        aliases.insert("la".to_string(), "ls -la".to_string());
        aliases.insert("..".to_string(), "cd ..".to_string());
        aliases.insert("...".to_string(), "cd ../..".to_string());
        aliases.insert("~".to_string(), "cd ~".to_string());
        aliases.insert("h".to_string(), "history".to_string());
        aliases.insert("c".to_string(), "clear".to_string());
        
        Self { aliases }
    }

    /// Expand command aliases
    pub fn expand_alias(&self, command: &str) -> String {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if let Some(first_part) = parts.first() {
            if let Some(alias_expansion) = self.aliases.get(*first_part) {
                if parts.len() > 1 {
                    format!("{} {}", alias_expansion, parts[1..].join(" "))
                } else {
                    alias_expansion.clone()
                }
            } else {
                command.to_string()
            }
        } else {
            command.to_string()
        }
    }

    /// Expand paths with ~ and environment variables
    pub fn expand_path(&self, path: &str) -> PathBuf {
        let expanded = if path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                if path == "~" {
                    home.to_string_lossy().to_string()
                } else {
                    path.replacen('~', &home.to_string_lossy(), 1)
                }
            } else {
                path.to_string()
            }
        } else {
            // Expand environment variables like $HOME, $USER, etc.
            let mut expanded = path.to_string();
            if let Some(dollar_pos) = path.find('$') {
                let var_start = dollar_pos + 1;
                if let Some(var_end) = path[var_start..].find(|c: char| !c.is_alphanumeric() && c != '_') {
                    let var_name = &path[var_start..var_start + var_end];
                    if let Ok(var_value) = std::env::var(var_name) {
                        expanded = path.replace(&format!("${}", var_name), &var_value);
                    }
                } else {
                    // Variable extends to end of string
                    let var_name = &path[var_start..];
                    if let Ok(var_value) = std::env::var(var_name) {
                        expanded = path.replace(&format!("${}", var_name), &var_value);
                    }
                }
            }
            expanded
        };

        PathBuf::from(expanded)
    }

    /// Get command completions for a given input
    pub fn get_completions(&self, input: &str, current_dir: &PathBuf) -> Vec<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Vec::new();
        }

        if parts.len() == 1 {
            // Complete command names
            self.complete_commands(parts[0])
        } else {
            // Complete file paths
            let last_part = parts.last().unwrap_or(&"");
            self.complete_paths(last_part, current_dir)
        }
    }

    fn complete_commands(&self, prefix: &str) -> Vec<String> {
        let mut completions = Vec::new();

        // Add built-in commands
        let builtins = [
            "cd", "pwd", "ls", "mkdir", "rmdir", "touch", "cat", 
            "echo", "whoami", "date", "env", "which", "history", 
            "clear", "help", "exit"
        ];

        for builtin in &builtins {
            if builtin.starts_with(prefix) {
                completions.push(builtin.to_string());
            }
        }

        // Add aliases
        for alias in self.aliases.keys() {
            if alias.starts_with(prefix) {
                completions.push(alias.clone());
            }
        }

        // Add commands from PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for path_dir in path_var.split(':') {
                if let Ok(entries) = std::fs::read_dir(path_dir) {
                    for entry in entries.filter_map(Result::ok) {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            if file_name.starts_with(prefix) {
                                // Check if it's executable
                                if let Ok(metadata) = entry.metadata() {
                                    #[cfg(unix)]
                                    {
                                        use std::os::unix::fs::PermissionsExt;
                                        if metadata.permissions().mode() & 0o111 != 0 {
                                            completions.push(file_name);
                                        }
                                    }
                                    #[cfg(not(unix))]
                                    {
                                        completions.push(file_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        completions.sort();
        completions.dedup();
        completions.truncate(20); // Limit to 20 completions
        completions
    }

    fn complete_paths(&self, prefix: &str, current_dir: &PathBuf) -> Vec<String> {
        let mut completions = Vec::new();
        
        let expanded_prefix = self.expand_path(prefix);
        let (dir_to_search, file_prefix) = if expanded_prefix.is_dir() {
            (expanded_prefix, String::new())
        } else {
            let parent = expanded_prefix.parent().unwrap_or(current_dir);
            let file_name = expanded_prefix.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            (parent.to_path_buf(), file_name)
        };

        if let Ok(entries) = std::fs::read_dir(&dir_to_search) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(&file_prefix) && !file_name.starts_with('.') {
                        let full_path = dir_to_search.join(&file_name);
                        let completion = if full_path.is_dir() {
                            format!("{}/", file_name)
                        } else {
                            file_name
                        };
                        completions.push(completion);
                    }
                }
            }
        }

        completions.sort();
        completions.truncate(15); // Limit to 15 path completions
        completions
    }

    /// Add a new alias
    pub fn add_alias(&mut self, name: String, command: String) {
        self.aliases.insert(name, command);
    }

    /// Remove an alias
    pub fn remove_alias(&mut self, name: &str) -> bool {
        self.aliases.remove(name).is_some()
    }

    /// List all aliases
    pub fn list_aliases(&self) -> Vec<(String, String)> {
        self.aliases.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Get smart command suggestions based on context
    pub fn get_smart_suggestions(&self, input: &str, current_dir: &PathBuf) -> Vec<CommandSuggestion> {
        let mut suggestions = Vec::new();
        
        // Analyze current directory for relevant commands
        if let Ok(entries) = std::fs::read_dir(current_dir) {
            let files: Vec<_> = entries.filter_map(Result::ok).collect();
            
            // Suggest git commands if in a git repository
            if files.iter().any(|entry| entry.file_name() == ".git") {
                suggestions.extend(vec![
                    CommandSuggestion::new("git status", "Check repository status", SuggestionType::Git),
                    CommandSuggestion::new("git add .", "Stage all changes", SuggestionType::Git),
                    CommandSuggestion::new("git commit -m", "Commit staged changes", SuggestionType::Git),
                    CommandSuggestion::new("git push", "Push commits to remote", SuggestionType::Git),
                    CommandSuggestion::new("git pull", "Pull changes from remote", SuggestionType::Git),
                ]);
            }
            
            // Suggest cargo commands if Cargo.toml exists
            if files.iter().any(|entry| entry.file_name() == "Cargo.toml") {
                suggestions.extend(vec![
                    CommandSuggestion::new("cargo build", "Build the project", SuggestionType::Build),
                    CommandSuggestion::new("cargo test", "Run tests", SuggestionType::Build),
                    CommandSuggestion::new("cargo run", "Run the project", SuggestionType::Build),
                    CommandSuggestion::new("cargo check", "Check for errors", SuggestionType::Build),
                ]);
            }
            
            // Suggest npm commands if package.json exists
            if files.iter().any(|entry| entry.file_name() == "package.json") {
                suggestions.extend(vec![
                    CommandSuggestion::new("npm install", "Install dependencies", SuggestionType::Package),
                    CommandSuggestion::new("npm start", "Start the application", SuggestionType::Package),
                    CommandSuggestion::new("npm test", "Run tests", SuggestionType::Package),
                    CommandSuggestion::new("npm run build", "Build the project", SuggestionType::Package),
                ]);
            }
            
            // Suggest Python commands if Python files exist
            if files.iter().any(|entry| {
                entry.file_name().to_string_lossy().ends_with(".py")
            }) {
                suggestions.extend(vec![
                    CommandSuggestion::new("python -m venv venv", "Create virtual environment", SuggestionType::Python),
                    CommandSuggestion::new("pip install -r requirements.txt", "Install requirements", SuggestionType::Python),
                    CommandSuggestion::new("python -m pytest", "Run Python tests", SuggestionType::Python),
                ]);
            }
        }
        
        // Filter suggestions based on input
        if !input.is_empty() {
            suggestions.retain(|s| s.command.starts_with(input));
        }
        
        suggestions.sort_by(|a, b| a.command.cmp(&b.command));
        suggestions.truncate(10);
        
        suggestions
    }
    
    /// Expand command with advanced alias resolution
    pub fn expand_command_advanced(&self, command: &str) -> ExpandedCommand {
        let expanded_alias = self.expand_alias(command);
        let parts: Vec<&str> = expanded_alias.split_whitespace().collect();
        
        if parts.is_empty() {
            return ExpandedCommand {
                original: command.to_string(),
                expanded: command.to_string(),
                command_type: CommandType::Unknown,
                suggestions: Vec::new(),
            };
        }
        
        let cmd = parts[0];
        let command_type = self.classify_command(cmd);
        let suggestions = self.get_command_suggestions(cmd, &parts[1..]);
        
        ExpandedCommand {
            original: command.to_string(),
            expanded: expanded_alias,
            command_type,
            suggestions,
        }
    }
    
    /// Validate command syntax and provide feedback
    pub fn validate_command_syntax(&self, command: &str) -> CommandValidation {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Basic syntax validation
        let mut paren_count = 0;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut string_char = '\0';
        
        for (i, ch) in command.char_indices() {
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
                        errors.push(ValidationError {
                            position: i,
                            message: "Unmatched closing parenthesis".to_string(),
                        });
                    }
                },
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count < 0 {
                        errors.push(ValidationError {
                            position: i,
                            message: "Unmatched closing brace".to_string(),
                        });
                    }
                },
                _ => {}
            }
        }
        
        if paren_count > 0 {
            errors.push(ValidationError {
                position: command.len(),
                message: format!("{} unclosed parenthesis", paren_count),
            });
        }
        
        if brace_count > 0 {
            errors.push(ValidationError {
                position: command.len(),
                message: format!("{} unclosed brace", brace_count),
            });
        }
        
        if in_string {
            errors.push(ValidationError {
                position: command.len(),
                message: "Unterminated string".to_string(),
            });
        }
        
        // Check for potentially dangerous commands
        let dangerous_commands = ["rm -rf", "sudo rm", "mkfs", "dd if=", "> /dev/"];
        for dangerous in &dangerous_commands {
            if command.contains(dangerous) {
                warnings.push(ValidationWarning {
                    message: format!("Potentially dangerous command detected: {}", dangerous),
                });
            }
        }
        
        CommandValidation {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
    
    fn classify_command(&self, cmd: &str) -> CommandType {
        match cmd {
            "git" => CommandType::Git,
            "cargo" => CommandType::Rust,
            "npm" | "yarn" | "pnpm" => CommandType::NodeJS,
            "python" | "python3" | "pip" | "pip3" => CommandType::Python,
            "docker" | "docker-compose" => CommandType::Docker,
            "kubectl" | "helm" => CommandType::Kubernetes,
            "ls" | "cd" | "pwd" | "mkdir" | "rm" | "cp" | "mv" => CommandType::FileSystem,
            "cat" | "less" | "more" | "head" | "tail" | "grep" => CommandType::TextProcessing,
            _ => {
                if self.aliases.contains_key(cmd) {
                    CommandType::Alias
                } else {
                    CommandType::System
                }
            }
        }
    }
    
    fn get_command_suggestions(&self, cmd: &str, args: &[&str]) -> Vec<String> {
        match cmd {
            "git" => {
                if args.is_empty() {
                    vec!["status".to_string(), "add".to_string(), "commit".to_string(), "push".to_string(), "pull".to_string()]
                } else {
                    match args[0] {
                        "commit" => vec!["-m".to_string(), "-am".to_string()],
                        "add" => vec![".".to_string(), "-A".to_string()],
                        _ => Vec::new(),
                    }
                }
            },
            "cargo" => {
                if args.is_empty() {
                    vec!["build".to_string(), "test".to_string(), "run".to_string(), "check".to_string()]
                } else {
                    Vec::new()
                }
            },
            "npm" => {
                if args.is_empty() {
                    vec!["install".to_string(), "start".to_string(), "test".to_string(), "run".to_string()]
                } else {
                    Vec::new()
                }
            },
            _ => Vec::new(),
        }
    }
    
    /// Handle advanced built-in commands
    pub fn handle_advanced_builtin(&mut self, command: &str) -> Option<ExecutionResult> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let start_time = std::time::Instant::now();

        match parts[0] {
            "alias" => {
                if parts.len() == 1 {
                    // List all aliases
                    let aliases_list = self.list_aliases()
                        .into_iter()
                        .map(|(name, cmd)| format!("{}='{}'", name, cmd))
                        .collect::<Vec<_>>()
                        .join("\n");
                    
                    Some(ExecutionResult {
                        stdout: if aliases_list.is_empty() {
                            "No aliases defined".to_string()
                        } else {
                            aliases_list
                        },
                        stderr: String::new(),
                        exit_code: 0,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                } else if parts.len() >= 2 {
                    // Add or show specific alias
                    if let Some(equals_pos) = parts[1].find('=') {
                        let name = parts[1][..equals_pos].to_string();
                        let mut value = parts[1][equals_pos + 1..].to_string();
                        
                        // Remove quotes if present
                        if value.starts_with('"') && value.ends_with('"') {
                            value = value[1..value.len()-1].to_string();
                        } else if value.starts_with('\'') && value.ends_with('\'') {
                            value = value[1..value.len()-1].to_string();
                        }
                        
                        self.add_alias(name.clone(), value);
                        Some(ExecutionResult {
                            stdout: format!("Alias '{}' added", name),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        })
                    } else {
                        // Show specific alias
                        if let Some(value) = self.aliases.get(parts[1]) {
                            Some(ExecutionResult {
                                stdout: format!("{}='{}'", parts[1], value),
                                stderr: String::new(),
                                exit_code: 0,
                                execution_time: start_time.elapsed(),
                                correction: None,
                                pid: None,
                            })
                        } else {
                            Some(ExecutionResult {
                                stdout: String::new(),
                                stderr: format!("alias: {}: not found", parts[1]),
                                exit_code: 1,
                                execution_time: start_time.elapsed(),
                                correction: None,
                                pid: None,
                            })
                        }
                    }
                } else {
                    None
                }
            }

            "unalias" => {
                if parts.len() > 1 {
                    if self.remove_alias(parts[1]) {
                        Some(ExecutionResult {
                            stdout: format!("Alias '{}' removed", parts[1]),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        })
                    } else {
                        Some(ExecutionResult {
                            stdout: String::new(),
                            stderr: format!("unalias: {}: not found", parts[1]),
                            exit_code: 1,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        })
                    }
                } else {
                    Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "unalias: missing operand".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                }
            }

            "type" => {
                if parts.len() > 1 {
                    let cmd = parts[1];
                    
                    // Check if it's an alias
                    if let Some(alias_value) = self.aliases.get(cmd) {
                        Some(ExecutionResult {
                            stdout: format!("{} is aliased to '{}'", cmd, alias_value),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        })
                    } else {
                        // Check if it's a built-in
                        let builtins = [
                            "cd", "pwd", "ls", "mkdir", "rmdir", "touch", "cat", 
                            "echo", "whoami", "date", "env", "which", "history", 
                            "clear", "help", "exit", "alias", "unalias", "type"
                        ];
                        
                        if builtins.contains(&cmd) {
                            Some(ExecutionResult {
                                stdout: format!("{} is a shell builtin", cmd),
                                stderr: String::new(),
                                exit_code: 0,
                                execution_time: start_time.elapsed(),
                                correction: None,
                                pid: None,
                            })
                        } else {
                            // Check if it's in PATH
                            match which::which(cmd) {
                                Ok(path) => Some(ExecutionResult {
                                    stdout: format!("{} is {}", cmd, path.display()),
                                    stderr: String::new(),
                                    exit_code: 0,
                                    execution_time: start_time.elapsed(),
                                    correction: None,
                                    pid: None,
                                }),
                                Err(_) => Some(ExecutionResult {
                                    stdout: String::new(),
                                    stderr: format!("{}: not found", cmd),
                                    exit_code: 1,
                                    execution_time: start_time.elapsed(),
                                    correction: None,
                                    pid: None,
                                }),
                            }
                        }
                    }
                } else {
                    Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "type: missing operand".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                }
            }

            _ => None,
        }
    }
}

impl Default for AdvancedCommands {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_expansion() {
        let advanced = AdvancedCommands::new();
        
        assert_eq!(advanced.expand_alias("l"), "ls -la");
        assert_eq!(advanced.expand_alias("ll"), "ls -l");
        assert_eq!(advanced.expand_alias("l filename"), "ls -la filename");
        assert_eq!(advanced.expand_alias("nonexistent"), "nonexistent");
    }

    #[test]
    fn test_path_expansion() {
        let advanced = AdvancedCommands::new();
        
        // Test ~ expansion
        let home_path = advanced.expand_path("~");
        assert!(home_path.is_absolute());
        
        let subdir_path = advanced.expand_path("~/Documents");
        assert!(subdir_path.starts_with(dirs::home_dir().unwrap()));
    }

    #[test]
    fn test_alias_management() {
        let mut advanced = AdvancedCommands::new();
        
        advanced.add_alias("test".to_string(), "echo test".to_string());
        assert_eq!(advanced.expand_alias("test"), "echo test");
        
        assert!(advanced.remove_alias("test"));
        assert!(!advanced.remove_alias("nonexistent"));
        
        assert_eq!(advanced.expand_alias("test"), "test");
    }
}
