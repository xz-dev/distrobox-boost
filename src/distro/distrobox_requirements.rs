static PACKAGES_MAP: &[(&'static str, &'static [&'static str])] = &[
    (
        "alpine",
        &[
            "bc",
            "curl",
            "diffutils",
            "findmnt",
            "findutils",
            "gnupg",
            "less",
            "lsof",
            "mount",
            "umount",
            "ncurses",
            "pinentry",
            "posix-libc-utils",
            "procps",
            "shadow",
            "su-exec",
            "sudo",
            "util-linux",
            "util-linux-misc",
            "vte3",
            "wget",
            "$(apk search -q mesa-dri)",
            "$(apk search -q mesa-vulkan)",
            "vulkan-loader",
        ],
    ),
    (
        "arch",
        &[
            "bc",
            "curl",
            "diffutils",
            "findutils",
            "gnupg",
            "less",
            "lsof",
            "ncurses",
            "pinentry",
            "procps-ng",
            "shadow",
            "sudo",
            "time",
            "util-linux",
            "wget",
            "mesa",
            "opengl-driver",
            "vulkan-intel",
            "vte-common",
            "vulkan-radeon",
        ],
    ),
    (
        "centos",
        &[
            "bc",
            "curl",
            "diffutils",
            "findutils",
            "gnupg2",
            "less",
            "lsof",
            "ncurses",
            "pam",
            "passwd",
            "pinentry",
            "procps-ng",
            "shadow-utils",
            "sudo",
            "time",
            "tzdata",
            "util-linux",
            "vte-profile",
            "wget",
            "mesa-dri-drivers",
            "mesa-vulkan-drivers",
            "vulkan",
        ],
    ),
    (
        "debian",
        &[
            "apt-utils",
            "bc",
            "curl",
            "dialog",
            "diffutils",
            "findutils",
            "gnupg2",
            "less",
            "libnss-myhostname",
            "libvte-2.9*-common",
            "libvte-common",
            "lsof",
            "ncurses-base",
            "passwd",
            "pinentry-curses",
            "procps",
            "sudo",
            "time",
            "util-linux",
            "wget",
            "libegl1-mesa",
            "libgl1-mesa-glx",
            "libvulkan1",
            "mesa-vulkan-drivers",
        ],
    ),
    (
        "fedora",
        &[
            "bc",
            "curl",
            "diffutils",
            "dnf-plugins-core",
            "findutils",
            "gnupg2",
            "less",
            "lsof",
            "ncurses",
            "pam",
            "passwd",
            "pinentry",
            "procps-ng",
            "shadow-utils",
            "sudo",
            "time",
            "tzdata",
            "util-linux",
            "vte-profile",
            "wget",
            "mesa-dri-drivers",
            "mesa-vulkan-drivers",
            "vulkan",
        ],
    ),
    (
        "opensuse",
        &[
            "bc",
            "curl",
            "diffutils",
            "findutils",
            "gnupg",
            "less",
            "libvte-2*",
            "lsof",
            "ncurses",
            "pam",
            "pam-extra",
            "pinentry",
            "procps",
            "shadow",
            "sudo",
            "systemd",
            "time",
            "util-linux",
            "util-linux-systemd",
            "wget",
            "Mesa-dri",
            "libvulkan1",
            "libvulkan_intel",
            "libvulkan_radeon",
        ],
    ),
    (
        "rocky",
        &[
            "bc",
            "curl",
            "diffutils",
            "findutils",
            "gnupg2",
            "less",
            "lsof",
            "ncurses",
            "pam",
            "passwd",
            "pinentry",
            "procps-ng",
            "shadow-utils",
            "sudo",
            "time",
            "tzdata",
            "util-linux",
            "vte-profile",
            "wget",
            "mesa-dri-drivers",
            "mesa-vulkan-drivers",
            "vulkan",
        ],
    ),
    (
        "ubuntu",
        &[
            "apt-utils",
            "bc",
            "curl",
            "dialog",
            "diffutils",
            "findutils",
            "gnupg2",
            "less",
            "libnss-myhostname",
            "libvte-2.9*-common",
            "libvte-common",
            "lsof",
            "ncurses-base",
            "passwd",
            "pinentry-curses",
            "procps",
            "sudo",
            "time",
            "util-linux",
            "wget",
            "libegl1-mesa",
            "libgl1-mesa-glx",
            "libvulkan1",
            "mesa-vulkan-drivers",
        ],
    ),
];

pub fn get_distrobox_packages(distro_id: &str) -> Vec<&str> {
    let mut packages = Vec::new();
    for (distro, pkgs) in PACKAGES_MAP.iter() {
        if distro_id == *distro {
            packages = pkgs.to_vec();
        }
    }
    packages
}

#[cfg(test)]
mod tests {
    use crate::config::{get_container_manager, get_distrobox_mode, set_distrobox_mode};
    use crate::oci::command_helper::run_container;
    use crate::oci::image_builder::build_image;

    fn test_distrobox_packages(distro_id: &str, base_image: &str) {
        let container_runner = &get_container_manager();
        let image_name = format!("test_{}-distrobox", distro_id);
        let tmp_distrobox_mode = get_distrobox_mode();
        set_distrobox_mode(true);

        let result = build_image(container_runner, &image_name, &base_image, &vec![]);
        set_distrobox_mode(tmp_distrobox_mode);

        let image_name = match result {
            Ok(image_name) => {
                println!("Final image name: {}", image_name);
                image_name
            }
            Err(e) => {
                println!("Error building image: {:?}", e);
                assert!(false, "Error building image");
                return;
            }
        };

        let cmds = vec![
            "find", "mount", "passwd", "sudo", "useradd", "diff", "pinentry", "wget", "curl",
            "less", "bc", "time", "lsof",
        ];

        for cmd in cmds {
            let check_cmd = format!("command -v {}", cmd);

            match run_container(container_runner, "", &image_name, &check_cmd) {
                Ok((stdout, _stderr)) => assert!(!stdout.is_empty()),
                Err(e) => panic!("Error getting OS info from image {}: {:?}", image_name, e),
            }
        }
    }
    #[test]
    fn test_build_distrobox_for_alpine() {
        test_distrobox_packages("alpine", "alpine");
    }

    #[test]
    fn test_build_distrobox_for_arch() {
        test_distrobox_packages("arch", "archlinux");
    }

    #[test]
    fn test_build_distrobox_for_centos() {
        test_distrobox_packages("centos", "centos:centos7");
    }

    #[test]
    fn test_build_distrobox_for_debian() {
        test_distrobox_packages("debian", "debian");
    }

    #[test]
    fn test_build_distrobox_for_fedora() {
        test_distrobox_packages("fedora", "registry.fedoraproject.org/fedora-toolbox:latest");
    }

    #[test]
    fn test_build_distrobox_for_opensuse() {
        test_distrobox_packages(
            "opensuse",
            "registry.opensuse.org/opensuse/distrobox:latest",
        );
    }

    #[test]
    fn test_build_distrobox_for_rocky() {
        test_distrobox_packages("rocky", "rockylinux:9");
    }

    #[test]
    fn test_build_distrobox_for_ubuntu() {
        test_distrobox_packages("ubuntu", "ubuntu");
    }
}
