use std::error::Error;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::{fmt, thread};

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
    _run_command(command_name, args, true)
}
pub fn run_command_no_pipe(
    command_name: &str,
    args: &[&str],
) -> Result<(String, String), CommandError> {
    _run_command(command_name, args, false)
}

fn _run_command(
    command_name: &str,
    args: &[&str],
    pipe_output: bool,
) -> Result<(String, String), CommandError> {
    println!("Run command: {} {}", command_name, args.join(" "));

    let mut child = Command::new(command_name)
        .args(args)
        .stdout(if pipe_output {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .stderr(if pipe_output {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .spawn()
        .map_err(|e| CommandError {
            stdout: String::new(),
            stderr: String::new(),
            inner: Some(e),
        })?;

    if pipe_output {
        let stdout_thread = thread::spawn({
            let mut stdout_child = child.stdout.take().unwrap();
            move || {
                let mut buffer = String::new();
                stdout_child.read_to_string(&mut buffer).unwrap();
                buffer
            }
        });

        let stderr_thread = thread::spawn({
            let mut stderr_child = child.stderr.take().unwrap();
            move || {
                let mut buffer = String::new();
                stderr_child.read_to_string(&mut buffer).unwrap();
                buffer
            }
        });

        let stdout = stdout_thread.join().unwrap();
        let stderr = stderr_thread.join().unwrap();

        let status = child.wait().map_err(|e| CommandError {
            stdout: stdout.clone(),
            stderr: stderr.clone(),
            inner: Some(e),
        })?;

        if status.success() {
            Ok((stdout, stderr))
        } else {
            Err(CommandError {
                stdout,
                stderr,
                inner: None,
            })
        }
    } else {
        let status = child.wait().map_err(|e| CommandError {
            stdout: String::new(),
            stderr: String::new(),
            inner: Some(e),
        })?;

        if status.success() {
            Ok((String::new(), String::new()))
        } else {
            Err(CommandError {
                stdout: String::new(),
                stderr: String::new(),
                inner: None,
            })
        }
    }
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
