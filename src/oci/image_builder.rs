use crate::config::get_distrobox_mode;
use crate::distro::distrobox_requirements::get_distrobox_packages;
use crate::distro::os_info::parse_os_release;
use crate::distro::package_manager::*;
use crate::oci::command_helper::*;
use crate::utils::command_helper::CommandError;
use crate::utils::mutex_lock::*;
use lazy_static::lazy_static;
use std::collections::hash_map::{DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn pre_build_image(
    container_runner: &str,
    target_image: &str,
    base_image: &str,
    image_prefix: &str,
) -> Result<String, CommandError> {
    if base_image.starts_with("dockerfile://") {
        println!("Build dockerfile: {}", &base_image);
        let image_name = format!("{}/dockerfile/{}", &image_prefix, &target_image);
        let dockerfile_path = &base_image[12..];
        let dockerfile_content_path = Path::new(dockerfile_path)
            .parent()
            .unwrap()
            .to_str()
            .unwrap();
        let _ = build_image_from_dockerfile_simple(
            container_runner,
            &image_name,
            &dockerfile_path,
            dockerfile_content_path,
        )?;
        Ok(image_name.to_string())
    } else {
        Ok(base_image.to_string())
    }
}

pub fn build_image(
    container_runner: &str,
    target_image: &str,
    base_image: &str,
    request_package_manager: &Option<String>,
    packages: &Vec<String>,
    image_prefix: &str,
    distrobox_mode: bool,
) -> Result<String, CommandError> {
    let base_image = pre_build_image(container_runner, target_image, base_image, image_prefix)?;
    let cmd = "cat /etc/os-release".to_string();
    let output = run_container(container_runner, "", &base_image, &cmd, true)?;
    let distro_info = parse_os_release(&output.stdout).unwrap();
    let package_manager = request_package_manager
        .clone()
        .unwrap_or(get_package_manager(&distro_info.0, &distro_info.1));
    let slim_image_name = format!(
        "{}/builder/{}",
        image_prefix,
        target_image.replace(":", "-").replace("/", "_")
    );

    let mut filter_map = HashMap::new();
    filter_map.insert("image".to_string(), base_image.to_string());

    fn _run_and_commit_image(
        runner: &str,
        cmd: &str,
        target_image: &str,
        base_image: &str,
        filter_map: &HashMap<String, String>,
        realtime_output: bool,
    ) -> Result<(), CommandError> {
        run_and_commit_image(&ContainerData {
            runner,
            cmd,
            target_image,
            base_image,
            filters: &get_filter_vec(filter_map)
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .as_slice(),
            instructions: &get_instructions(filter_map)
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .as_slice(),
            realtime_output,
        })
    }

    let cmd = generate_update_command(&package_manager);
    println!("Update image: {}", slim_image_name);
    let updated_image = format!("{}:db_updated", slim_image_name);
    filter_map.insert("status".to_string(), "db_update".to_string());
    _run_and_commit_image(
        container_runner,
        &cmd,
        &updated_image,
        &base_image,
        &filter_map,
        true,
    )?;
    println!("Updated image: {}", updated_image);
    let mut basic_package_image = updated_image.clone();
    if get_distrobox_mode() {
        println!("Install distrobox requirements");
        let packages = get_distrobox_packages(&distro_info.0, &distro_info.1);
        let cmd = generate_install_command(&package_manager, &packages);
        basic_package_image = format!("{}:distrobox_pre", slim_image_name);
        filter_map.insert("status".to_string(), "distrobox_pre_install".to_string());
        filter_map.insert("packages0".to_string(), packages.join(";"));
        _run_and_commit_image(
            container_runner,
            &cmd,
            &basic_package_image,
            &updated_image,
            &filter_map,
            true,
        )?;
    }
    println!(
        "Initial image name(with updated tag): {}",
        basic_package_image
    );
    if !packages.is_empty() {
        let mut installed_packages = Vec::new();
        for package in packages {
            installed_packages.push(package);
            let package_label = installed_packages
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(";");
            let cmd = generate_install_command(&package_manager, &vec![package.as_str()]);
            let in_seceonds = get_seconds();
            let package_installed_image = format!(
                "{}:pkg{}-{}{}",
                installed_packages.len(),
                slim_image_name,
                hash(&package_label),
                in_seceonds
            );
            filter_map.insert("status".to_string(), "package_install".to_string());
            filter_map.insert("package1".to_string(), package_label);
            _run_and_commit_image(
                container_runner,
                &cmd,
                &package_installed_image,
                &basic_package_image,
                &filter_map,
                true,
            )?;
            println!(
                "Package installed at {}\nPackages: {}",
                package_installed_image, package
            );
            basic_package_image = package_installed_image;
        }
    }

    if distrobox_mode {
        println!("Touch /run/.containersetupdone for distrobox");
        let distrobox_setup_tag_image = format!("{}:mark_distrobox_setup_done", slim_image_name);
        let cmd = "touch /run/.containersetupdone";
        filter_map.insert("status".to_string(), "distrobox_setup".to_string());
        _run_and_commit_image(
            container_runner,
            &cmd,
            &distrobox_setup_tag_image,
            &basic_package_image,
            &filter_map,
            false,
        )?;
        basic_package_image = distrobox_setup_tag_image;
    }

    println!("Final snap image name: {}", basic_package_image);
    tag_image(container_runner, &basic_package_image, target_image)?;
    Ok(target_image.to_string())
}

fn get_filter_vec(filter_map: &HashMap<String, String>) -> Vec<String> {
    let mut filter_vec = Vec::new();
    for (key, value) in filter_map {
        filter_vec.push(format!("label={}={}", key, value));
    }
    filter_vec
}

fn get_instructions(filter_map: &HashMap<String, String>) -> Vec<String> {
    let mut instructions = Vec::new();
    for (key, value) in filter_map {
        instructions.push(format!("LABEL {}={}", key, value));
    }
    instructions.push(format!("LABEL updated_at={}", get_seconds()));
    instructions
}

lazy_static! {
    static ref GLOBAL_SYNC_MAP: SynchronizedMap<String> = SynchronizedMap::new();
}

pub struct ContainerData<'a> {
    pub runner: &'a str,
    pub cmd: &'a str,
    pub base_image: &'a str,
    pub target_image: &'a str,
    pub filters: &'a [&'a str],
    pub instructions: &'a [&'a str],
    pub realtime_output: bool,
}

