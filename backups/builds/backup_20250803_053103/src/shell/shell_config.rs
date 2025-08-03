use crate::shell::{ShellInfo, ShellError, ShellResult};

impl ShellInfo {
    pub fn configure_as_default_shell(mut self) -> ShellResult<()> {
        let shell_path_str = self.executable_path.to_str().ok_or(ShellError::ParseError("Invalid shell path".to_string()))?;

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("chsh")
                .arg("-s")
                .arg(shell_path_str)
                .output()
                .map_err(|e| ShellError::ExecutionError(e.to_string()))?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("chsh")
                .arg("-s")
                .arg(shell_path_str)
                .output()
                .map_err(|e| ShellError::ExecutionError(e.to_string()))?;
        }

        #[cfg(target_os = "windows")]
        {
            // Add logic for changing shell on windows
        }

        self.is_default = true;
        Ok(())
    }

    pub fn configure_rc_files(&mut self) -> ShellResult<()> {
        for rc_file in self.shell_type.rc_file_names() {
            let path = std::env::home_dir().ok_or(ShellError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")))?.join(rc_file);
            self.rc_files.push(path);
        }
        Ok(())
    }
}
