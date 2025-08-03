use std::process::{Command, Stdio};
use std::collections::HashMap;

pub async fn execute_command(command: String) -> String {
    // Use a standard thread to execute the command
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    std::thread::spawn(move || {
        let result = execute_command_sync(command);
        let _ = tx.send(result);
    });
    
    rx.await.unwrap_or_else(|_| "Error: Failed to execute command".to_string())
}

fn execute_command_sync(command: String) -> String {
    // Handle built-in commands first
    if let Some(output) = handle_builtin_command_sync(&command) {
        return output;
    }

    // Parse the command
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }

    let cmd = parts[0];
    let args = &parts[1..];

    // Execute the command using std::process::Command
    match Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stderr.is_empty() {
                format!("{}{}", stdout, stderr)
            } else {
                stdout.to_string()
            }
        }
        Err(e) => {
            format!("Error executing command: {}", e)
        }
    }
}

fn handle_builtin_command_sync(command: &str) -> Option<String> {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "clear" => {
            // Clear is handled by the application, not here
            Some(String::new())
        }
        "help" => {
            Some(get_help_text())
        }
        "warp" => {
            if parts.len() > 1 {
                match parts[1] {
                    "version" => Some("Warp Terminal Clone v0.1.0".to_string()),
                    "theme" => {
                        if parts.len() > 2 {
                            Some(format!("Theme changed to: {}", parts[2]))
                        } else {
                            Some("Available themes: dark, light".to_string())
                        }
                    }
                    "settings" => Some("Warp settings panel - feature coming soon!".to_string()),
                    _ => Some("Unknown warp command. Try 'warp help'".to_string()),
                }
            } else {
                Some("Warp Terminal Clone - try 'warp help' for commands".to_string())
            }
        }
        _ => None,
    }
}

#[allow(dead_code)]
async fn handle_builtin_command(command: &str) -> Option<String> {
    handle_builtin_command_sync(command)
}

fn get_help_text() -> String {
    r#"Warp Terminal Clone - Built-in Commands:

Basic Commands:
  help              Show this help message
  clear             Clear the terminal (Ctrl+L)
  
Warp Commands:
  warp version      Show version information  
  warp theme <name> Change theme (dark/light)
  warp settings     Open settings panel

Navigation:
  ↑/↓               Navigate command history
  Tab               Autocomplete (coming soon)
  Enter             Execute command
  
Features:
  - Intelligent command suggestions
  - Syntax highlighting
  - Command history
  - Block-based output
  - Modern UI with Iced
  
For system commands, use standard shell commands (ls, cd, git, etc.)
"#.to_string()
}

#[derive(Debug)]
pub struct ShellSession {
    environment: HashMap<String, String>,
    current_dir: std::path::PathBuf,
    history: Vec<String>,
}

impl Default for ShellSession {
    fn default() -> Self {
        let mut env = HashMap::new();
        
        // Set up basic environment variables
        if let Ok(home) = std::env::var("HOME") {
            env.insert("HOME".to_string(), home);
        }
        if let Ok(user) = std::env::var("USER") {
            env.insert("USER".to_string(), user);
        }
        if let Ok(path) = std::env::var("PATH") {
            env.insert("PATH".to_string(), path);
        }

        Self {
            environment: env,
            current_dir: std::env::current_dir().unwrap_or_else(|_| "/".into()),
            history: Vec::new(),
        }
    }
}

impl ShellSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn execute(&mut self, command: String) -> String {
        // Add to history
        if !command.trim().is_empty() {
            self.history.push(command.clone());
        }

        // Handle directory changes
        if command.trim().starts_with("cd ") {
            return self.handle_cd(&command).await;
        }

        // Handle pwd
        if command.trim() == "pwd" {
            return self.current_dir.display().to_string();
        }

