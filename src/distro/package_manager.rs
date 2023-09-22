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
        "yum" => format!("yum -y install --skip-broken {}", packages_str),
        "apt" => format!("apt-get install -y {}", packages_str),
        "dnf" => format!("dnf -y install {}", packages_str),
        "zypper" => format!("zypper --non-interactive install {}", packages_str),
        _ => String::new(),
    }
}

pub fn generate_list_packages_command(package_manager: &str) -> String {
    match package_manager {
        "apk" => "apk info -v".to_string(),
        "pacman" => "pacman -Q".to_string(),
        "yum" => "yum list installed".to_string(),
        "apt" => "apt list --installed".to_string(),
        "dnf" => "dnf list installed".to_string(),
        "zypper" => "zypper se -s --installed-only ".to_string(),
        _ => String::new(),
    }
}

pub fn parse_list_packages_command_output(
    package_manager: &str,
    output: &str,
) -> Vec<(String, String)> {
    match package_manager {
        "apk" => output
            .lines()
            .map(|line| {
                let parts = line.rsplit_once('-').unwrap();
                let name = parts.0.to_string();
                let version = parts.1.to_string();
                (name, version)
            })
            .collect(),

        "pacman" => output
            .lines()
            .map(|line| {
                let parts = line.rsplit_once(' ').unwrap();
                let name = parts.0.to_string();
                let version = parts.1.to_string();
                (name, version)
            })
            .collect(),

        "yum" => output
            .lines()
            .skip(1)
            .map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next().unwrap().to_string();
                let version = parts.next().unwrap().to_string();
                (name, version)
            })
            .collect(),

        "apt" => output
            .lines()
            .skip(1)
            .filter_map(|line| {
                // Skip the first line
                let mut parts = line.split('/');
                if let Some(name) = parts.next() {
                    let version = parts
                        .next()
                        .unwrap_or("")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    Some((name.to_string(), version))
                } else {
                    None
                }
            })
            .collect(),

        "dnf" => output
            .lines()
            .skip(1)
            .map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next().unwrap().to_string();
                let version = parts.next().unwrap().to_string();
                (name, version)
            })
            .collect(),

        "zypper" => output
            .lines()
            .skip(2)
            .filter_map(|line| {
                // Skip the first 2 lines
                let mut parts = line.split('|');
                if let (Some(name), Some(version)) =
                    (parts.nth(1).map(str::trim), parts.nth(2).map(str::trim))
                {
                    Some((name.to_string(), version.to_string()))
                } else {
                    None
                }
            })
            .collect(),

        _ => vec![],
    }
}

pub fn get_package_manager(distro_id: &str, _distro_version: &str) -> String {
    // distr_info: cat /etc/os-release
    // data from os_info.rs parse_os_release
    const PACKAGE_MAP: &[(&str, &str)] = &[
        ("alpine", "apk"),
        ("arch", "pacman"),
        ("centos", "yum"),
        ("rocky", "yum"),
        ("debian", "apt"),
        ("fedora", "dnf"),
        ("opensuse", "zypper"),
        ("ubuntu", "apt"),
    ];

    if let Some((_, manager)) = PACKAGE_MAP.iter().find(|(key, _)| distro_id.contains(key)) {
        return manager.to_string();
    }

    String::new()
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
        let update_result = run_container(container_runner, "", image_name, &update_cmd, true);
        assert!(
            update_result.is_ok(),
            "Failed to run update command on image {}",
            image_name
        );

        // Generate and run the install command
        let packages = ["bash", "nano"];
        let install_cmd = format!(
            "{} ; {}",
            update_cmd,
            generate_install_command(package_manager_name, &packages)
        );
        let install_result = run_container(container_runner, "", image_name, &install_cmd, true);
        assert!(
            install_result.is_ok(),
            "Failed to run install command on image {}",
            image_name
        );
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

    fn test_package_list_command_single_image(
        container_runner: &str,
        image_name: &str,
        package_manager_name: &str,
    ) {
        println!("Testing package list command for image: {}", image_name);
        let list_packages_cmd = generate_list_packages_command(package_manager_name);
        let list_packages_result =
            run_container(container_runner, "", image_name, &list_packages_cmd, true);
        assert!(
            list_packages_result.is_ok(),
            "Failed to run install command on image {}",
            image_name
        );
        let list_packages_output = list_packages_result.unwrap().stdout;
        assert!(
            !list_packages_output.is_empty(),
            "Package list command output is empty"
        );
        let package_list =
            parse_list_packages_command_output(package_manager_name, &list_packages_output);
        assert!(!package_list.is_empty(), "Package list is empty");
        for (i, (name, version)) in package_list.iter().enumerate() {
            assert!(!name.is_empty(), "Package at index {} has an empty name", i);
            assert!(
                !version.is_empty(),
                "Package at index {} has an empty version: {}",
                i,
                name
            );
        }
    }
    macro_rules! generate_package_list_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (image, package_manager) = $value;
            test_package_list_command_single_image(&get_container_manager(), image, package_manager);
        }
    )*
    }
}

    generate_package_list_tests! {
        test_package_list_apk: ("alpine:latest", "apk"),
        test_package_list_pacman: ("archlinux", "pacman"),
        test_package_list_yum: ("rockylinux:9", "yum"),
        test_package_list_apt: ("debian", "apt"),
        test_package_list_dnf: ("registry.fedoraproject.org/fedora-toolbox:latest", "dnf"),
        test_package_list_zypper: ("registry.opensuse.org/opensuse/distrobox:latest", "zypper"),
    }

    fn test_get_package_manager_real_image(
        container_runner: &str,
        image_name: &str,
        expected_package_manager: &str,
    ) {
        // Run the container and use parse_os_release to get os info
        let os_info_cmd = "cat /etc/os-release";
        let os_info_result = run_container(container_runner, "", image_name, os_info_cmd, true);

        let (distro_name, distro_version) = match os_info_result {
            Ok(output) => parse_os_release(&output.stdout),
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
