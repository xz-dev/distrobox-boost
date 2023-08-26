// run external progamm such as &get_container_manager() "docker"

use crate::utils::command_helper::*;
use std::collections::HashSet;

pub fn run_container(
    container_runner: &str,
    name: &str,
    image_name: &str,
    cmd: &str,
    realtime_output: bool,
) -> Result<CommandOutput, CommandError> {
    let mut args = vec![];
    if name.is_empty() {
        args.push("--rm");
    }
    run_container_with_args(
        container_runner,
        name,
        image_name,
        cmd,
        &args,
        realtime_output,
    )
}

pub fn run_container_with_args(
    container_runner: &str,
    name: &str,
    image_name: &str,
    cmd: &str,
    extra_args: &[&str],
    realtime_output: bool,
) -> Result<CommandOutput, CommandError> {
    let mut args = vec!["run", "--user", "root"];
    if !name.is_empty() {
        args.extend_from_slice(&["--name", name]);
    }
    args.extend_from_slice(extra_args);
    if !cmd.is_empty() {
        println!("Using sh -c to run command: {}", cmd);
        args.extend_from_slice(&["--entrypoint", "sh", image_name, "-c", cmd]);
    } else {
        args.push(image_name);
    }

    let output = run_command(container_runner, &args, realtime_output)?;
    Ok(output)
}

pub fn stop_container_with_args(
    container_runner: &str,
    name: &str,
    extra_args: &[&str],
) -> Result<CommandOutput, CommandError> {
    let mut args = vec!["stop", name];
    args.extend_from_slice(extra_args);
    let output = run_command(container_runner, &args, false)?;
    Ok(output)
}

pub fn remove_container(container_runner: &str, name: &str) -> Result<CommandOutput, CommandError> {
    let args = ["rm", name];
    let output = run_command(container_runner, &args, false)?;
    Ok(output)
}

pub fn commit_container(
    container_runner: &str,
    comtainer_name: &str,
    image_name: &str,
    instructions: &[&str],
) -> Result<CommandOutput, CommandError> {
    let mut args = vec!["commit", comtainer_name, image_name];
    for instruction in instructions {
        args.push("-c");
        args.push(instruction);
    }
    // clean CMD, commit will save the CMD from the container
    args.push("-c");
    args.push("CMD []");
    let output = run_command(container_runner, &args, false)?;
    Ok(output)
}

pub fn remove_image(container_runner: &str, name: &str) -> Result<CommandOutput, CommandError> {
    let args = ["rmi", name];
    let output = run_command(container_runner, &args, false)?;
    Ok(output)
}

pub fn find_images(container_runner: &str, filters: &[&str]) -> Result<Vec<String>, CommandError> {
    let mut result = Vec::new();
    for filter in filters {
        let args = vec!["images", "-q", "--filter", filter];
        let output = run_command(container_runner, &args, false)?;
        if output.status.is_some_and(|status| status == 0) {
            let images: HashSet<String> = output
                .stdout
                .split("\n")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            result.push(images);
        }
    }

    let intersection = result
        .into_iter()
        .fold(None, |acc, set| match acc {
            None => Some(set),
            Some(acc) => Some(&acc & &set),
        })
        .unwrap_or_else(HashSet::new);

    Ok(intersection.into_iter().collect())
}

pub fn check_container_exists(
    container_runner: &str,
    container_name: &str,
) -> Result<bool, CommandError> {
    let filter_string = format!("name=^{}$", container_name);
    let args = vec!["ps", "-aq", "-f", &filter_string];
    let output = run_command(container_runner, &args, false)?;
    Ok(!output.stdout.is_empty())
}

pub fn tag_image(
    container_runner: &str,
    name: &str,
    new_name: &str,
) -> Result<CommandOutput, CommandError> {
    let args = ["tag", name, new_name];
    let output = run_command(container_runner, &args, false)?;
    Ok(output)
}

