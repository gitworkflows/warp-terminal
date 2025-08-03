use crate::shell::{SupportedShell, ShellInfo, ShellResult, ShellError};
use std::path::PathBuf;
use std::process::Command;

pub struct PlatformShells;

impl PlatformShells {
    /// Get platform-specific shell locations
    pub fn get_platform_shell_paths() -> Vec<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            vec![
                PathBuf::from("/bin/bash"),
                PathBuf::from("/bin/zsh"),
                PathBuf::from("/usr/local/bin/fish"),
                PathBuf::from("/opt/homebrew/bin/fish"),
                PathBuf::from("/usr/local/bin/pwsh"),
                PathBuf::from("/opt/homebrew/bin/pwsh"),
            ]
        }

        #[cfg(target_os = "linux")]
        {
            vec![
                PathBuf::from("/bin/bash"),
                PathBuf::from("/usr/bin/bash"),
                PathBuf::from("/bin/zsh"),
                PathBuf::from("/usr/bin/zsh"),
                PathBuf::from("/usr/bin/fish"),
                PathBuf::from("/usr/local/bin/fish"),
                PathBuf::from("/usr/bin/pwsh"),
                PathBuf::from("/usr/local/bin/pwsh"),
            ]
        }

        #[cfg(target_os = "windows")]
        {
            vec![
                PathBuf::from("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
                PathBuf::from("C:\\Program Files\\PowerShell\\7\\pwsh.exe"),
                PathBuf::from("C:\\Program Files (x86)\\Git\\bin\\bash.exe"),
                PathBuf::from("C:\\Windows\\System32\\wsl.exe"),
            ]
        }
    }

    /// Detect login shell
    pub fn get_login_shell() -> ShellResult<ShellInfo> {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            // Get the login shell from the environment
            if let Ok(shell_path) = std::env::var("SHELL") {
                let path = PathBuf::from(&shell_path);
                
                // Determine shell type from path
                let shell_type = Self::identify_shell_from_path(&path)?;
                
                return Ok(ShellInfo::new(shell_type, path));
            }
            
            // Fallback to default shell detection
            ShellInfo::detect_default_shell()
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, default to PowerShell 7 if available, otherwise PowerShell 5
            let pwsh_path = PathBuf::from("C:\\Program Files\\PowerShell\\7\\pwsh.exe");
            if pwsh_path.exists() {
                return Ok(ShellInfo::new(SupportedShell::PowerShell, pwsh_path));
            }
            
            let ps5_path = PathBuf::from("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe");
            if ps5_path.exists() {
                return Ok(ShellInfo::new(SupportedShell::PowerShell5, ps5_path));
            }
            
            Err(ShellError::ShellNotFound("No PowerShell installation found".to_string()))
        }
    }

    /// Identify shell type from executable path
    pub fn identify_shell_from_path(path: &PathBuf) -> ShellResult<SupportedShell> {
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ShellError::ParseError("Invalid shell path".to_string()))?;

        match file_name {
            "bash" => {
                // Check if it's Git Bash on Windows
                #[cfg(target_os = "windows")]
                {
                    if path.to_string_lossy().contains("Git") {
                        return Ok(SupportedShell::GitBash);
                    }
                }
                Ok(SupportedShell::Bash)
            }
            "zsh" => Ok(SupportedShell::Zsh),
            "fish" => Ok(SupportedShell::Fish),
            "pwsh" => Ok(SupportedShell::PowerShell),
            "powershell.exe" => Ok(SupportedShell::PowerShell5),
            "wsl" | "wsl.exe" => Ok(SupportedShell::Wsl2),
            "nu" => Ok(SupportedShell::Nushell),
            _ => Err(ShellError::ShellNotSupported(file_name.to_string())),
        }
    }

    /// Get version information for a shell
    pub fn get_shell_version(shell_info: &ShellInfo) -> ShellResult<String> {
        let version_args = match shell_info.shell_type {
            SupportedShell::Bash => vec!["--version"],
            SupportedShell::Zsh => vec!["--version"],
            SupportedShell::Fish => vec!["--version"],
            SupportedShell::PowerShell => vec!["--version"],
            SupportedShell::PowerShell5 => vec!["-Command", "$PSVersionTable.PSVersion"],
            SupportedShell::GitBash => vec!["--version"],
            SupportedShell::Wsl2 => vec!["--version"],
            SupportedShell::Nushell => vec!["--version"],
        };

        let output = Command::new(&shell_info.executable_path)
            .args(&version_args)
            .output()
            .map_err(|e| ShellError::ExecutionError(e.to_string()))?;

        let version_output = String::from_utf8_lossy(&output.stdout);
        
        // Parse version from output (simplified)
        let version = version_output.lines()
            .next()
            .unwrap_or("unknown")
            .to_string();

        Ok(version)
    }

    /// Check if shell requires special installation steps
    pub fn requires_installation(shell: &SupportedShell) -> bool {
        match shell {
            #[cfg(target_os = "macos")]
            SupportedShell::Fish | SupportedShell::PowerShell => true,
            
            #[cfg(target_os = "linux")]
            SupportedShell::Fish | SupportedShell::PowerShell => true,
            
            #[cfg(target_os = "windows")]
            SupportedShell::Bash | SupportedShell::Fish | SupportedShell::Zsh => true,
            
            _ => false,
        }
    }

    /// Get installation instructions for a shell
    pub fn get_installation_instructions(shell: &SupportedShell) -> Option<Vec<String>> {
        #[cfg(target_os = "macos")]
        {
            match shell {
                SupportedShell::Fish => Some(vec![
                    "Install using Homebrew:".to_string(),
                    "brew install fish".to_string(),
                    "".to_string(),
                    "Or download from:".to_string(),
                    "https://fishshell.com".to_string(),
                ]),
                SupportedShell::PowerShell => Some(vec![
                    "Install using Homebrew:".to_string(),
                    "brew install --cask powershell".to_string(),
                    "".to_string(),
                    "Or download from:".to_string(),
                    "https://github.com/PowerShell/PowerShell/releases".to_string(),
                ]),
                _ => None,
            }
        }

        #[cfg(target_os = "linux")]
        {
            match shell {
                SupportedShell::Fish => Some(vec![
                    "Install using package manager:".to_string(),
                    "Ubuntu/Debian: sudo apt install fish".to_string(),
                    "CentOS/RHEL: sudo yum install fish".to_string(),
                    "Arch: sudo pacman -S fish".to_string(),
                ]),
                SupportedShell::PowerShell => Some(vec![
                    "Install using package manager:".to_string(),
                    "Ubuntu: sudo snap install powershell --classic".to_string(),
                    "Or download from:".to_string(),
                    "https://github.com/PowerShell/PowerShell/releases".to_string(),
                ]),
                _ => None,
            }
        }

        #[cfg(target_os = "windows")]
        {
            match shell {
                SupportedShell::GitBash => Some(vec![
                    "Install Git for Windows:".to_string(),
                    "https://git-scm.com/download/win".to_string(),
                ]),
                SupportedShell::Wsl2 => Some(vec![
                    "Enable WSL2:".to_string(),
                    "wsl --install".to_string(),
                    "".to_string(),
                    "Or follow the guide:".to_string(),
                    "https://docs.microsoft.com/en-us/windows/wsl/install".to_string(),
                ]),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_shell_from_path() {
        let bash_path = PathBuf::from("/bin/bash");
        assert_eq!(
            PlatformShells::identify_shell_from_path(&bash_path).unwrap(),
            SupportedShell::Bash
        );

        let zsh_path = PathBuf::from("/usr/local/bin/zsh");
        assert_eq!(
            PlatformShells::identify_shell_from_path(&zsh_path).unwrap(),
            SupportedShell::Zsh
        );
    }

    #[test]
    fn test_requires_installation() {
        #[cfg(target_os = "macos")]
        {
            assert!(PlatformShells::requires_installation(&SupportedShell::Fish));
            assert!(!PlatformShells::requires_installation(&SupportedShell::Bash));
        }
    }
}
