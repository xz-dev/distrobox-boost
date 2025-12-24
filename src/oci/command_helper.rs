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
    image_name: &str,
    dockerfile_path: &str,
    context: &str,
) -> Result<CommandOutput, CommandError> {
    let args = vec!["build", "-t", image_name, "-f", dockerfile_path, context];
    let output = run_command(container_runner, &args, true)?;
    Ok(output)
}

pub fn inspect_image(
    container_runner: &str,
    image_name: &str,
    format: &str,
) -> Result<String, CommandError> {
    let format_arg = format!("--format={{{{{}}}}}", format);
    let args = vec!["inspect", &format_arg, image_name];
    let output = run_command(container_runner, &args, false)?;
    Ok(if output.stdout.trim() == "<no value>" {
        "".to_string()
    } else {
        output.stdout
    })
}

pub fn list_images_by_prefix(
    container_runner: &str,
    prefix: &str,
) -> Result<Vec<String>, CommandError> {
    let filter = format!("reference={}/*", prefix);
    let args = vec![
        "images",
        "--filter",
        &filter,
        "--format",
        "{{.Repository}}:{{.Tag}}",
    ];
    let output = run_command(container_runner, &args, false)?;

    let images: Vec<String> = output
        .stdout
        .lines()
        .filter(|s| !s.is_empty() && !s.contains("<none>"))
        .map(|s| s.to_string())
        .collect();

    Ok(images)
}

pub fn export_images(
    container_runner: &str,
    images: &[String],
    output_path: &str,
) -> Result<(), CommandError> {
    if images.is_empty() {
        return Err(CommandError {
            stdout: String::new(),
            stderr: "No images to export".to_string(),
            status: Some(1),
            inner: None,
        });
    }

    // Build save command: podman uses -m for multi-image, docker doesn't need it
    let mut args: Vec<&str> = vec!["save"];
    if container_runner.contains("podman") {
        args.push("-m");
    }
    args.push("-o");
    args.push(output_path);
    for image in images {
        args.push(image);
    }

    run_command(container_runner, &args, true)?;
    Ok(())
}

