use crate::distro::os_info::parse_os_release;
use crate::distro::package_manager::*;
use crate::oci::command_helper::*;

pub fn build_image(
    container_runner: &str,
    image_name: &str,
    base_image: &str,
    packages: &Vec<String>,
) -> Result<String, CommandError> {
    let mut cmd = "cat /etc/os-release".to_string();
    let (stdout, _stderr) = run_container(container_runner, "", base_image, &cmd)?;
    println!("OS info: \n{}", stdout);
    let distro_info = parse_os_release(&stdout).unwrap();
    let package_manager = get_package_manager(&distro_info.0, &distro_info.1);

    let updated_container_name = format!("distrobox-{}-db-updated", image_name);
    println!("Update image: {}", updated_container_name);
    if !check_container_exists(container_runner, &updated_container_name)? {
        println!("Updating image: {}", base_image);
        cmd = generate_update_command(&package_manager);
        let (stdout, _stderr) =
            run_container(container_runner, &updated_container_name, base_image, &cmd)?;
        println!("Update command stdout: {}", stdout);
    } else {
        println!("Container {} already exists", updated_container_name);
        remove_container(container_runner, &updated_container_name)?;
    }
    let updated_image_name = format!("{}:distrobox-db-updated", image_name);
    if !check_image_exists(container_runner, &updated_image_name)? {
        println!("Commit image: {}", updated_image_name);
        let (_stdout, _stderr) = commit_container(
            container_runner,
            &updated_container_name,
            &updated_image_name,
        )?;
    } else {
        println!("Image {} already exists", updated_image_name);
    }
    println!(
        "Initial image name(with updated tag): {}",
        updated_image_name
    );
    if packages.is_empty() {
        return Ok(updated_image_name);
    }
    let mut last_image_name = updated_image_name.clone();
    let mut package_container_name = format!("distrobox-{}-pkg", image_name);
    let mut package_image_name = format!("{}:distrobox-pkg", image_name);
    for package in packages {
        package_container_name = format!("{}-{}", package_container_name, package);
        package_image_name = format!("{}-{}", package_image_name, package);
        println!("Install package: {}", package);
        if !check_container_exists(container_runner, &package_container_name)? {
            cmd = generate_install_command(&package_manager, &vec![package.as_str()]);
            let (stdout, _stderr) = run_container(
                container_runner,
                &package_container_name,
                &last_image_name,
                &cmd,
            )?;
            println!("Install command stdout: {}", stdout);
        } else {
            println!("Container {} already exists", package_container_name);
            remove_container(container_runner, &package_container_name)?;
        }
        if !check_image_exists(container_runner, &package_image_name)? {
            println!("Commit image: {}", package_image_name);
            let (_stdout, _stderr) = commit_container(
                container_runner,
                &package_container_name,
                &package_image_name,
            )?;
        } else {
            println!("Image {} already exists", package_image_name);
        }

        last_image_name = package_image_name.clone();
    }
    println!("Final image name: {}", last_image_name);
    Ok(last_image_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_image() {
        let container_runner = "podman";
        let image_name = "test_image";
        let base_image = "archlinux";
        let packages = vec!["bash".to_string(), "pacman".to_string()];

        let result = build_image(container_runner, image_name, base_image, &packages);

        match result {
            Ok(image_name) => {
                println!("Final image name: {}", image_name);
                assert!(image_name.contains("distrobox-pkg"));
            }
            Err(e) => {
                println!("Error building image: {:?}", e);
                assert!(false, "Error building image");
            }
        }
    }

    #[test]
    fn test_build_image_valid() {
        let container_runner = "podman";
        let image_name = "test_image";
        let base_image = "archlinux";
        let packages = vec!["fish".to_string(), "htop".to_string()];

        let result = build_image(container_runner, image_name, base_image, &packages).unwrap();
        println!("Final image name: {}", result);

        // Test if 'fish' and 'top' commands exist
        let commands = vec!["fish", "htop"];
        for command in commands {
            let cmd = format!("command -v {}", command);
            let result = run_container(container_runner, "", &result, &cmd);
            match result {
                Ok((stdout, _stderr)) => {
                    println!("Output of {}: {}", cmd, stdout);
                    assert!(!stdout.is_empty(), "Command {} does not exist", command);
                }
                Err(e) => {
                    println!("Error checking command {}: {:?}", command, e);
                    assert!(false, "Error checking command {}", command);
                }
            }
        }
    }
}
