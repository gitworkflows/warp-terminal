use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
pub struct CompletionEngine {
    commands: HashMap<String, Vec<String>>,
    file_cache: HashMap<String, Vec<String>>,
    cache_ttl: std::time::Duration,
    last_cache_update: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub text: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompletionKind {
    Command,
    Subcommand,
    Option,
    Flag,
    File,
    Directory,
    Variable,
    Function,
    Keyword,
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            commands: HashMap::new(),
            file_cache: HashMap::new(),
            cache_ttl: std::time::Duration::from_secs(30),
            last_cache_update: std::time::Instant::now(),
        };
        
        engine.load_default_commands();
        engine
    }

    pub fn get_completions(&mut self, partial: &str) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Update cache if needed
        self.update_cache_if_needed();
        
        // Command completions
        completions.extend(self.get_command_completions(partial));
        
        // File path completions
        completions.extend(self.get_file_completions(partial));
        
        // Variable completions
        completions.extend(self.get_variable_completions(partial));
        
        // Sort by score (descending)
        completions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to top 20 results
        completions.truncate(20);
        
        completions
    }

    pub fn get_command_completions(&self, partial: &str) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Check if this looks like a command (first word)
        if !partial.contains(' ') {
            for (command, _subcommands) in &self.commands {
                if command.starts_with(partial) {
                    let score = self.calculate_prefix_score(partial, command);
                    completions.push(CompletionItem {
                        text: command.clone(),
                        kind: CompletionKind::Command,
                        detail: Some("command".to_string()),
                        documentation: self.get_command_description(command),
                        insert_text: Some(command.clone()),
                        score,
                    });
                }
            }
        } else {
            // Check for subcommand completions
            let parts: Vec<&str> = partial.split_whitespace().collect();
            if let Some(base_command) = parts.first() {
                if let Some(subcommands) = self.commands.get(*base_command) {
                    let subcommand_partial = parts.get(1).map_or("", |v| v);
                    
                    for subcommand in subcommands {
                        if subcommand.starts_with(subcommand_partial) {
                            let score = self.calculate_prefix_score(subcommand_partial, subcommand);
                            completions.push(CompletionItem {
                                text: subcommand.clone(),
                                kind: CompletionKind::Subcommand,
                                detail: Some(format!("{} subcommand", base_command)),
                                documentation: self.get_subcommand_description(base_command, subcommand),
                                insert_text: Some(subcommand.clone()),
                                score,
                            });
                        }
                    }
                }
            }
        }
        
        completions
    }

    pub fn get_file_completions(&mut self, partial: &str) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Check if the partial looks like a file path
        if partial.contains('/') || partial.starts_with('~') || partial.starts_with('.') {
            let (dir_path, filename_partial) = self.split_path(partial);
            
            if let Some(entries) = self.get_directory_entries(&dir_path) {
                for entry in entries {
                    if entry.starts_with(&filename_partial) {
                        let is_dir = self.is_directory(&format!("{}/{}", dir_path, entry));
                        let full_path = if dir_path == "." {
                            entry.clone()
                        } else {
                            format!("{}/{}", dir_path, entry)
                        };
                        
                        let score = self.calculate_prefix_score(&filename_partial, &entry);
                        let kind = if is_dir {
                            CompletionKind::Directory
                        } else {
                            CompletionKind::File
                        };
                        
                        let display_text = if is_dir {
                            format!("{}/", entry)
                        } else {
                            entry.clone()
                        };
                        
                        completions.push(CompletionItem {
                            text: display_text,
                            kind,
                            detail: Some(if is_dir { "directory" } else { "file" }.to_string()),
                            documentation: Some(full_path.clone()),
                            insert_text: Some(if is_dir {
                                format!("{}/", entry)
                            } else {
                                entry
                            }),
                            score,
                        });
                    }
                }
            }
        }
        
        completions
    }

    pub fn get_variable_completions(&self, partial: &str) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        if partial.starts_with("$") {
            let var_partial = &partial[1..]; // Remove the '

            
            // Common environment variables
            let common_vars = [
                "HOME", "PATH", "USER", "PWD", "OLDPWD", "SHELL", "TERM",
                "EDITOR", "PAGER", "LANG", "LC_ALL", "TZ", "PS1", "PS2",
            ];
            
            for var in &common_vars {
                if var.to_lowercase().starts_with(&var_partial.to_lowercase()) {
                    let score = self.calculate_prefix_score(var_partial, var);
                    completions.push(CompletionItem {
                        text: format!("${}", var),
                        kind: CompletionKind::Variable,
                        detail: Some("environment variable".to_string()),
                        documentation: std::env::var(var).ok(),
                        insert_text: Some(format!("${}", var)),
                        score,
                    });
                }
            }
            
            // Get actual environment variables
            for (key, value) in std::env::vars() {
                if key.to_lowercase().starts_with(&var_partial.to_lowercase()) 
                    && !common_vars.contains(&key.as_str()) {
                    let score = self.calculate_prefix_score(var_partial, &key) * 0.8; // Lower score for non-common vars
                    completions.push(CompletionItem {
                        text: format!("${}", key),
                        kind: CompletionKind::Variable,
                        detail: Some("environment variable".to_string()),
                        documentation: Some(value),
                        insert_text: Some(format!("${}", key)),
                        score,
                    });
                }
            }
        }
        
        completions
    }

    fn load_default_commands(&mut self) {
        // Load common shell commands with their subcommands
        self.commands.insert("git".to_string(), vec![
            "add".to_string(), "branch".to_string(), "checkout".to_string(),
            "clone".to_string(), "commit".to_string(), "diff".to_string(),
            "fetch".to_string(), "init".to_string(), "log".to_string(),
            "merge".to_string(), "pull".to_string(), "push".to_string(),
            "rebase".to_string(), "reset".to_string(), "status".to_string(),
            "stash".to_string(), "tag".to_string(),
        ]);

        self.commands.insert("cargo".to_string(), vec![
            "build".to_string(), "check".to_string(), "clean".to_string(),
            "doc".to_string(), "new".to_string(), "run".to_string(),
            "test".to_string(), "bench".to_string(), "update".to_string(),
            "search".to_string(), "publish".to_string(), "install".to_string(),
        ]);

        self.commands.insert("npm".to_string(), vec![
            "install".to_string(), "start".to_string(), "test".to_string(),
            "run".to_string(), "build".to_string(), "init".to_string(),
            "publish".to_string(), "update".to_string(), "outdated".to_string(),
            "audit".to_string(), "cache".to_string(),
        ]);

        self.commands.insert("docker".to_string(), vec![
            "build".to_string(), "run".to_string(), "ps".to_string(),
            "images".to_string(), "pull".to_string(), "push".to_string(),
            "stop".to_string(), "start".to_string(), "restart".to_string(),
            "rm".to_string(), "rmi".to_string(), "logs".to_string(),
            "exec".to_string(), "compose".to_string(),
        ]);

        self.commands.insert("kubectl".to_string(), vec![
            "get".to_string(), "describe".to_string(), "create".to_string(),
            "delete".to_string(), "apply".to_string(), "exec".to_string(),
            "logs".to_string(), "port-forward".to_string(), "proxy".to_string(),
            "scale".to_string(), "rollout".to_string(), "config".to_string(),
        ]);

        // Add basic shell commands without subcommands
        let basic_commands = [
            "ls", "cd", "pwd", "mkdir", "rmdir", "rm", "cp", "mv", "cat",
            "less", "more", "head", "tail", "grep", "find", "sort", "uniq",
            "wc", "awk", "sed", "cut", "tr", "xargs", "which", "whereis",
            "locate", "file", "stat", "chmod", "chown", "chgrp", "tar",
            "gzip", "gunzip", "zip", "unzip", "curl", "wget", "ssh", "scp",
            "rsync", "ps", "top", "htop", "kill", "killall", "jobs", "fg",
            "bg", "nohup", "screen", "tmux", "vim", "nvim", "emacs", "nano",
            "code", "open", "echo", "printf", "date", "cal", "uptime", "whoami",
            "id", "groups", "history", "alias", "unalias", "export", "unset",
            "source", "type", "man", "info", "help", "sudo", "su",
        ];

        for cmd in &basic_commands {
            self.commands.insert(cmd.to_string(), Vec::new());
        }
    }

    fn update_cache_if_needed(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_cache_update) > self.cache_ttl {
            self.file_cache.clear();
            self.last_cache_update = now;
        }
    }

    fn get_directory_entries(&mut self, dir_path: &str) -> Option<Vec<String>> {
        // Check cache first
        if let Some(cached) = self.file_cache.get(dir_path) {
            return Some(cached.clone());
        }

        // Expand tilde if present
        let expanded_path = if dir_path.starts_with('~') {
            if let Some(home) = std::env::var("HOME").ok() {
                dir_path.replacen('~', &home, 1)
            } else {
                dir_path.to_string()
            }
        } else {
            dir_path.to_string()
        };

        // Read directory
        if let Ok(entries) = fs::read_dir(&expanded_path) {
            let mut result = Vec::new();
            
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip hidden files unless explicitly requested
                    if !name.starts_with('.') || dir_path.contains("/.") {
                        result.push(name.to_string());
                    }
                }
            }
            
            result.sort();
            self.file_cache.insert(dir_path.to_string(), result.clone());
            Some(result)
        } else {
            None
        }
    }

    fn split_path(&self, path: &str) -> (String, String) {
        if let Some(last_slash) = path.rfind('/') {
            let dir = if last_slash == 0 {
                "/".to_string()
            } else {
                path[..last_slash].to_string()
            };
            let filename = path[last_slash + 1..].to_string();
            (dir, filename)
        } else {
            (".".to_string(), path.to_string())
        }
    }

    fn is_directory(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn calculate_prefix_score(&self, partial: &str, candidate: &str) -> f32 {
        if candidate == partial {
            100.0 // Exact match
        } else if candidate.starts_with(partial) {
            // Prefix match - shorter candidates score higher
            let length_penalty = candidate.len() as f32 - partial.len() as f32;
            90.0 - (length_penalty * 0.1)
        } else if candidate.to_lowercase().starts_with(&partial.to_lowercase()) {
            // Case-insensitive prefix match
            80.0 - (candidate.len() as f32 - partial.len() as f32) * 0.1
        } else if candidate.contains(partial) {
            // Contains match
            60.0
        } else {
            // Fuzzy match (simplified)
            let mut score = 0.0;
            let candidate_lower = candidate.to_lowercase();
            let partial_lower = partial.to_lowercase();
            
            let mut partial_chars = partial_lower.chars().peekable();
            for (i, candidate_char) in candidate_lower.chars().enumerate() {
                if let Some(&partial_char) = partial_chars.peek() {
                    if candidate_char == partial_char {
                        score += 10.0 / (i + 1) as f32; // Earlier matches score higher
                        partial_chars.next();
                    }
                }
            }
            
            // Bonus if all characters matched
            if partial_chars.peek().is_none() {
                score * 2.0
            } else {
                score
            }
        }
    }

    fn get_command_description(&self, command: &str) -> Option<String> {
        match command {
            "git" => Some("Git version control system".to_string()),
            "cargo" => Some("Rust package manager".to_string()),
            "npm" => Some("Node.js package manager".to_string()),
            "docker" => Some("Container management platform".to_string()),
            "kubectl" => Some("Kubernetes command-line tool".to_string()),
            "ls" => Some("List directory contents".to_string()),
            "cd" => Some("Change current directory".to_string()),
            "pwd" => Some("Print working directory".to_string()),
            "cat" => Some("Display file contents".to_string()),
            "grep" => Some("Search text patterns".to_string()),
            "find" => Some("Find files and directories".to_string()),
            _ => None,
        }
    }

    fn get_subcommand_description(&self, command: &str, subcommand: &str) -> Option<String> {
        match (command, subcommand) {
            ("git", "add") => Some("Add files to staging area".to_string()),
            ("git", "commit") => Some("Create a new commit".to_string()),
            ("git", "push") => Some("Upload changes to remote repository".to_string()),
            ("git", "pull") => Some("Download changes from remote repository".to_string()),
            ("git", "status") => Some("Show working tree status".to_string()),
            ("cargo", "build") => Some("Compile the current package".to_string()),
            ("cargo", "run") => Some("Run the current package".to_string()),
            ("cargo", "test") => Some("Run tests".to_string()),
            ("npm", "install") => Some("Install dependencies".to_string()),
            ("npm", "start") => Some("Run start script".to_string()),
            ("docker", "run") => Some("Run a new container".to_string()),
            ("docker", "ps") => Some("List running containers".to_string()),
            _ => None,
        }
    }

    pub fn add_custom_command(&mut self, command: String, subcommands: Vec<String>) {
        self.commands.insert(command, subcommands);
    }

    pub fn remove_command(&mut self, command: &str) {
        self.commands.remove(command);
    }

    pub fn clear_file_cache(&mut self) {
        self.file_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_completions() {
        let engine = CompletionEngine::new();
        let completions = engine.get_command_completions("gi");
        
        assert!(completions.iter().any(|c| c.text == "git"));
        assert!(completions[0].score > 80.0); // Should have high score for prefix match
    }

    #[test]
    fn test_subcommand_completions() {
        let engine = CompletionEngine::new();
        let completions = engine.get_command_completions("git st");
        
        // Should find "status" subcommand
        assert!(completions.iter().any(|c| c.text == "status"));
    }

    #[test]
    fn test_variable_completions() {
        let engine = CompletionEngine::new();
        let completions = engine.get_variable_completions("$HO");
        
        assert!(completions.iter().any(|c| c.text == "$HOME"));
    }

    #[test]
    fn test_prefix_scoring() {
        let engine = CompletionEngine::new();
        
        // Exact match should score highest
        assert!(engine.calculate_prefix_score("test", "test") > 95.0);
        
        // Prefix match should score high
        assert!(engine.calculate_prefix_score("te", "test") > 85.0);
        
        // Contains match should score lower
        assert!(engine.calculate_prefix_score("es", "test") < 70.0);
    }

    #[test]
    fn test_path_splitting() {
        let engine = CompletionEngine::new();
        
        assert_eq!(engine.split_path("dir/file.txt"), ("dir".to_string(), "file.txt".to_string()));
        assert_eq!(engine.split_path("file.txt"), (".".to_string(), "file.txt".to_string()));
        assert_eq!(engine.split_path("/etc/hosts"), ("/etc".to_string(), "hosts".to_string()));
    }
}
