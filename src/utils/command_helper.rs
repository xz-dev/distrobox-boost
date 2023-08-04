use std::error::Error;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::{fmt, thread};

#[derive(Debug)]
pub struct CommandError {
    pub stdout: String,
    pub stderr: String,
    pub status: Option<i32>,
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

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: Option<i32>,
}

pub fn run_command(
    command_name: &str,
    args: &[&str],
    realtime_output: bool,
) -> Result<CommandOutput, CommandError> {
    _run_command(command_name, args, false, realtime_output)
}
pub fn run_command_no_pipe(
    command_name: &str,
    args: &[&str],
) -> Result<CommandOutput, CommandError> {
    _run_command(command_name, args, true, false)
}

fn _run_command(
    command_name: &str,
    args: &[&str],
    inherit: bool,
    realtime_output: bool,
) -> Result<CommandOutput, CommandError> {
    println!("Run command: {} {}", command_name, args.join(" "));
    let mut collected_stdout = String::new();
    let mut collected_stderr = String::new();

    let mut child = Command::new(command_name)
        .args(args)
        .stdout(if inherit {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .stderr(if inherit {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .spawn()
        .map_err(|e| CommandError {
            stdout: collected_stdout.clone(),
            stderr: collected_stderr.clone(),
            status: None,
            inner: Some(e),
        })?;

    if !inherit {
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let stdout_handle = thread::spawn(move || {
            let mut collected_stdout = String::new();
            for line in stdout_reader.lines() {
                let l = line.unwrap();
                if realtime_output {
                    println!("{}", l);
                }
                collected_stdout.push_str(&l);
                collected_stdout.push('\n');
            }
            collected_stdout
        });

        let stderr_handle = thread::spawn(move || {
            let mut collected_stderr = String::new();
            for line in stderr_reader.lines() {
                let l = line.unwrap();
                if realtime_output {
                    eprintln!("{}", l);
                }
                collected_stderr.push_str(&l);
                collected_stderr.push('\n');
            }
            collected_stderr
        });

        collected_stdout = stdout_handle.join().unwrap();
        collected_stderr = stderr_handle.join().unwrap();
    }
    let status = child.wait().map_err(|e| CommandError {
        stdout: collected_stdout.clone(),
        stderr: collected_stderr.clone(),
        status: None,
        inner: Some(e),
    })?;

    if status.success() {
        Ok(CommandOutput {
            stdout: collected_stdout,
            stderr: collected_stderr,
            status: status.code(),
        })
    } else {
        Err(CommandError {
            stdout: collected_stdout,
            stderr: collected_stderr,
            status: status.code(),
            inner: None,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_success() {
        let output = run_command("echo", &["Hello, World!"], false).unwrap();
        assert_eq!(output.stdout.trim_end(), "Hello, World!");
        assert_eq!(output.stderr.trim_end(), "");
        assert_eq!(output.status, Some(0));
    }

    #[test]
    fn test_run_command_failure() {
        let error = run_command("nonexistent_command", &[], false).unwrap_err();
        assert!(error.inner.is_some());

        let error = run_command("ls", &["nonexistent_directory"], false).unwrap_err();
        assert!(error.stderr.contains("nonexistent_directory"));
        assert_ne!(error.status, Some(0));
    }

    #[test]
    fn test_run_command_realtime_output_success() {
        let output = run_command("echo", &["Hello, World!"], true).unwrap();
        assert_eq!(output.stdout.trim_end(), "Hello, World!");
        assert_eq!(output.stderr.trim_end(), "");
        assert_eq!(output.status, Some(0));
    }

    #[test]
    fn test_run_command_realtime_output_failure() {
        let error = run_command("nonexistent_command", &[], true).unwrap_err();
        assert!(error.inner.is_some());

        let error = run_command("ls", &["nonexistent_directory"], true).unwrap_err();
        assert!(error.stderr.contains("nonexistent_directory"));
        assert_ne!(error.status, Some(0));
    }

    #[test]
    fn test_run_command_no_pipe_success() {
        let output = run_command_no_pipe("sh", &["-c", "exit 0"]).unwrap();
        assert_eq!(output.stdout.trim_end(), "");
        assert_eq!(output.stderr.trim_end(), "");
        assert_eq!(output.status, Some(0));
    }

    #[test]
    fn test_run_command_no_pipe_failure() {
        let error = run_command_no_pipe("nonexistent_command", &[]).unwrap_err();
        assert!(error.inner.is_some());

        let error = run_command_no_pipe("sh", &["-c", "exit 1"]).unwrap_err();
        assert_eq!(error.stdout.trim_end(), "");
        assert_eq!(error.stderr.trim_end(), "");
        assert_ne!(error.status, Some(0));
    }
}
