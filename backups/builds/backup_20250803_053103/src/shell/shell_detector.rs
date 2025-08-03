use crate::shell::{SupportedShell, ShellInfo, ShellResult, ShellError};
use std::path::PathBuf;
use std::process::Command;

impl ShellInfo {
    pub fn detect_default_shell() -> ShellResult<Self> {
        // Determine the operating system
        #[cfg(target_os = "macos")]
        let default_shell = SupportedShell::Zsh;

        #[cfg(target_os = "linux")]
        let default_shell = SupportedShell::Bash;

        #[cfg(target_os = "windows")]
        let default_shell = SupportedShell::PowerShell;

        let shell_path = Command::new("which")
            .arg(default_shell.executable_name())
            .output()
            .map_err(|err| ShellError::ExecutionError(err.to_string()))?
            .stdout;

        let shell_path = String::from_utf8_lossy(&shell_path).trim().to_string();

        if shell_path.is_empty() {
            return Err(ShellError::ShellNotFound(default_shell.display_name().to_string()));
        }

        Ok(ShellInfo::new(default_shell, PathBuf::from(shell_path)))
    }

    pub fn detect_shells() -> Vec<Self> {
        let mut shells = Vec::new();

        for shell in [SupportedShell::Bash, SupportedShell::Zsh, SupportedShell::Fish, SupportedShell::PowerShell, SupportedShell::PowerShell5, SupportedShell::GitBash, SupportedShell::Wsl2] {
            let executable_name = shell.executable_name();

            if let Ok(path) = Command::new("which").arg(executable_name).output() {
                let path_str = String::from_utf8_lossy(&path.stdout).trim().to_string();

                if !path_str.is_empty() {
                    let shell_info = ShellInfo {
                        shell_type: shell,
                        executable_path: PathBuf::from(path_str),
                        version: None,
                        is_default: false,
                        is_available: true,
                        rc_files: Vec::new(),
                    };

                    shells.push(shell_info);
                }
            }
        }

        shells
    }
}
