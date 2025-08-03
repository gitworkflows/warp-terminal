pub mod shell_config;
pub mod shell_detector;
pub mod shell_manager;
pub mod platform_shells;
pub mod rc_file_handler;

pub use shell_config::*;
pub use shell_detector::*;
pub use shell_manager::*;
pub use platform_shells::*;
pub use rc_file_handler::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SupportedShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    PowerShell5,
    GitBash,
    Wsl2,
    Nushell, // For detection but not supported
}

impl SupportedShell {
    pub fn is_supported(&self) -> bool {
        match self {
            SupportedShell::Bash
            | SupportedShell::Zsh
            | SupportedShell::Fish
            | SupportedShell::PowerShell
            | SupportedShell::PowerShell5
            | SupportedShell::GitBash
            | SupportedShell::Wsl2 => true,
            SupportedShell::Nushell => false,
        }
    }

    pub fn executable_name(&self) -> &'static str {
        match self {
            SupportedShell::Bash => "bash",
            SupportedShell::Zsh => "zsh",
            SupportedShell::Fish => "fish",
            SupportedShell::PowerShell => "pwsh",
            SupportedShell::PowerShell5 => "powershell",
            SupportedShell::GitBash => "bash",
            SupportedShell::Wsl2 => "wsl",
            SupportedShell::Nushell => "nu",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SupportedShell::Bash => "Bash",
            SupportedShell::Zsh => "Zsh",
            SupportedShell::Fish => "Fish",
            SupportedShell::PowerShell => "PowerShell 7",
            SupportedShell::PowerShell5 => "PowerShell 5",
            SupportedShell::GitBash => "Git Bash",
            SupportedShell::Wsl2 => "WSL2",
            SupportedShell::Nushell => "Nushell (Unsupported)",
        }
    }

    pub fn rc_file_names(&self) -> Vec<&'static str> {
        match self {
            SupportedShell::Bash => vec![".bashrc", ".bash_profile"],
            SupportedShell::Zsh => vec![".zshrc"],
            SupportedShell::Fish => vec!["config.fish"],
            SupportedShell::PowerShell | SupportedShell::PowerShell5 => {
                vec!["Microsoft.PowerShell_profile.ps1"]
            }
            SupportedShell::GitBash => vec![".bashrc", ".bash_profile"],
            SupportedShell::Wsl2 => vec![".bashrc", ".bash_profile"],
            SupportedShell::Nushell => vec!["config.nu"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub shell_type: SupportedShell,
    pub executable_path: std::path::PathBuf,
    pub version: Option<String>,
    pub is_default: bool,
    pub is_available: bool,
    pub rc_files: Vec<std::path::PathBuf>,
}

impl ShellInfo {
    pub fn new(shell_type: SupportedShell, executable_path: std::path::PathBuf) -> Self {
        Self {
            shell_type,
            executable_path,
            version: None,
            is_default: false,
            is_available: true,
            rc_files: Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("Shell not found: {0}")]
    ShellNotFound(String),
    
    #[error("Shell not supported: {0}")]
    ShellNotSupported(String),
    
    #[error("Failed to execute shell command: {0}")]
    ExecutionError(String),
    
    #[error("Failed to parse shell output: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Platform not supported")]
    PlatformNotSupported,
}

pub type ShellResult<T> = Result<T, ShellError>;
