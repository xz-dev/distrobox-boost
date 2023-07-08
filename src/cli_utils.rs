// run external progamm such as "podman" "docker"

use std::io;
use std::process::Command;

fn run_container(
    container_runner: &str,
    name: &str,
    image_name: &str,
    cmd: &str,
) -> io::Result<String> {
    let mut cmd_parts = cmd.split_whitespace();
    let cmd_name = cmd_parts.next();
    let cmd_args: Vec<&str> = cmd_parts.collect();

    let mut command = Command::new(container_runner);
    command.arg("run").arg("--name").arg(name);

    if let Some(entrypoint) = cmd_name {
        println!("entrypoint: {}", entrypoint);
        command.arg("--entrypoint").arg(entrypoint);
    }

    command.arg(image_name); // image_name should between entrypoint and its arguments

    println!("cmd_args: {:?}", cmd_args);
    for arg in cmd_args {
        command.arg(arg);
    }

    let output = command.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    if output.status.success() {
        println!("Command executed successfully");
        Ok(stdout)
    } else {
        println!("Command failed");
        Err(io::Error::new(io::ErrorKind::Other, stderr))
    }
}

fn remove_container(container_runner: &str, name: &str) -> io::Result<()> {
    let mut command = Command::new(container_runner);
    command.arg("rm").arg(name);

    let output = command.output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ))
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
}