pub fn import_images(
    container_runner: &str,
    input_path: &str,
) -> Result<String, CommandError> {
    if !std::path::Path::new(input_path).exists() {
        return Err(CommandError {
            stdout: String::new(),
            stderr: format!("File not found: {}", input_path),
            status: Some(1),
            inner: None,
        });
    }

    let args = vec!["load", "-i", input_path];
    let output = run_command(container_runner, &args, true)?;
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_container_manager;
    use std::env;

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
        run_container(container_runner, "", image_name, "ls", false).unwrap(); // for pull image
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

    #[test]
    fn test_build_image_from_dockerfile_simple() {
        let container_runner = &get_container_manager();
        let name = "test_build_image_from_dockerfile_simple";
        let mut path = env::current_dir().unwrap();
        path.push("tests/files/example_dockerfile_build_image");
        let dockerfile_path = path.to_str().unwrap().to_owned();
        let context = ".";
        let _ = remove_image(container_runner, name);
        let result =
            build_image_from_dockerfile_simple(container_runner, name, &dockerfile_path, context);
        assert!(result.is_ok());
        let cmd = "fish -c 'ls /build_image_dockerfile_test'";
        assert!(run_container(container_runner, "", name, cmd, true).is_ok());
        let _ = remove_image(container_runner, name);
    }

    #[test]
    fn test_inspect_image() {
        let container_runner = &get_container_manager();
        let image_name = "ubuntu";
        run_container(container_runner, "", image_name, "ls", false).unwrap(); // for pull image

        let result = inspect_image(container_runner, image_name, ".Id");
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty(), "Failed to inspect image");
    }

    #[test]
    fn test_inspect_image_nonexistent_image() {
        let container_runner = &get_container_manager();
        let image_name = "nonexistent_image";

        let result = inspect_image(container_runner, image_name, ".Id");
        assert!(result.is_err());
    }

    #[test]
    fn test_inspect_image_invalid_format() {
        let container_runner = &get_container_manager();
        let image_name = "ubuntu";
        run_container(container_runner, "", image_name, "ls", false).unwrap(); // for pull image

        let result = inspect_image(container_runner, image_name, "112233");
        assert!(result.is_err());
    }

    #[test]
    fn test_inspect_image_nonexistent_config() {
        let container_runner = &get_container_manager();
        let image_name = "ubuntu";
        run_container(container_runner, "", image_name, "ls", false).unwrap(); // for pull image

        let result = inspect_image(container_runner, image_name, ".Config.Labels.Non");
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_empty(),
            "Failed to inspect nonexistent config"
        );
    }

    #[test]
    fn test_list_images_by_prefix() {
        let container_runner = &get_container_manager();
        // Test with a non-existent prefix - should return empty list
        let result = list_images_by_prefix(container_runner, "nonexistent-prefix-12345");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_list_images_by_prefix_with_images() {
        let container_runner = &get_container_manager();
        let test_id = format!("{}", std::process::id());
        let test_prefix = format!("distrobox-boost-test-list-{}", test_id);
        let image_name = "ubuntu";
        let container_name = format!("test_list_images_container_{}", test_id);
        let test_image = format!("{}/test:latest", test_prefix);

        // Clean up first (in case of previous failed run)
        let _ = remove_container(container_runner, &container_name);
        let _ = remove_image(container_runner, &test_image);

        // Create a test image with the prefix
        run_container(container_runner, &container_name, image_name, "ls", false).unwrap();
        commit_container(container_runner, &container_name, &test_image, &vec![]).unwrap();

        // List images with the prefix
        let result = list_images_by_prefix(container_runner, &test_prefix);
        assert!(result.is_ok());
        let images = result.unwrap();
        assert!(images.len() >= 1, "Should find at least one image");
        assert!(
            images.iter().any(|img| img.contains(&test_prefix)),
            "Should contain image with test prefix"
        );

        // Clean up
        let _ = remove_container(container_runner, &container_name);
        let _ = remove_image(container_runner, &test_image);
    }

    #[test]
    fn test_export_images_empty() {
        let container_runner = &get_container_manager();
        let result = export_images(container_runner, &[], "/tmp/test_empty.tar");
        assert!(result.is_err(), "Should fail with empty image list");
    }

    #[test]
    fn test_export_images() {
        let container_runner = &get_container_manager();
        let test_id = format!("{}", std::process::id());
        let test_prefix = format!("distrobox-boost-test-export-{}", test_id);
        let image_name = "ubuntu";
        let container_name = format!("test_export_images_container_{}", test_id);
        let export_path = format!("/tmp/distrobox_boost_test_export_{}.tar", test_id);
        let test_image = format!("{}/test:latest", test_prefix);

        // Clean up first (in case of previous failed run)
        let _ = remove_container(container_runner, &container_name);
        let _ = remove_image(container_runner, &test_image);
        let _ = std::fs::remove_file(&export_path);

        // Create a test image
        run_container(container_runner, &container_name, image_name, "ls", false).unwrap();
        commit_container(container_runner, &container_name, &test_image, &vec![]).unwrap();

        // Export the image
        let images = vec![test_image.clone()];
        let result = export_images(container_runner, &images, &export_path);
        assert!(result.is_ok(), "Export should succeed: {:?}", result.err());

        // Verify file exists
        assert!(
            std::path::Path::new(&export_path).exists(),
            "Export file should exist"
        );

        // Clean up
        let _ = remove_container(container_runner, &container_name);
        let _ = remove_image(container_runner, &test_image);
        let _ = std::fs::remove_file(&export_path);
    }

    #[test]
    fn test_import_images_nonexistent_file() {
        let container_runner = &get_container_manager();
        let result = import_images(container_runner, "/nonexistent/path/file.tar");
        assert!(result.is_err(), "Should fail with nonexistent file");
    }

    #[test]
    fn test_import_images() {
        let container_runner = &get_container_manager();
        let test_id = format!("{}", std::process::id());
        let test_prefix = format!("distrobox-boost-test-import-{}", test_id);
        let image_name = "ubuntu";
        let container_name = format!("test_import_images_container_{}", test_id);
        let export_path = format!("/tmp/distrobox_boost_test_import_{}.tar", test_id);
        let test_image = format!("{}/test:latest", test_prefix);

        // Clean up first (in case of previous failed run)
        let _ = remove_container(container_runner, &container_name);
        let _ = remove_image(container_runner, &test_image);
        let _ = std::fs::remove_file(&export_path);

        // Create and export a test image
        run_container(container_runner, &container_name, image_name, "ls", false).unwrap();
        commit_container(container_runner, &container_name, &test_image, &vec![]).unwrap();
        let images = vec![test_image.clone()];
        export_images(container_runner, &images, &export_path).unwrap();

        // Remove the image
        let _ = remove_image(container_runner, &test_image);

        // Import the image
        let result = import_images(container_runner, &export_path);
        assert!(result.is_ok(), "Import should succeed: {:?}", result.err());

        // Verify image exists after import
        let list_result = list_images_by_prefix(container_runner, &test_prefix);
        assert!(list_result.is_ok());
        assert!(
            !list_result.unwrap().is_empty(),
            "Image should exist after import"
        );

        // Clean up
        let _ = remove_container(container_runner, &container_name);
        let _ = remove_image(container_runner, &test_image);
        let _ = std::fs::remove_file(&export_path);
    }
}
