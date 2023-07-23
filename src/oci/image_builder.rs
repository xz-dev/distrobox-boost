use crate::config::get_distrobox_mode;
use crate::distro::distrobox_requirements::get_distrobox_packages;
use crate::distro::os_info::parse_os_release;
use crate::distro::package_manager::*;
use crate::oci::command_helper::*;
use crate::utils::mutex_lock::*;
use lazy_static::lazy_static;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn build_image(
    container_runner: &str,
    image_name: &str,
    base_image: &str,
    packages: &Vec<String>,
) -> Result<String, CommandError> {
    let cmd = "cat /etc/os-release".to_string();
    let (stdout, _stderr) = run_container(container_runner, "", base_image, &cmd)?;
    let distro_info = parse_os_release(&stdout).unwrap();
    let package_manager = get_package_manager(&distro_info.0, &distro_info.1);
    let slim_image_name = image_name.replace(":", "_");

    fn get_seconds_since_epoch() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::new(0, 0));
        since_the_epoch.as_secs()
    }

    let cmd = generate_update_command(&package_manager);
    println!("Update image: {}", slim_image_name);
    let mut final_image_name = process_container(ContainerData {
        runner: container_runner,
        expect_image: &format!("{}:db_updated", slim_image_name),
        base_image: &base_image,
        cmd: &cmd,
        filters: &vec![
            format!("label=image={}", base_image).as_str(),
            "label=status=db_update",
        ],
        instructions: &vec![
            format!("LABEL image={}", base_image).as_str(),
            "LABEL status=db_update",
            format!("LABEL updated_at={}", get_seconds_since_epoch()).as_str(),
        ],
    })?;
    println!("Updated image: {}", final_image_name);
    if get_distrobox_mode() {
        println!("Install distrobox requirements");
        let packages = get_distrobox_packages(&distro_info.0);
        let cmd = generate_install_command(&package_manager, &packages);
        final_image_name = process_container(ContainerData {
            runner: container_runner,
            expect_image: &format!("{}:distrobox_pre", slim_image_name),
            base_image: &final_image_name,
            cmd: &cmd,
            filters: &vec![
                format!("label=image={}", base_image).as_str(),
                "label=status=distrobox_pre_install",
                format!("label=packages0={}", packages.join(";")).as_str(),
            ],
            instructions: &vec![
                "LABEL status=distrobox_pre_install",
                format!("LABEL packages0={}", packages.join(";")).as_str(),
                format!("LABEL updated_at={}", get_seconds_since_epoch()).as_str(),
            ],
        })?;
    }
    println!("Initial image name(with updated tag): {}", final_image_name);
    if packages.is_empty() {
        println!("Final snap image name: {}", final_image_name);
        let (_stdout, _stderr) = tag_image(container_runner, &final_image_name, image_name)?;
        return Ok(image_name.to_string());
    }
    let mut package_label = String::new();
    for package in packages {
        package_label = format!("{}{};", package_label, package);
        let package_label_trimmed = package_label[..package_label.len() - 1].to_string();
        let cmd = generate_install_command(&package_manager, &vec![package.as_str()]);
        let in_seceonds = get_seconds_since_epoch();
        final_image_name = process_container(ContainerData {
            runner: container_runner,
            expect_image: &format!("{}:pkg-{}-{}", slim_image_name, hash(&package_label_trimmed), in_seceonds),
            base_image: &final_image_name,
            cmd: &cmd,
            filters: &vec![
                format!("label=image={}", base_image).as_str(),
                "label=status=package_install",
                format!("label=package1={}", package_label_trimmed).as_str(),
            ],
            instructions: &vec![
                "LABEL status=package_install",
                format!("LABEL package1={}", package_label_trimmed).as_str(),
                format!("LABEL updated_at={}", in_seceonds).as_str(),
            ],
        })?;
        println!("Package installed: {} at {}", package, final_image_name);
    }

    println!("Final snap image name: {}", final_image_name);
    let (_stdout, _stderr) = tag_image(container_runner, &final_image_name, image_name)?;
    Ok(image_name.to_string())
}

lazy_static! {
    static ref GLOBAL_SYNC_MAP: SynchronizedMap<String> = SynchronizedMap::new();
}

pub struct ContainerData<'a> {
    pub runner: &'a str,
    pub base_image: &'a str,
    pub cmd: &'a str,
    pub expect_image: &'a str,
    pub filters: &'a [&'a str],
    pub instructions: &'a [&'a str],
}

fn process_container(data: ContainerData) -> Result<String, CommandError> {
    let container_runner = data.runner;
    GLOBAL_SYNC_MAP.execute(
        data.expect_image.to_string(),
        || -> Result<String, CommandError> {
            let image_id_list = find_images(container_runner, data.filters)?;
            if !image_id_list.is_empty() {
                let image_list = get_image_name(container_runner, &image_id_list.first().unwrap())?;
                if let Some(image_list) = image_list {
                    if let Some(image_name) = image_list.first() {
                        println!("Image {} already exists", image_name);
                        return Ok(image_name.to_string());
                    }
                }
            }

            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let in_millis = since_the_epoch.as_millis();
            let container_name = format!("{}-{}", data.expect_image.replace(":", "-"), in_millis);

            if !check_container_exists(data.runner, &container_name)? {
                println!("Running container: {}", &container_name);
                let (stdout, _stderr) =
                    run_container(data.runner, &container_name, &data.base_image, &data.cmd)?;
                println!("Command stdout: {}", stdout);
            } else {
                println!("Container {} already exists", &container_name);
            }

            println!(
                "Commit image: {} by {}",
                &data.expect_image, &container_name
            );
            let (_stdout, _stderr) = commit_container(
                data.runner,
                &container_name,
                &data.expect_image,
                data.instructions,
            )?;
            remove_container(data.runner, &container_name)?;

            Ok(data.expect_image.to_string())
        },
    )
}

fn hash<T: Hash>(t: T) -> u16 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    (s.finish() & 0xffff) as u16
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
                println!("Build image name: {}", image_name);
                assert_eq!(image_name, "test_image");
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