        // Execute other commands
        execute_command(command).await
    }

    async fn handle_cd(&mut self, command: &str) -> String {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        
        let target = if parts.len() > 1 {
            parts[1]
        } else {
            // cd with no arguments goes to home
            return if let Some(home) = self.environment.get("HOME") {
                match std::env::set_current_dir(home) {
                    Ok(_) => {
                        self.current_dir = std::path::PathBuf::from(home);
                        format!("Changed directory to: {}", home)
                    }
                    Err(e) => format!("cd: {}", e),
                }
            } else {
                "cd: HOME not set".to_string()
            };
        };

        // Expand ~ to home directory
        let path = if target.starts_with('~') {
            if let Some(home) = self.environment.get("HOME") {
                target.replacen('~', home, 1)
            } else {
                target.to_string()
            }
        } else {
            target.to_string()
        };

        // Handle relative paths
        let full_path = if std::path::Path::new(&path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            self.current_dir.join(path)
        };

        match std::env::set_current_dir(&full_path) {
            Ok(_) => {
                self.current_dir = full_path.clone();
                format!("Changed directory to: {}", full_path.display())
            }
            Err(e) => format!("cd: {}: {}", full_path.display(), e),
        }
    }

    pub fn get_current_dir(&self) -> &std::path::Path {
        &self.current_dir
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    pub fn get_environment(&self) -> &HashMap<String, String> {
        &self.environment
    }

    pub fn set_env_var(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }
}

// Advanced PTY integration (placeholder for future implementation)
pub struct PtySession {
    // This would integrate with a library like portable-pty or nix
    // For now, we'll use the simpler Command approach
}

impl PtySession {
    pub fn new() -> Self {
        Self {}
    }

    // Future: implement PTY-based shell interaction for better compatibility
    pub async fn spawn_shell(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for PTY shell spawning
        Ok(())
    }
}

// Command completion and suggestion helpers
pub struct CommandCompleter {
    commands: Vec<String>,
    flags: HashMap<String, Vec<String>>,
}

impl Default for CommandCompleter {
    fn default() -> Self {
        let mut completer = Self {
            commands: vec![
                "ls", "cd", "pwd", "cat", "grep", "find", "git", "npm", "cargo",
                "python", "node", "vim", "nano", "clear", "history", "echo",
                "cp", "mv", "rm", "mkdir", "rmdir", "chmod", "chown", "ps",
                "kill", "top", "htop", "df", "du", "free", "uname", "curl",
                "wget", "ssh", "scp", "rsync", "tar", "zip", "unzip",
            ].into_iter().map(|s| s.to_string()).collect(),
            flags: HashMap::new(),
        };

        // Add common flags for popular commands
        completer.flags.insert("ls".to_string(), vec![
            "-l".to_string(), "-a".to_string(), "-h".to_string(), 
            "-t".to_string(), "-r".to_string(), "-1".to_string(),
        ]);

        completer.flags.insert("git".to_string(), vec![
            "status".to_string(), "add".to_string(), "commit".to_string(),
            "push".to_string(), "pull".to_string(), "clone".to_string(),
            "branch".to_string(), "checkout".to_string(), "merge".to_string(),
            "log".to_string(), "diff".to_string(), "stash".to_string(),
        ]);

        completer.flags.insert("cargo".to_string(), vec![
            "build".to_string(), "run".to_string(), "test".to_string(),
            "check".to_string(), "clean".to_string(), "doc".to_string(),
            "publish".to_string(), "install".to_string(), "new".to_string(),
        ]);

        completer
    }
}

impl CommandCompleter {
    pub fn get_completions(&self, input: &str) -> Vec<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Vec::new();
        }

        if parts.len() == 1 {
            // Complete command names
            self.commands.iter()
                .filter(|cmd| cmd.starts_with(parts[0]))
                .cloned()
                .collect()
        } else {
            // Complete flags/subcommands for the given command
            let command = parts[0];
            if let Some(flags) = self.flags.get(command) {
                let current = parts.last().unwrap_or(&"");
                flags.iter()
                    .filter(|flag| flag.starts_with(current))
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        }
    }
}