fn recommit_image(
    container_runner: &str,
    target_image: &str,
    image_id_list: &[String],
) -> Result<(), CommandError> {
    let image_id = image_id_list.first().unwrap();
    println!("Image {} already exists", image_id);
    tag_image(container_runner, image_id, target_image)?;
    println!("Tagged image: {} by {}", target_image, image_id);
    return Ok(());
}

fn create_new_image(data: &ContainerData) -> Result<(), CommandError> {
    let container_name = format!(
        "{}-{}",
        data.target_image
            .rsplit_once("/")
            .unwrap_or(("", data.target_image))
            .1
            .replace(":", "-"),
        get_seconds()
    );

    if !check_container_exists(data.runner, &container_name)? {
        println!("Running container: {}", &container_name);
        let output = run_container(
            data.runner,
            &container_name,
            &data.base_image,
            &data.cmd,
            data.realtime_output,
        )?;
        if let Some(status) = output.status {
            println!("status: {}", status);
        }
    } else {
        println!("Container {} already exists", &container_name);
    }

    println!(
        "Commit image: {} by {}",
        &data.target_image, &container_name
    );
    commit_container(
        data.runner,
        &container_name,
        &data.target_image,
        data.instructions,
    )?;
    remove_container(data.runner, &container_name)?;

    Ok(())
}

fn run_and_commit_image(data: &ContainerData) -> Result<(), CommandError> {
    let key = data.filters.join(";");
    GLOBAL_SYNC_MAP.execute(key, || -> Result<(), CommandError> {
        let image_id_list = find_images(data.runner, data.filters)?;

        if !image_id_list.is_empty() {
            recommit_image(data.runner, data.target_image, &image_id_list)?;
        } else {
            create_new_image(&data)?;
        };

        Ok(())
    })
}

fn hash<T: Hash>(t: T) -> u8 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    (s.finish() & 0xffff) as u8
}

