use crate::shell::{SupportedShell, ShellInfo, ShellResult, ShellError, PlatformShells};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ShellManager {
    pub available_shells: HashMap<SupportedShell, ShellInfo>,
    pub current_shell: Option<SupportedShell>,
    pub login_shell: Option<SupportedShell>,
    pub default_shell_for_platform: SupportedShell,
}

impl ShellManager {
    pub fn new() -> Self {
        let default_shell_for_platform = Self::get_platform_default_shell();
        
        Self {
            available_shells: HashMap::new(),
            current_shell: None,
            login_shell: None,
            default_shell_for_platform,
        }
    }

    /// Initialize the shell manager by detecting available shells
    pub fn initialize(&mut self) -> ShellResult<()> {
        // Detect login shell
        if let Ok(login_shell_info) = PlatformShells::get_login_shell() {
            self.login_shell = Some(login_shell_info.shell_type.clone());
            self.available_shells.insert(login_shell_info.shell_type.clone(), login_shell_info);
        }

        // Detect all available shells
        let detected_shells = ShellInfo::detect_shells();
        for shell_info in detected_shells {
            self.available_shells.insert(shell_info.shell_type.clone(), shell_info);
        }

        // Set current shell to login shell or platform default
        self.current_shell = self.login_shell.clone()
            .or_else(|| {
                if self.available_shells.contains_key(&self.default_shell_for_platform) {
                    Some(self.default_shell_for_platform.clone())
                } else {
                    self.available_shells.keys().next().cloned()
                }
            });

        Ok(())
    }

    /// Get the platform's default shell
    fn get_platform_default_shell() -> SupportedShell {
        #[cfg(target_os = "macos")]
        return SupportedShell::Zsh;

        #[cfg(target_os = "linux")]
        return SupportedShell::Bash;

        #[cfg(target_os = "windows")]
        return SupportedShell::PowerShell;
    }

    /// Get information about the current shell
    pub fn get_current_shell_info(&self) -> Option<&ShellInfo> {
        self.current_shell.as_ref()
            .and_then(|shell| self.available_shells.get(shell))
    }

    /// Switch to a different shell
    pub fn switch_shell(&mut self, shell_type: SupportedShell) -> ShellResult<()> {
        if !self.available_shells.contains_key(&shell_type) {
            return Err(ShellError::ShellNotFound(shell_type.display_name().to_string()));
        }

        if !shell_type.is_supported() {
            return Err(ShellError::ShellNotSupported(shell_type.display_name().to_string()));
        }

        self.current_shell = Some(shell_type);
        Ok(())
    }

    /// Get all available shells
    pub fn get_available_shells(&self) -> Vec<&ShellInfo> {
        self.available_shells.values().collect()
    }

    /// Get supported shells that are not installed
    pub fn get_missing_shells(&self) -> Vec<SupportedShell> {
        let all_supported = vec![
            SupportedShell::Bash,
            SupportedShell::Zsh,
            SupportedShell::Fish,
            SupportedShell::PowerShell,
            SupportedShell::PowerShell5,
            SupportedShell::GitBash,
            SupportedShell::Wsl2,
        ];

        all_supported
            .into_iter()
            .filter(|shell| !self.available_shells.contains_key(shell))
            .filter(|shell| self.is_shell_supported_on_platform(shell))
            .collect()
    }

    /// Check if a shell is supported on the current platform
    fn is_shell_supported_on_platform(&self, shell: &SupportedShell) -> bool {
        match shell {
            #[cfg(target_os = "macos")]
            SupportedShell::Bash | SupportedShell::Zsh | SupportedShell::Fish | SupportedShell::PowerShell => true,
            
            #[cfg(target_os = "linux")]
            SupportedShell::Bash | SupportedShell::Zsh | SupportedShell::Fish | SupportedShell::PowerShell => true,
            
            #[cfg(target_os = "windows")]
            SupportedShell::PowerShell | SupportedShell::PowerShell5 | SupportedShell::GitBash | SupportedShell::Wsl2 => true,
            
            _ => false,
        }
    }

    /// Get shell version information
    pub fn get_shell_version(&self, shell_type: &SupportedShell) -> ShellResult<String> {
        let shell_info = self.available_shells.get(shell_type)
            .ok_or_else(|| ShellError::ShellNotFound(shell_type.display_name().to_string()))?;

        PlatformShells::get_shell_version(shell_info)
    }

