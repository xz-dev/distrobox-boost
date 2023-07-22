use crate::distro::os_info::parse_os_release;
use crate::distro::package_manager::*;
use crate::oci::command_helper::*;
use crate::utils::mutex_lock::*;
use lazy_static::lazy_static;

pub fn build_image(
    container_runner: &str,
    image_name: &str,
    base_image: &str,
    packages: &Vec<String>,
) -> Result<String, CommandError> {
    let cmd = "cat /etc/os-release".to_string();
    let (stdout, _stderr) = run_container(container_runner, "", base_image, &cmd)?;
    println!("OS info: \n{}", stdout);
    let distro_info = parse_os_release(&stdout).unwrap();
    let package_manager = get_package_manager(&distro_info.0, &distro_info.1);

    let updated_image_name = format!(
        "distrobox-{}-db-updated",
        image_name.splitn(2, ":").next().unwrap()
    );
    let cmd = generate_update_command(&package_manager);
    println!("Update image: {}", updated_image_name);
    process_container(ContainerData {
        runner: container_runner,
        name: &updated_image_name,
        image: &base_image,
        cmd: &cmd,
    })?;
    println!(
        "Initial image name(with updated tag): {}",
        updated_image_name
    );
    if packages.is_empty() {
        return Ok(updated_image_name);
    }
    let mut last_image_name = updated_image_name.clone();
    let mut package_image_name = format!("distrobox-{}-pkg", image_name);
    for package in packages {
        package_image_name = format!("{}-{}", package_image_name, package);
        let cmd = generate_install_command(&package_manager, &vec![package.as_str()]);
        process_container(ContainerData {
            runner: container_runner,
            name: &package_image_name,
            image: &last_image_name,
            cmd: &cmd,
        })?;
        last_image_name = package_image_name.clone();
    }
    println!("Final image name: {}", last_image_name);
    Ok(last_image_name)
}

lazy_static! {
    static ref GLOBAL_SYNC_MAP: SynchronizedMap<String> = SynchronizedMap::new();
}

pub struct ContainerData<'a> {
    pub runner: &'a str,
    pub name: &'a str,
    pub image: &'a str,
    pub cmd: &'a str,
}

pub fn process_container(data: ContainerData) -> Result<(), CommandError> {
    GLOBAL_SYNC_MAP.execute(data.name.to_string(), || -> Result<(), CommandError> {
        if !check_image_exists(data.runner, &data.name)? {
            if !check_container_exists(data.runner, &data.name)? {
                println!("Running container: {}", &data.name);
                let (stdout, _stderr) =
                    run_container(data.runner, &data.name, &data.image, &data.cmd)?;
                println!("Command stdout: {}", stdout);
            } else {
                println!("Container {} already exists", data.name);
            }
            println!("Commit image: {}", &data.name);
            let (_stdout, _stderr) = commit_container(data.runner, &data.name, &data.name)?;
            remove_container(data.runner, &data.name)?;
        } else {
            println!("Image {} already exists", data.name);
        }

        Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_container_manager;

    #[test]
    fn test_build_image() {
        let container_runner = &get_container_manager();
        let image_name = "test_image";
        let base_image = "archlinux";
        let packages = vec!["bash".to_string(), "pacman".to_string()];

        let result = build_image(container_runner, image_name, base_image, &packages);

        match result {
            Ok(image_name) => {
                println!("Final image name: {}", image_name);
                assert_eq!(image_name, "distrobox-test_image-pkg-bash-pacman");
            }
            Err(e) => {
                println!("Error building image: {:?}", e);
                assert!(false, "Error building image");
            }
        }
    }

    #[test]
    fn test_build_image_valid() {
        let container_runner = &get_container_manager();
        let image_name = "test_image";
        let base_image = "archlinux:latest";
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
