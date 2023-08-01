use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct CommandError {
    pub stdout: String,
    pub stderr: String,
    pub inner: Option<io::Error>,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stdout: {}\nstderr: {}", self.stdout, self.stderr)
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.as_ref().map(|x| x as &(dyn Error + 'static))
    }
}

pub fn run_command(command_name: &str, args: &[&str]) -> Result<(String, String), CommandError> {
    println!("Run command: {} {}", command_name, args.join(" "));

    let mut child = Command::new(command_name);
    for arg in args {
        child.arg(arg);
    }

    let mut child = child
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::inherit())
        .spawn()
        .map_err(|e| CommandError {
            stdout: String::new(),
            stderr: String::new(),
            inner: Some(e),
        })?;

    // Get stdout and stderr
    let stdout = child.stdout.take().ok_or_else(|| CommandError {
        stdout: String::new(),
        stderr: String::new(),
        inner: Some(io::Error::new(io::ErrorKind::BrokenPipe, "No stdout")),
    })?;

    let stderr = child.stderr.take().ok_or_else(|| CommandError {
        stdout: String::new(),
        stderr: String::new(),
        inner: Some(io::Error::new(io::ErrorKind::BrokenPipe, "No stderr")),
    })?;

    // Copy stdout and stderr
    let stdout_data = copy_output(stdout);
    let stderr_data = copy_output(stderr);

    // Wait for the command to finish
    let status = child.wait().map_err(|e| CommandError {
        stdout: stdout_data.clone(),
        stderr: stderr_data.clone(),
        inner: Some(e),
    })?;

    if status.success() {
        Ok((stdout_data, stderr_data))
    } else {
        Err(CommandError {
            stdout: stdout_data,
            stderr: stderr_data,
            inner: None,
        })
    }
}
fn copy_output(mut stream: impl Read) -> String {
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).unwrap();
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_success() {
        let (stdout, stderr) = run_command("echo", &["Hello, World!"]).unwrap();
        assert_eq!(stdout.trim(), "Hello, World!");
        assert_eq!(stderr.trim(), "");
    }

    #[test]
    fn test_run_command_failure() {
        let error = run_command("nonexistent_command", &[]).unwrap_err();
        assert!(error.inner.is_some());

        let error = run_command("ls", &["nonexistent_directory"]).unwrap_err();
        assert!(error.stderr.contains("nonexistent_directory"));
    }
}
