// run external progamm such as "podman" "docker"

use std::error::Error;
use std::fmt;
use std::io;
use std::process::Command;

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
        // 泛型错误，没有记录其内部原因。
        None
    }
}

pub fn run_container(
    container_runner: &str,
    name: &str,
    image_name: &str,
    cmd: &str,
) -> Result<(String, String), CommandError> {
    let mut args = vec!["run", "--user", "root"];
    if !name.is_empty() {
        args.extend_from_slice(&["--name", name]);
    } else {
        args.push("--rm");
    }
    if !cmd.is_empty() {
        println!("Using sh -c to run command: {}", cmd);
        args.extend_from_slice(&["--entrypoint", "sh", image_name, "-c", cmd]);
    } else {
        args.push(image_name);
    }

    let (stdout, stderr) = run_command(container_runner, &args)?;
    Ok((stdout, stderr))
}

pub fn remove_container(
    container_runner: &str,
    name: &str,
) -> Result<(String, String), CommandError> {
    let args = ["rm", name];
    let (stdout, stderr) = run_command(container_runner, &args)?;
    Ok((stdout, stderr))
}

pub fn commit_container(
    container_runner: &str,
    name: &str,
    image_name: &str,
) -> Result<(String, String), CommandError> {
    let args = ["commit", name, image_name];
    let (stdout, stderr) = run_command(container_runner, &args)?;
    Ok((stdout, stderr))
}

pub fn check_image_exists(container_runner: &str, image_name: &str) -> Result<bool, CommandError> {
    let args = vec!["images", "-q", image_name];
    let (stdout, _stderr) = run_command(container_runner, &args)?;
    Ok(!stdout.trim().is_empty())
}

pub fn check_container_exists(
    container_runner: &str,
    container_name: &str,
) -> Result<bool, CommandError> {
    let filter_string = format!("name=^{}$", container_name);
    let args = vec!["ps", "-aq", "-f", &filter_string];
    let (stdout, _stderr) = run_command(container_runner, &args)?;
    Ok(!stdout.trim().is_empty())
}

fn run_command(command_name: &str, args: &[&str]) -> Result<(String, String), CommandError> {
    let mut command = Command::new(command_name);

    for arg in args {
        command.arg(arg);
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(e) => {
            return Err(CommandError {
                stdout: String::new(),
                stderr: String::new(),
                inner: Some(e),
            })
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    if output.status.success() {
        println!("Command executed successfully");
        Ok((stdout, stderr))
    } else {
        println!("Command failed");
        Err(CommandError {
            stdout,
            stderr,
            inner: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_command() {
        let container_runner = "podman";
        let name = "test_case_1";
        let image_name = "ubuntu";
        let cmd = "ls";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd);
        let _ = remove_container(container_runner, name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_container_with_empty_name() {
        let container_runner = "podman";
        let image_name = "ubuntu";
        let cmd = "echo 'Hello, World!'";

        let result = run_container(container_runner, "", image_name, cmd);
        assert!(
            result.is_ok(),
            "Expected the container to run successfully with an empty name."
        );

        // Check if the command output is as expected
        let (stdout, _) = result.unwrap();
        assert_eq!(stdout.trim(), "Hello, World!", "Unexpected command output");
    }

    #[test]
    fn test_valid_command_with_args() {
        let container_runner = "podman";
        let name = "test_case_2";
        let image_name = "ubuntu";
        let cmd = "ls -la /";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd);
        let _ = remove_container(container_runner, name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_command() {
        let container_runner = "podman";
        let name = "test_case_3";
        let image_name = "ubuntu";
        let cmd = "";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd);
        let _ = remove_container(container_runner, name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_command() {
        let container_runner = "podman";
        let name = "test_case_4";
        let image_name = "ubuntu";
        let cmd = "non_existent_command";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd);
        let _ = remove_container(container_runner, name);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_image() {
        let container_runner = "podman";
        let name = "test_case_5";
        let image_name = "non_existent_image";
        let cmd = "ls";

        assert!(run_container(container_runner, name, image_name, cmd).is_err());
    }

    #[test]
    fn test_remove_container() {
        let container_runner = "podman";
        let name = "test_case_remove_1";
        let image_name = "ubuntu";
        let cmd = "ls /";

        // First, run a container with the specified name
        let _ = run_container(container_runner, name, image_name, cmd);

        // Then, try to remove the container
        assert!(remove_container(container_runner, name).is_ok());
    }

    #[test]
    fn test_remove_non_existent_container() {
        let container_runner = "podman";
        let name = "test_case_remove_2";

        // Try to remove a container that doesn't exist
        assert!(remove_container(container_runner, name).is_err());
    }

    #[test]
    fn test_commit_container_with_file() {
        let container_runner = "podman"; // Or "docker"
        let container_name = "test_commit_with_file";
        let container2_name = "test_run_with_file";
        let image_name = "ubuntu";

        // Create a file in the container using the echo command
        let cmd = "bash -c 'echo \"Hello, World!\" > /testfile.txt'";
        let _ = remove_container(container_runner, container_name);
        let _ = run_container(container_runner, container_name, image_name, cmd);

        // Commit the container to a new image
        let new_image_name = "test_commit_image_with_file";
        let result = commit_container(container_runner, container_name, new_image_name);
        assert!(result.is_ok(), "Commit failed: {:?}", result.err());

        // Run a new container with the new image and check if the file exists
        let cmd_check_file = "bash -c 'cat /testfile.txt'";
        let _ = remove_container(container_runner, container2_name);
        let run_result = run_container(
            container_runner,
            container2_name,
            new_image_name,
            cmd_check_file,
        );
        assert!(run_result.is_ok(), "File not found: {:?}", run_result.err());

        // Check if the file contains the expected content
        let (file_content, _) = run_result.unwrap();
        assert_eq!(
            file_content.trim(),
            "Hello, World!",
            "Unexpected file content"
        );

        // Clean up: remove the temporary container and the new image
        let _ = remove_container(container_runner, container_name);
        let _ = remove_container(container_runner, container2_name);
        let _ = Command::new(container_runner)
            .arg("rmi")
            .arg(new_image_name)
            .output();
    }

    #[test]
    fn test_commit_invalid_container() {
        let container_runner = "podman";
        let name = "invalid_container";
        let image_name = "new_image_from_invalid_container";

        let result = commit_container(container_runner, name, image_name);

        assert!(
            result.is_err(),
            "Expected an error when committing an invalid container, but got a success."
        );
    }

    #[test]
    fn test_check_container_exists() {
        let container_runner = "podman";
        let name = "test_check_container_exists";
        let _ = remove_container(container_runner, name);

        // Before run the container
        let result = check_container_exists(container_runner, name);
        assert!(result.is_ok());
        assert_eq!(false, result.unwrap());

        // After run the container
        let image_name = "ubuntu";
        let cmd = "ls";
        let _ = run_container(container_runner, name, image_name, cmd);
        let result = check_container_exists(container_runner, name);
        let _ = remove_container(container_runner, name);

        assert!(result.is_ok());
        assert_eq!(true, result.unwrap());
    }

    #[test]
    fn test_check_image_exists() {
        let container_runner = "podman";

        // Non-existent image
        let image_name = "non_existent_image";
        let _ = remove_container(container_runner, image_name);
        let result = check_image_exists(container_runner, image_name);
        assert!(result.is_ok());
        assert_eq!(false, result.unwrap());

        // Existent image
        let image_name = "ubuntu";
        let _ = run_container(container_runner, "", image_name, "");
        let result = check_image_exists(container_runner, image_name);

        assert!(result.is_ok());
        assert_eq!(true, result.unwrap());
    }
}
