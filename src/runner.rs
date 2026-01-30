use std::process::{Command, Output};
use thiserror::Error;

use crate::output;

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Command failed: {cmd}\n{stderr}")]
    CommandFailed { cmd: String, stderr: String },

    #[error("Failed to execute command: {0}")]
    ExecutionError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RunnerError>;

/// Run a command with arguments
/// If dry_run is true, only prints the command without executing
pub fn run(cmd: &str, args: &[&str], dry_run: bool) -> Result<Output> {
    let full_cmd = format!("{} {}", cmd, args.join(" "));
    output::command(&full_cmd);

    if dry_run {
        // Return a fake successful output for dry run
        return Ok(Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });
    }

    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(RunnerError::CommandFailed {
            cmd: full_cmd,
            stderr,
        });
    }

    Ok(output)
}

/// Run a command and return stdout as a string
pub fn run_capture(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(RunnerError::CommandFailed {
            cmd: format!("{} {}", cmd, args.join(" ")),
            stderr,
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Run a command silently (no output printing)
#[allow(dead_code)]
pub fn run_silent(cmd: &str, args: &[&str]) -> Result<Output> {
    let output = Command::new(cmd).args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(RunnerError::CommandFailed {
            cmd: format!("{} {}", cmd, args.join(" ")),
            stderr,
        });
    }

    Ok(output)
}

/// Run a command with stdin input
#[allow(dead_code)]
pub fn run_with_stdin(cmd: &str, args: &[&str], stdin: &str) -> Result<Output> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut child_stdin) = child.stdin.take() {
        child_stdin.write_all(stdin.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(RunnerError::CommandFailed {
            cmd: format!("{} {}", cmd, args.join(" ")),
            stderr,
        });
    }

    Ok(output)
}
