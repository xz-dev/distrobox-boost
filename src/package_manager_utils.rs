use crate::oci_utils::run_container;
use std::io;

static PACKAGE_MAP: &[(&str, &str)] = &[
    ("alpine", "apk"),
    ("arch", "pacman"),
    ("centos", "yum"),
    ("debian", "apt"),
    ("fedora", "dnf"),
    ("opensuse", "zypper"),
    ("ubuntu", "apt"),
];

fn get_package_manager(distro_info: &str) -> String {
    let mut package_manager = String::new();
    for line in distro_info.lines() {
        if line.contains("ID=") {
            let mut line_split = line.split('=');
            line_split.next();
            let distro_id = line_split.next().unwrap().trim_matches('\"').to_string();
            if let Some((_, manager)) = PACKAGE_MAP.iter().find(|(key, _)| key == &distro_id) {
                package_manager = manager.to_string();
            } else if let Some((_, manager)) =
                PACKAGE_MAP.iter().find(|(key, _)| distro_id.contains(key))
            {
                package_manager = manager.to_string();
            }
        }
    }
    package_manager
}

fn get_distro_info(container_runner: &str, image_name: &str) -> Result<String, io::Error> {
    let (stdout, _stderr) = run_container(container_runner, "", image_name, "cat /etc/os-release")?;
    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_distro_info() {
        let container_runner = "podman";
        let image_name = "ubuntu";
        let distro_info_result = get_distro_info(container_runner, image_name);

        match distro_info_result {
            Ok(distro_info) => {
                assert!(distro_info.contains(r#"NAME="Ubuntu""#));
            }
            Err(e) => panic!("Error occurred: {}", e),
        }
    }

    #[test]
    fn test_get_package_manager() {
        let distro_info_opensuse = r#"
NAME="openSUSE Tumbleweed"
# VERSION="20230714"
ID="opensuse-tumbleweed"
ID_LIKE="opensuse suse"
VERSION_ID="20230714"
PRETTY_NAME="openSUSE Tumbleweed"
ANSI_COLOR="0;32"
CPE_NAME="cpe:/o:opensuse:tumbleweed:20230714"
BUG_REPORT_URL="https://bugzilla.opensuse.org"
SUPPORT_URL="https://bugs.opensuse.org"
HOME_URL="https://www.opensuse.org"
DOCUMENTATION_URL="https://en.opensuse.org/Portal:Tumbleweed"
LOGO="distributor-logo-Tumbleweed"#;

        let distro_info_ubuntu = r#"
PRETTY_NAME="Ubuntu 22.04.2 LTS"
NAME="Ubuntu"
VERSION_ID="22.04"
VERSION="22.04.2 LTS (Jammy Jellyfish)"
VERSION_CODENAME=jammy
ID=ubuntu
ID_LIKE=debian
HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
PRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"
UBUNTU_CODENAME=jammy"#;

        assert_eq!(get_package_manager(distro_info_opensuse), "zypper");
        assert_eq!(get_package_manager(distro_info_ubuntu), "apt");
    }
}