fn get_seconds() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{get_container_manager, set_distrobox_mode, get_distrobox_boost_test_image_prefix};

    #[test]
    fn test_build_image() {
        let container_runner = &get_container_manager();
        let image_name = "test_image";
        let base_image = "ubuntu";
        let packages = vec!["bash".to_string(), "pacman".to_string()];

        let result = build_image(
            container_runner,
            image_name,
            base_image,
            &None,
            &packages,
            &get_distrobox_boost_test_image_prefix(),
            false,
        );

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
        let image_name = "test_build_image_valid";
        let base_image = "ubuntu";
        let packages = vec!["fish".to_string(), "htop".to_string()];

        let result = build_image(
            container_runner,
            image_name,
            base_image,
            &None,
            &packages,
            &get_distrobox_boost_test_image_prefix(),
            true,
        )
        .unwrap();
        println!("Final image name: {}", result);

        // Test if 'fish' and 'top' commands exist
        let commands = vec!["fish", "htop"];
        for command in commands {
            let cmd = format!("command -v {}", command);
            let result = run_container(container_runner, "", &result, &cmd, true);
            assert!(result.is_ok(), "Error running command {}", command);
        }

        let result = run_container(
            container_runner,
            "",
            &result,
            "ls /run/.containersetupdone",
            true,
        );
        assert!(
            result.is_ok(),
            "/run/.containersetupdone not found at {}",
            image_name
        );
    }
    #[test]
    fn test_build_image_valid_no_distrobox_mode() {
        let container_runner = &get_container_manager();
        let image_name = "test_build_image_valid_no_distrobox_mode";
        let base_image = "ubuntu";
        let packages = vec!["fish".to_string(), "htop".to_string()];

        let result = build_image(
            container_runner,
            image_name,
            base_image,
            &None,
            &packages,
            &get_distrobox_boost_test_image_prefix(),
            false,
        )
        .unwrap();
        println!("Final image name: {}", result);

        // Test if 'fish' and 'top' commands exist
        let commands = vec!["fish", "htop"];
        for command in commands {
            let cmd = format!("command -v {}", command);
            let result = run_container(container_runner, "", &result, &cmd, false);
            assert!(result.is_ok(), "Error running command {}", command);
        }

        let result = run_container(
            container_runner,
            "",
            &result,
            "ls /run/.containersetupdone",
            true,
        );
        assert!(
            result.is_err(),
            "/run/.containersetupdone found at {} in no distrobox mode",
            image_name
        );
    }
    #[test]
    fn test_build_image_valid_distrobox_mode() {
        let container_runner = &get_container_manager();
        let image_name = "test_image_distrobox_mode";
        let base_image = "ubuntu";
        let packages = vec![];

        set_distrobox_mode(true);
        let result = build_image(
            container_runner,
            image_name,
            base_image,
            &None,
            &packages,
            &get_distrobox_boost_test_image_prefix(),
            true,
        )
        .unwrap();
        println!("Final image name: {}", result);

        let result = run_container(
            container_runner,
            "",
            &result,
            "ls /run/.containersetupdone",
            true,
        );
        assert!(
            result.is_ok(),
            "/run/.containersetupdone not found at {}",
            image_name
        );
    }

    #[test]
    fn test_create_new_image() {
        let container_runner = &get_container_manager();
        let target_image = "test_create_new_image";
        let base_image = "ubuntu";
        let mut filter_map = HashMap::new();
        filter_map.insert("test".to_string(), "test_create_new_image".to_string());

        let _ = remove_image(container_runner, &target_image);
        let image_filter = get_filter_vec(&filter_map);

        let result = create_new_image(&ContainerData {
            runner: container_runner,
            cmd: "touch /test_create_new_image",
            base_image,
            target_image,
            filters: &image_filter
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>(),
            instructions: &get_instructions(&filter_map)
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .as_slice(),
            realtime_output: true,
        });

        let filter_images = find_images(
            container_runner,
            &image_filter
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .as_slice(),
        )
        .unwrap();
        assert!(!filter_images.is_empty());
        let _ = remove_image(container_runner, &target_image);

        assert!(result.is_ok());
    }
}
