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
    use crate::oci::command_helper::run_container;

    #[test]
    fn test_parse_os_release_apk() {
        test_parse_os_release("podman", "alpine");
    }

    #[test]
    fn test_parse_os_release_pacman() {
        test_parse_os_release("podman", "archlinux");
    }

    #[test]
    fn test_parse_os_release_yum() {
        test_parse_os_release("podman", "centos");
    }

    #[test]
    fn test_parse_os_release_yum_rockylinux() {
        test_parse_os_release("podman", "rockylinux:9");
    }

    #[test]
    fn test_parse_os_release_apt() {
        test_parse_os_release("podman", "debian");
    }

    #[test]
    fn test_parse_os_release_dnf() {
        test_parse_os_release("podman", "registry.fedoraproject.org/fedora-toolbox");
    }

    #[test]
    fn test_parse_os_release_zypper() {
        test_parse_os_release("podman", "registry.opensuse.org/opensuse/distrobox:latest");
    }

    #[test]
    fn test_parse_os_release_apt_ubuntu() {
        test_parse_os_release("podman", "ubuntu");
    }

    fn test_parse_os_release(container_runner: &str, image_name: &str) {
        let (stdout, _stderr) =
            run_container(container_runner, "", image_name, "cat /etc/os-release").unwrap();
        let parsed = parse_os_release(&stdout);
        assert!(
            parsed.is_some(),
            "Failed to parse OS release info for image \"{}\"",
            image_name
        );
    }
}
