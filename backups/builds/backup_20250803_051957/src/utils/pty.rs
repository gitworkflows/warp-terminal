use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;

pub async fn execute_command(command: String) -> String {
    let mut cmd = Command::new("/bin/sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let stdout = cmd.stdout.take().expect("Failed to open stdout");
    let stderr = cmd.stderr.take().expect("Failed to open stderr");

    let mut reader = BufReader::new(stdout).lines();
    let mut reader_err = BufReader::new(stderr).lines();

    let mut output = String::new();

    while let Some(line) = reader.next_line().await.expect("Failed to read line") {
        output.push_str(&line);
        output.push('\n');
    }

    while let Some(line) = reader_err.next_line().await.expect("Failed to read line") {
        output.push_str(&line);
        output.push('\n');
    }

    output
}
