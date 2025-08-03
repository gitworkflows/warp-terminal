use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncReadExt, BufReader};
use serde::{Serialize, Deserialize};
use super::command_corrections::{CommandCorrector, CommandCorrection};

/// Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: std::time::Duration,
    /// Command correction suggestion if available
    pub correction: Option<CommandCorrection>,
    /// Process ID of the executed command, if available
    pub pid: Option<u32>,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            execution_time: std::time::Duration::from_millis(0),
            correction: None,
            pid: None,
        }
    }
}

/// Command executor for running shell commands
#[derive(Debug, Clone)]
pub struct CommandExecutor {
    /// Working directory for commands
    working_dir: std::path::PathBuf,
    /// Environment variables
    env_vars: std::collections::HashMap<String, String>,
    /// Shell to use for execution
    shell: String,
    /// Command corrector for suggesting fixes
    corrector: CommandCorrector,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        let working_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("/"));
        
        let mut env_vars = std::collections::HashMap::new();
        for (key, value) in std::env::vars() {
            env_vars.insert(key, value);
        }

        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/zsh".to_string());

        Self {
            working_dir,
            env_vars,
            shell,
            corrector: CommandCorrector::new(),
        }
    }

    /// Set the working directory
    pub fn set_working_dir(&mut self, dir: std::path::PathBuf) {
        self.working_dir = dir;
    }

    /// Add environment variable
    pub fn set_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Execute a command asynchronously
    pub async fn execute_command(&self, command_text: &str) -> ExecutionResult {
        let start_time = std::time::Instant::now();
        
        if let Some(result) = self.handle_builtin_command(command_text).await {
            return result;
        }

        match self.execute_external_command(command_text).await {
            Ok(mut result) => {
                let correction = self.corrector.suggest_correction(command_text, &result.stderr, result.exit_code);
                result.correction = correction;
                result.execution_time = start_time.elapsed();
                result
            }
            Err(e) => {
                ExecutionResult {
                    stdout: String::new(),
                    stderr: format!("Error executing command: {}", e),
                    exit_code: -1,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                }
            }
        }
    }

    /// Handle built-in terminal commands
    async fn handle_builtin_command(&self, command_text: &str) -> Option<ExecutionResult> {
        let parts: Vec<&str> = command_text.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let start_time = std::time::Instant::now();

        match parts[0] {
            "cd" => {
                let target_dir = if parts.len() > 1 {
                    parts[1]
                } else {
                    "~"
                };

                let new_dir = if target_dir == "~" {
                    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
                } else if target_dir.starts_with("~/") {
                    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
                    home.join(&target_dir[2..])
                } else {
                    std::path::PathBuf::from(target_dir)
                };

                match std::env::set_current_dir(&new_dir) {
                    Ok(_) => Some(ExecutionResult {
                        stdout: format!("Changed directory to: {}", new_dir.display()),
                        stderr: String::new(),
                        exit_code: 0,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    }),
                    Err(e) => Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: format!("cd: {}: {}", target_dir, e),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    }),
                }
            }
            
            "pwd" => {
                let current_dir = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("/"));
                
                Some(ExecutionResult {
                    stdout: format!("{}", current_dir.display()),
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            "echo" => {
                let output = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    String::new()
                };

                Some(ExecutionResult {
                    stdout: output,
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            "clear" => {
                Some(ExecutionResult {
                    stdout: "\x1B[2J\x1B[H".to_string(), // ANSI clear screen
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            "ls" => {
                match std::fs::read_dir(".") {
                    Ok(entries) => {
                        let files = entries.filter_map(Result::ok).map(|e| e.file_name().into_string().unwrap_or_default()).collect::<Vec<_>>().join("\n");
                        Some(ExecutionResult {
                            stdout: files,
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        })
                    }
                    Err(e) => Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: format!("Error listing files: {}", e),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    }),
                }
            }

            "mkdir" => {
                if parts.len() > 1 {
                    match std::fs::create_dir(&parts[1]) {
                        Ok(_) => Some(ExecutionResult {
                            stdout: format!("Directory {} created", parts[1]),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                        Err(e) => Some(ExecutionResult {
                            stdout: String::new(),
                            stderr: format!("mkdir: {}: {}", parts[1], e),
                            exit_code: 1,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                    }
                } else {
                    Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "mkdir: missing operand".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                }
            }

            "help" | "warp-help" => {
                let help_text = r#"Warp Terminal - Built-in Commands:
  cd <dir>        Change directory
  pwd             Print working directory
  ls              List directory contents
  mkdir <dir>     Create a directory
  rmdir <dir>     Remove a directory
  touch <file>    Create a file
  cat <file>      Display file contents
  echo <text>     Echo text
  whoami          Show current user
  date            Show current date/time
  env             Show environment variables
  which <cmd>     Show path to command
  history         Show command history
  clear           Clear screen
  help            Show this help
  exit            Exit terminal

For external commands, just type them normally.
Examples:
  ls -la
  git status
  python script.py
  npm install"#;

                Some(ExecutionResult {
                    stdout: help_text.to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            "rmdir" => {
                if parts.len() > 1 {
                    match std::fs::remove_dir(&parts[1]) {
                        Ok(_) => Some(ExecutionResult {
                            stdout: format!("Directory {} removed", parts[1]),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                        Err(e) => Some(ExecutionResult {
                            stdout: String::new(),
                            stderr: format!("rmdir: {}: {}", parts[1], e),
                            exit_code: 1,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                    }
                } else {
                    Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "rmdir: missing operand".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                }
            }

            "touch" => {
                if parts.len() > 1 {
                    match std::fs::File::create(&parts[1]) {
                        Ok(_) => Some(ExecutionResult {
                            stdout: format!("File {} created", parts[1]),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                        Err(e) => Some(ExecutionResult {
                            stdout: String::new(),
                            stderr: format!("touch: {}: {}", parts[1], e),
                            exit_code: 1,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                    }
                } else {
                    Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "touch: missing operand".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                }
            }

            "cat" => {
                if parts.len() > 1 {
                    match std::fs::read_to_string(&parts[1]) {
                        Ok(content) => Some(ExecutionResult {
                            stdout: content,
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                        Err(e) => Some(ExecutionResult {
                            stdout: String::new(),
                            stderr: format!("cat: {}: {}", parts[1], e),
                            exit_code: 1,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                    }
                } else {
                    Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "cat: missing operand".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                }
            }

            "whoami" => {
                let username = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string());
                Some(ExecutionResult {
                    stdout: username,
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            "date" => {
                let now = std::time::SystemTime::now();
                match now.duration_since(std::time::UNIX_EPOCH) {
                    Ok(duration) => {
                        // Simple date formatting - in a real implementation you'd use chrono
                        Some(ExecutionResult {
                            stdout: format!("Unix timestamp: {}", duration.as_secs()),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        })
                    }
                    Err(_) => Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "date: unable to get current time".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    }),
                }
            }

            "env" => {
                let env_vars = std::env::vars().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("\n");
                Some(ExecutionResult {
                    stdout: env_vars,
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            "history" => {
                // This is a placeholder - in a real implementation, you'd track command history
                Some(ExecutionResult {
                    stdout: "Command history support coming soon!".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            "which" => {
                if parts.len() > 1 {
                    match which::which(&parts[1]) {
                        Ok(path) => Some(ExecutionResult {
                            stdout: path.display().to_string(),
                            stderr: String::new(),
                            exit_code: 0,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                        Err(_) => Some(ExecutionResult {
                            stdout: String::new(),
                            stderr: format!("{}: command not found", parts[1]),
                            exit_code: 1,
                            execution_time: start_time.elapsed(),
                            correction: None,
                            pid: None,
                        }),
                    }
                } else {
                    Some(ExecutionResult {
                        stdout: String::new(),
                        stderr: "which: missing operand".to_string(),
                        exit_code: 1,
                        execution_time: start_time.elapsed(),
                        correction: None,
                        pid: None,
                    })
                }
            }

            "exit" => {
                Some(ExecutionResult {
                    stdout: "Goodbye!".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    execution_time: start_time.elapsed(),
                    correction: None,
                    pid: None,
                })
            }

            _ => None, // Not a built-in command
        }
    }

    /// Execute external command using shell
    async fn execute_external_command(&self, command_text: &str) -> Result<ExecutionResult, std::io::Error> {
        let mut cmd = Command::new(&self.shell);
        cmd.arg("-c")
           .arg(command_text)
           .current_dir(&self.working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .stdin(Stdio::null());

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn()?;
        let pid = child.id();

        // Capture stdout and stderr concurrently
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let stdout_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut output = String::new();
            reader.read_to_string(&mut output).await.unwrap_or(0);
            output
        });

        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut output = String::new();
            reader.read_to_string(&mut output).await.unwrap_or(0);
            output
        });

        // Wait for the process to complete
        let status = child.wait().await?;
        
        // Collect outputs
        let stdout_result = stdout_handle.await.unwrap_or_default();
        let stderr_result = stderr_handle.await.unwrap_or_default();

        Ok(ExecutionResult {
            stdout: stdout_result,
            stderr: stderr_result,
            exit_code: status.code().unwrap_or(-1),
            execution_time: std::time::Duration::from_millis(0), // Will be set by caller
            correction: None,
            pid,
        })
    }

    /// Check if a command is available in PATH
    pub async fn is_command_available(&self, command: &str) -> bool {
        which::which(command).is_ok()
    }

    /// Get command suggestions based on input
    pub async fn get_command_suggestions(&self, input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Built-in commands
        let builtins = ["cd", "pwd", "ls", "mkdir", "rmdir", "touch", "cat", "echo", "whoami", "date", "env", "which", "history", "clear", "help", "exit"];
        for builtin in &builtins {
            if builtin.starts_with(input) {
                suggestions.push(builtin.to_string());
            }
        }

        // Common commands (you could expand this or make it configurable)
        let common_commands = [
            "ls", "cat", "grep", "find", "ps", "top", "kill", "chmod", "chown",
            "git", "npm", "yarn", "python", "node", "cargo", "rustc", "gcc",
            "make", "cmake", "docker", "kubectl", "ssh", "scp", "curl", "wget"
        ];

        for command in &common_commands {
            if command.starts_with(input) && self.is_command_available(command).await {
                suggestions.push(command.to_string());
            }
        }

        suggestions.sort();
        suggestions.truncate(10); // Limit suggestions
        suggestions
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builtin_pwd() {
        let executor = CommandExecutor::new();
        let result = executor.execute_command("pwd").await;
        
        assert_eq!(result.exit_code, 0);
        assert!(!result.stdout.is_empty());
        assert!(result.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_builtin_echo() {
        let executor = CommandExecutor::new();
        let result = executor.execute_command("echo hello world").await;
        
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "hello world");
        assert!(result.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_builtin_help() {
        let executor = CommandExecutor::new();
        let result = executor.execute_command("help").await;
        
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Built-in Commands"));
        assert!(result.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_external_command() {
        let executor = CommandExecutor::new();
        let result = executor.execute_command("echo external").await;
        
        assert_eq!(result.exit_code, 0);
        // Should work either as builtin or external
    }
}