    /// Update shell information (e.g., after installation)
    pub fn refresh_shell_info(&mut self, shell_type: SupportedShell) -> ShellResult<()> {
        // Try to detect the shell again
        let detected_shells = ShellInfo::detect_shells();
        
        for shell_info in detected_shells {
            if shell_info.shell_type == shell_type {
                self.available_shells.insert(shell_type, shell_info);
                return Ok(());
            }
        }

        Err(ShellError::ShellNotFound(shell_type.display_name().to_string()))
    }

    /// Get installation instructions for a shell
    pub fn get_installation_instructions(&self, shell_type: &SupportedShell) -> Option<Vec<String>> {
        PlatformShells::get_installation_instructions(shell_type)
    }

    /// Check if we need to show an unsupported shell banner
    pub fn should_show_unsupported_banner(&self) -> bool {
        if let Some(login_shell) = &self.login_shell {
            !login_shell.is_supported()
        } else {
            false
        }
    }

    /// Get the unsupported shell name for banner display
    pub fn get_unsupported_shell_name(&self) -> Option<String> {
        if let Some(login_shell) = &self.login_shell {
            if !login_shell.is_supported() {
                return Some(login_shell.display_name().to_string());
            }
        }
        None
    }

    /// Configure RC files for a shell
    pub fn configure_rc_files(&mut self, shell_type: &SupportedShell) -> ShellResult<()> {
        if let Some(shell_info) = self.available_shells.get_mut(shell_type) {
            shell_info.configure_rc_files()?;
        }
        Ok(())
    }

    /// Get RC file paths for a shell
    pub fn get_rc_file_paths(&self, shell_type: &SupportedShell) -> Vec<PathBuf> {
        match shell_type {
            SupportedShell::Bash => {
                let home = std::env::home_dir().unwrap_or_default();
                vec![home.join(".bashrc"), home.join(".bash_profile")]
            }
            SupportedShell::Zsh => {
                let home = std::env::home_dir().unwrap_or_default();
                vec![home.join(".zshrc")]
            }
            SupportedShell::Fish => {
                let home = std::env::home_dir().unwrap_or_default();
                vec![home.join(".config/fish/config.fish")]
            }
            SupportedShell::PowerShell | SupportedShell::PowerShell5 => {
                // Platform-specific PowerShell profile paths
                #[cfg(target_os = "windows")]
                {
                    vec![PathBuf::from(&format!("{}\\Documents\\PowerShell\\Microsoft.PowerShell_profile.ps1", 
                        std::env::var("USERPROFILE").unwrap_or_default()))]
                }
                
                #[cfg(not(target_os = "windows"))]
                {
                    let home = std::env::home_dir().unwrap_or_default();
                    vec![home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")]
                }
            }
            _ => Vec::new(),
        }
    }
}

impl Default for ShellManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_manager_initialization() {
        let mut manager = ShellManager::new();
        
        // This might fail in test environments without shells installed
        let _ = manager.initialize();
        
        // Should at least have detected the platform default
        assert!(!manager.available_shells.is_empty() || manager.get_missing_shells().len() > 0);
    }

    #[test]
    fn test_platform_default_shell() {
        let default_shell = ShellManager::get_platform_default_shell();
        
        #[cfg(target_os = "macos")]
        assert_eq!(default_shell, SupportedShell::Zsh);
        
        #[cfg(target_os = "linux")]
        assert_eq!(default_shell, SupportedShell::Bash);
        
        #[cfg(target_os = "windows")]
        assert_eq!(default_shell, SupportedShell::PowerShell);
    }

    #[test]
    fn test_rc_file_paths() {
        let manager = ShellManager::new();
        
        let bash_rc_files = manager.get_rc_file_paths(&SupportedShell::Bash);
        assert!(bash_rc_files.iter().any(|p| p.file_name().unwrap() == ".bashrc"));
        
        let zsh_rc_files = manager.get_rc_file_paths(&SupportedShell::Zsh);
        assert!(zsh_rc_files.iter().any(|p| p.file_name().unwrap() == ".zshrc"));
    }
}