pub fn pin_image(container_runner: &str, image_name: &str) -> Result<String, CommandError> {
    let name = format!("pin-{}", image_name.replace(":", "-").replace("/", "-"));
    run_container_with_args(
        container_runner,
        &name,
        image_name,
        "tail -f /dev/null",
        &["-d", "--restart", "unless-stopped"],
        false,
    )?;
    Ok(name)
}

pub fn unpin_image(container_runner: &str, image_name: &str) -> Result<String, CommandError> {
    let name = format!("pin-{}", image_name.replace(":", "-").replace("/", "-"));
    let _ = stop_container_with_args(container_runner, &name, &["-t", "0"]);
    remove_container(container_runner, &name)?;
    Ok(name)
}

pub fn build_image_from_dockerfile_simple(
    container_runner: &str,
    name: &str,
    dockerfile_path: &str,
    context: &str,
) -> Result<CommandOutput, CommandError> {
    let args = vec!["build", "-t", name, "-f", dockerfile_path, context];
    let output = run_command(container_runner, &args, true)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_container_manager;

    #[test]
    fn test_valid_command() {
        let container_runner = &get_container_manager();
        let name = "test_case_1";
        let image_name = "ubuntu";
        let cmd = "ls";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd, true);
        let _ = remove_container(container_runner, name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_container_with_empty_name() {
        let container_runner = &get_container_manager();
        let image_name = "ubuntu";
        let cmd = "echo 'Hello, World!'";

        let result = run_container(container_runner, "", image_name, cmd, true);
        assert!(
            result.is_ok(),
            "Expected the container to run successfully with an empty name."
        );

        // Check if the command output is as expected
        assert_eq!(
            result.unwrap().stdout.trim_end(),
            "Hello, World!",
            "Unexpected command output"
        );
    }

    #[test]
    fn test_valid_command_with_args() {
        let container_runner = &get_container_manager();
        let name = "test_case_2";
        let image_name = "ubuntu";
        let cmd = "ls -la /";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd, true);
        let _ = remove_container(container_runner, name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_command() {
        let container_runner = &get_container_manager();
        let name = "test_case_3";
        let image_name = "ubuntu";
        let cmd = "";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd, true);
        let _ = remove_container(container_runner, name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_command() {
        let container_runner = &get_container_manager();
        let name = "test_case_4";
        let image_name = "ubuntu";
        let cmd = "non_existent_command";

        let _ = remove_container(container_runner, name);
        let result = run_container(container_runner, name, image_name, cmd, true);
        let _ = remove_container(container_runner, name);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_image() {
        let container_runner = &get_container_manager();
        let name = "test_case_5";
        let image_name = "non_existent_image";
        let cmd = "ls";

        assert!(run_container(container_runner, name, image_name, cmd, true).is_err());
    }

    #[test]
    fn test_remove_container() {
        let container_runner = &get_container_manager();
        let name = "test_case_remove_1";
        let image_name = "ubuntu";
        let cmd = "ls /";

        // First, run a container with the specified name
        let _ = run_container(container_runner, name, image_name, cmd, true);

        // Then, try to remove the container
        assert!(remove_container(container_runner, name).is_ok());
    }

    #[test]
    fn test_remove_non_existent_container() {
        let container_runner = &get_container_manager();
        let name = "test_case_remove_2";

        // Try to remove a container that doesn't exist
        assert!(remove_container(container_runner, name).is_err());
    }

    #[test]
    fn test_commit_container_with_file() {
        let container_runner = &get_container_manager();
        let container_name = "test_commit_with_file";
        let container2_name = "test_run_with_file";
        let image_name = "ubuntu";

        // Create a file in the container using the echo command
        let cmd = "bash -c 'echo \"Hello, World!\" > /testfile.txt'";
        let _ = remove_container(container_runner, container_name);
        run_container(container_runner, container_name, image_name, cmd, true).unwrap();

        // Commit the container to a new image
        let new_image_name = "test_commit_image_with_file";
        let result = commit_container(container_runner, container_name, new_image_name, &vec![]);
        assert!(result.is_ok(), "Commit failed: {:?}", result.err());

        // Run a new container with the new image and check if the file exists
        let cmd_check_file = "bash -c 'cat /testfile.txt'";
        let _ = remove_container(container_runner, container2_name);
        let run_result = run_container(
            container_runner,
            container2_name,
            new_image_name,
            cmd_check_file,
            true,
        );
        assert!(run_result.is_ok(), "File not found: {:?}", run_result.err());

        // Check if the file contains the expected content
        let file_content = run_result.unwrap().stdout;
        assert_eq!(
            file_content.trim_end(),
            "Hello, World!",
            "Unexpected file content"
        );

        // Clean up: remove the temporary container and the new image
        let _ = remove_container(container_runner, container_name);
        let _ = remove_container(container_runner, container2_name);
        let _ = remove_image(container_runner, new_image_name);
    }

    #[test]
    fn test_commit_invalid_container() {
        let container_runner = &get_container_manager();
        let name = "invalid_container";
        let image_name = "new_image_from_invalid_container";

        let result = commit_container(container_runner, name, image_name, &vec![]);

        assert!(
            result.is_err(),
            "Expected an error when committing an invalid container, but got a success."
        );
    }

    #[test]
    fn test_remove_image() {
        let container_runner = &get_container_manager();
        let name = "test_case_remove_image_1";
        let image_name = "ubuntu";
        run_container(container_runner, "", image_name, "ls /tmp", false).unwrap(); // for pull image
        tag_image(container_runner, image_name, name).unwrap();
        assert!(remove_image(container_runner, name).is_ok());
    }

    #[test]
    fn test_remove_non_existent_image() {
        let container_runner = &get_container_manager();
        let name = "test_case_remove_image_non_existent";

        // Try to remove a container that doesn't exist
        assert!(remove_image(container_runner, name).is_err());
    }

    #[test]
    fn test_find_images_single_filter() {
        let container_runner = &get_container_manager();
        let image_name = "ubuntu";
        let container_name = "test_find_images_single_container";
        let _ = remove_container(container_runner, container_name);
        run_container(container_runner, container_name, image_name, "ls", false).unwrap();
        let new_image_name = "test_find_images_single_image";
        let new_image_other_name = "test_find_images_single_image_other";
        let filters = vec!["label=test=find_images_single_filter"];

        let clean = || {
            for image in find_images(container_runner, &filters).unwrap() {
                let _ = remove_image(container_runner, &image);
            }

            for image in find_images(
                container_runner,
                &vec!["label=test=find_images_single_filter_other"],
            )
            .unwrap()
            {
                let _ = remove_image(container_runner, &image);
            }
        };
        clean();

        commit_container(
            container_runner,
            container_name,
            new_image_name,
            &vec!["LABEL test=find_images_single_filter"],
        )
        .unwrap();
        commit_container(
            container_runner,
            container_name,
            new_image_other_name,
            &vec!["LABEL test=find_images_single_filter_other"],
        )
        .unwrap();

        let result = find_images(container_runner, &filters);

        assert!(result.unwrap().len() == 1);
        let _ = remove_container(container_runner, container_name);
        clean();
    }

    #[test]
    fn test_find_images_multiple_filters() {
        let container_runner = &get_container_manager();
        let image_name = "ubuntu";
        let container_name = "test_find_images_multiple_container";
        let _ = remove_container(container_runner, container_name);
        run_container(container_runner, container_name, image_name, "ls", false).unwrap();

        let new_image_name = "test_find_images_multiple_image";
        let new_image_other_name = "test_find_images_multiple_other_image";
        let filters = vec!["label=test=find_images_multiple_filters", "label=test2=0"];

        let clean = || {
            for image in find_images(container_runner, &filters).unwrap() {
                let _ = remove_image(container_runner, &image);
            }
            for image in find_images(
                container_runner,
                &vec!["label=test=find_images_multiple_filters", "label=test2=1"],
            )
            .unwrap()
            {
                let _ = remove_image(container_runner, &image);
            }
        };
        clean();

        commit_container(
            container_runner,
            container_name,
            new_image_name,
            &vec!["LABEL test=find_images_multiple_filters", "LABEL test2=0"],
        )
        .unwrap();
        commit_container(
            container_runner,
            container_name,
            new_image_other_name,
            &vec!["LABEL test=find_images_multiple_filters", "LABEL test2=1"],
        )
        .unwrap();

        let result = find_images(container_runner, &filters);

        println!("{:?}", result);
        assert!(result.unwrap().len() == 1);
        let _ = remove_container(container_runner, container_name);
        clean();
    }

    #[test]
    fn test_find_images_no_match() {
        let container_runner = &get_container_manager();
        let filters = vec!["label=nonexistent"];

        let result = find_images(container_runner, &filters);

        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_find_images_invalid_filter() {
        let container_runner = &get_container_manager();
        let filters = vec!["invalidfilter"];

        let result = find_images(container_runner, &filters);

        assert!(result.is_err());
    }

    #[test]
    fn test_check_container_exists() {
        let container_runner = &get_container_manager();
        let name = "test_check_container_exists";
        let _ = remove_container(container_runner, name);

        // Before run the container
        let result = check_container_exists(container_runner, name);
        assert!(result.is_ok());
        assert_eq!(false, result.unwrap());

        // After run the container
        let image_name = "ubuntu";
        let cmd = "ls";
        let _ = run_container(container_runner, name, image_name, cmd, true);
        let result = check_container_exists(container_runner, name);
        let _ = remove_container(container_runner, name);

        assert!(result.is_ok());
        assert_eq!(true, result.unwrap());
    }

    #[test]
    fn test_pin_image() {
        let container_runner = &get_container_manager();
        let _ = unpin_image(container_runner, "ubuntu:latest");
        // Attempt to pin the ubuntu image
        let result = pin_image(container_runner, "ubuntu:latest");
        assert!(result.is_ok(), "Failed to pin the ubuntu image");

        // Attempt to pin the same image again, expect an error
        let result = pin_image(container_runner, "ubuntu:latest");
        assert!(
            result.is_err(),
            "Should fail when attempting to pin the same image twice"
        );
        let _ = unpin_image(container_runner, "ubuntu:latest");
    }
    #[test]
    fn test_pin_image_nonexistent() {
        let container_runner = &get_container_manager();
        // Attempt to pin an image that does not exist, expect an error
        let result = pin_image(container_runner, "localhost/nonexistent1");
        assert!(
            result.is_err(),
            "Should fail when attempting to pin a nonexistent image"
        );
    }

    #[test]
    fn test_unpin_image() {
        let container_runner = &get_container_manager();
        let _ = unpin_image(container_runner, "ubuntu");
        // Pin the ubuntu image
        let _ = pin_image(container_runner, "ubuntu");
        // Unpin the pinned image
        let unpin_result = unpin_image(container_runner, "ubuntu");
        assert!(unpin_result.is_ok(), "Failed to unpin the ubuntu image");
    }
    #[test]
    fn test_unpin_image_nonexistent() {
        let container_runner = &get_container_manager();
        // Attempt to unpin an image that does not exist, expect an error
        let result = unpin_image(container_runner, "localhost/nonexistent1");
        assert!(
            result.is_err(),
            "Should fail when attempting to unpin a nonexistent image"
        );
    }
    #[test]
    fn test_unpin_image_stopped() {
        let container_runner = &get_container_manager();
        let image_name = "alpine"; // use different image than test_unpin_image
        let _ = unpin_image(container_runner, image_name);
        // Pin the ubuntu image
        let _ = pin_image(container_runner, image_name);

        // Stop the pinned container
        let _ = stop_container_with_args(
            container_runner,
            &format!("pin-{}", image_name),
            &["-t", "0"],
        );

        // Attempt to unpin the stopped container, expect success
        let result = unpin_image(container_runner, image_name);
        assert!(
            result.is_ok(),
            "Failed to unpin the stopped ubuntu container"
        );
    }
}
