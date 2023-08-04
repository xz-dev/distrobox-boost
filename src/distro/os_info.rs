pub fn parse_os_release(content: &str) -> Option<(String, String)> {
    let mut os_id = None;
    let mut os_version_id = None;

    for line in content.lines() {
        if line.starts_with("ID=") {
            os_id = Some(line[3..].trim_matches('"').to_string());
        } else if line.starts_with("VERSION_ID=") {
            os_version_id = Some(line[11..].trim_matches('"').to_string());
        }
    }

    match (os_id, os_version_id) {
        (Some(id), Some(version_id)) => Some((id, version_id)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_container_manager;
    use crate::oci::command_helper::run_container;

    macro_rules! create_test {
        ($test_name:ident, $image_name:expr) => {
            #[test]
            fn $test_name() {
                test_parse_os_release(&get_container_manager(), $image_name);
            }
        };
    }

    create_test!(test_parse_os_release_apk, "alpine");
    create_test!(test_parse_os_release_pacman, "archlinux");
    create_test!(test_parse_os_release_yum, "centos");
    create_test!(test_parse_os_release_yum_rockylinux, "rockylinux:9");
    create_test!(test_parse_os_release_apt, "debian");
    create_test!(
        test_parse_os_release_dnf,
        "registry.fedoraproject.org/fedora-toolbox"
    );
    create_test!(
        test_parse_os_release_zypper,
        "registry.opensuse.org/opensuse/distrobox:latest"
    );
    create_test!(test_parse_os_release_apt_ubuntu, "ubuntu");

    fn test_parse_os_release(container_runner: &str, image_name: &str) {
        let output = run_container(
            container_runner,
            "",
            image_name,
            "cat /etc/os-release",
            true,
        )
        .unwrap();
        let parsed = parse_os_release(&output.stdout);
        assert!(
            parsed.is_some(),
            "Failed to parse OS release info for image \"{}\"",
            image_name
        );
    }
}
