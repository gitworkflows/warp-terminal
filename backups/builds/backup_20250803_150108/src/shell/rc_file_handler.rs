use crate::shell::{ShellResult, ShellError};
use std::fs;
use std::path::PathBuf;

pub fn read_rc_file(path: &PathBuf) -> ShellResult<String> {
    fs::read_to_string(path).map_err(|e| ShellError::IoError(e))
}

pub fn write_rc_file(path: &PathBuf, content: &str) -> ShellResult<()> {
    fs::write(path, content).map_err(|e| ShellError::IoError(e))
}

pub fn append_to_rc_file(path: &PathBuf, content: &str) -> ShellResult<()> {
    let mut current_content = read_rc_file(path)?;
    current_content.push_str(content);
    write_rc_file(path, &current_content)
}

pub fn ensure_rc_file_exists(path: &PathBuf) -> ShellResult<()> {
    if !path.exists() {
        write_rc_file(path, "")?;
    }
    Ok(())
}
