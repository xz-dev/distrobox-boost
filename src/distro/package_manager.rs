pub fn generate_update_command(package_manager: &str) -> String {
    match package_manager {
        "apk" => "apk update".to_string(),
        "pacman" => "pacman -S -y -y".to_string(),
        "yum" => "yum makecache".to_string(),
        "apt" => "apt-get update".to_string(),
        "dnf" => "dnf makecache".to_string(),
        "zypper" => "zypper refresh".to_string(),
        _ => String::new(),
    }
}
pub fn generate_install_command(package_manager: &str, packages: &[&str]) -> String {
    let packages_str = packages.join(" ");
    match package_manager {
        "apk" => format!("apk add --no-cache {}", packages_str),
        "pacman" => format!("pacman -S --needed --noconfirm {}", packages_str),
        "yum" => format!("yum -y install --nobest {}", packages_str),
        "apt" => format!("apt-get install -y {}", packages_str),
        "dnf" => format!("dnf -y install {}", packages_str),
        "zypper" => format!("zypper --non-interactive install {}", packages_str),
        _ => String::new(),
    }
}

static PACKAGE_MAP: &[(&str, &str)] = &[
    ("alpine", "apk"),
    ("arch", "pacman"),
    ("centos", "yum"),
    ("rocky", "yum"),
    ("debian", "apt"),
    ("fedora", "dnf"),
    ("opensuse", "zypper"),
    ("ubuntu", "apt"),
];
pub fn get_package_manager(distro_id: &str, _distro_version: &str) -> String {
    // distr_info: cat /etc/os-release
    // data from os_info.rs parse_os_release
    let mut package_manager = String::new();
    if let Some((_, manager)) = PACKAGE_MAP.iter().find(|(key, _)| key == &distro_id) {
        package_manager = manager.to_string();
    } else if let Some((_, manager)) = PACKAGE_MAP.iter().find(|(key, _)| distro_id.contains(key)) {
        package_manager = manager.to_string();
    }
    package_manager
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_container_manager;
    use crate::distro::os_info::parse_os_release;
    use crate::oci::command_helper::run_container;

    fn test_package_installation_single_image(
        container_runner: &str,
        image_name: &str,
        package_manager_name: &str,
    ) {
        println!("Testing package installation for image: {}", image_name);

        // Generate and run the update command
        let update_cmd = generate_update_command(package_manager_name);
        let update_result = run_container(container_runner, "", image_name, &update_cmd);
        match update_result {
            Ok((stdout, stderr)) => {
                // Check the outputs here if necessary
                println!("Update command stdout: {}", stdout);
                println!("Update command stderr: {}", stderr);
            }
            Err(command_error) => {
                panic!(
                    "Failed to run update command on image {}. stdout: {}, stderr: {}",
                    image_name, command_error.stdout, command_error.stderr
                );
            }
        }

        // Generate and run the install command
        let packages = ["bash", "nano"];
        let install_cmd = format!(
            "{} ; {}",
            update_cmd,
            generate_install_command(package_manager_name, &packages)
        );
        let install_result = run_container(container_runner, "", image_name, &install_cmd);
        match install_result {
            Ok((stdout, stderr)) => {
                // Check the outputs here if necessary
                println!("Install command stdout: {}", stdout);
                println!("Install command stderr: {}", stderr);
            }
            Err(command_error) => {
                panic!(
                    "Failed to run install command on image {}. stdout: {}, stderr: {}",
                    image_name, command_error.stdout, command_error.stderr
                );
            }
        }
    }
    macro_rules! generate_package_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (image, package_manager) = $value;
            test_package_installation_single_image(&get_container_manager(), image, package_manager);
        }
    )*
    }
}

    generate_package_tests! {
        test_package_installation_apk: ("alpine:latest", "apk"),
        test_package_installation_pacman: ("archlinux", "pacman"),
        test_package_installation_yum: ("rockylinux:9", "yum"),
        test_package_installation_apt: ("debian", "apt"),
        test_package_installation_dnf: ("registry.fedoraproject.org/fedora-toolbox:latest", "dnf"),
        test_package_installation_zypper: ("registry.opensuse.org/opensuse/distrobox:latest", "zypper"),
    }

    fn test_get_package_manager_real_image(
        container_runner: &str,
        image_name: &str,
        expected_package_manager: &str,
    ) {
        // Run the container and use parse_os_release to get os info
        let os_info_cmd = "cat /etc/os-release";
        let os_info_result = run_container(container_runner, "", image_name, os_info_cmd);

        let (distro_name, distro_version) = match os_info_result {
            Ok((stdout, _stderr)) => parse_os_release(&stdout),
            Err(e) => panic!("Error getting OS info from image {}: {:?}", image_name, e),
        }
        .unwrap_or_else(|| panic!("Could not parse OS info from image {}", image_name));

        // Test get_package_manager
        assert_eq!(
            get_package_manager(&distro_name, &distro_version),
            expected_package_manager,
            "Package manager for image {} does not match",
            image_name
        );
    }

    macro_rules! generate_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (image, package_manager) = $value;
            test_get_package_manager_real_image(&get_container_manager(), image, package_manager);
        }
    )*
    }
}

    generate_tests! {
        test_get_package_manager_alpine: ("alpine", "apk"),
        test_get_package_manager_arch: ("archlinux", "pacman"),
        test_get_package_manager_centos: ("centos:centos7", "yum"),
        test_get_package_manager_rockylinux: ("rockylinux:9", "yum"),
        test_get_package_manager_debian: ("debian", "apt"),
        test_get_package_manager_fedora: ("registry.fedoraproject.org/fedora-toolbox:latest", "dnf"),
        test_get_package_manager_opensuse: ("registry.opensuse.org/opensuse/distrobox:latest", "zypper"),
        test_get_package_manager_ubuntu: ("ubuntu", "apt"),
    }
}
