use std::collections::HashMap;
use std::process::Command;

fn extract_environment_variables(
    script: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(script.to_string() + "\nenv")
        .output()?;

    if !output.status.success() {
        return Err("Script exited with non-zero status".into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();

    let mut variables = HashMap::new();
    for line in lines {
        let split: Vec<&str> = line.splitn(2, '=').collect();
        if split.len() == 2 {
            let key = split[0].to_string();
            let value = split[1].to_string();
            variables.insert(key, value);
        }
    }

    Ok(variables)
}

pub fn get_distrobox_config() -> HashMap<String, String> {
    // From https://github.com/89luca89/distrobox/blob/main/distrobox-create
    let script = r#"
# Defaults
export container_additional_packages=""
export container_always_pull=0
export container_clone=""
export container_generate_entry=1
export container_home_prefix=""
export container_image=""
export container_image_default="registry.fedoraproject.org/fedora-toolbox:38"
export container_init_hook=""
export container_manager="autodetect"
export container_manager_additional_flags=""
export container_name=""
export container_name_default="my-distrobox"
export container_pre_init_hook=""
export container_user_custom_home=""
export container_user_gid="$(id -rg)"
export container_user_home="${HOME:-"/"}"
export container_user_name="${USER}"
export container_user_uid="$(id -ru)"
export dryrun=0
export init=0
export non_interactive=0
export nvidia=0
export unshare_ipc=0
export unshare_netns=0"#
        .to_string()
        + r#"
# Use cd + dirname + pwd so that we do not have relative paths in mount points
# We're not using "realpath" here so that symlinks are not resolved this way
# "realpath" would break situations like Nix or similar symlink based package
# management.
distrobox_entrypoint_path="$(cd "$(dirname "${0}")" && pwd)/distrobox-init"
distrobox_export_path="$(cd "$(dirname "${0}")" && pwd)/distrobox-export"
distrobox_genentry_path="$(cd "$(dirname "${0}")" && pwd)/distrobox-generate-entry"
distrobox_hostexec_path="$(cd "$(dirname "${0}")" && pwd)/distrobox-host-exec"
# In case some of the scripts are not in the same path as create, let's search
# in PATH for them.
[ ! -e "${distrobox_entrypoint_path}" ] && distrobox_entrypoint_path="$(command -v distrobox-init)"
[ ! -e "${distrobox_export_path}" ] && distrobox_export_path="$(command -v distrobox-export)"
[ ! -e "${distrobox_genentry_path}" ] && distrobox_genentry_path="$(command -v distrobox-generate-entry)"
[ ! -e "${distrobox_hostexec_path}" ] && distrobox_hostexec_path="$(command -v distrobox-host-exec)"
# If the user runs this script as root in a login shell, set rootful=1.
# There's no need for them to pass the --root flag option in such cases.
[ "${container_user_uid}" -eq 0 ] && rootful=1 || rootful=0
verbose=0
version="1.5.0.2"

# Source configuration files, this is done in an hierarchy so local files have
# priority over system defaults
# leave priority to environment variables.
config_files="
	/usr/share/distrobox/distrobox.conf
	/usr/share/defaults/distrobox/distrobox.conf
	/usr/etc/distrobox/distrobox.conf
	/usr/local/share/distrobox/distrobox.conf
	/etc/distrobox/distrobox.conf
	${XDG_CONFIG_HOME:-"${HOME}/.config"}/distrobox/distrobox.conf
	${HOME}/.distroboxrc
"
for config_file in ${config_files}; do
	# Shellcheck will give error for sourcing a variable file as it cannot follow
	# it. We don't care so let's disable this linting for now.
	# shellcheck disable=SC1090
	[ -e "${config_file}" ] && . "${config_file}"
done
# If we're running this script as root -- as in logged in in the shell as root
# user, and not via SUDO/DOAS --, we don't need to set distrobox_sudo_program
# as it's meaningless for this use case.
if [ "${container_user_uid}" -ne 0 ]; then
	# If the DBX_SUDO_PROGRAM/distrobox_sudo_program variable was set by the
	# user, use its value instead of "sudo". But only if not running the script
	# as root (UID 0).
	distrobox_sudo_program=${DBX_SUDO_PROGRAM:-${distrobox_sudo_program:-"sudo"}}
fi
# Fixup non_interactive=[true|false], in case we find it in the config file(s)
[ "${non_interactive}" = "true" ] && non_interactive=1
[ "${non_interactive}" = "false" ] && non_interactive=0

[ -n "${DBX_CONTAINER_ALWAYS_PULL}" ] && container_always_pull="${DBX_CONTAINER_ALWAYS_PULL}"
[ -n "${DBX_CONTAINER_CUSTOM_HOME}" ] && container_user_custom_home="${DBX_CONTAINER_CUSTOM_HOME}"
[ -n "${DBX_CONTAINER_HOME_PREFIX}" ] && container_home_prefix="${DBX_CONTAINER_HOME_PREFIX}"
[ -n "${DBX_CONTAINER_IMAGE}" ] && container_image="${DBX_CONTAINER_IMAGE}"
[ -n "${DBX_CONTAINER_MANAGER}" ] && container_manager="${DBX_CONTAINER_MANAGER}"
[ -n "${DBX_CONTAINER_NAME}" ] && container_name="${DBX_CONTAINER_NAME}"
[ -n "${DBX_NON_INTERACTIVE}" ] && non_interactive="${DBX_NON_INTERACTIVE}"
[ -n "${DBX_container_generate_entry}" ] && container_generate_entry="${DBX_container_generate_entry}"
"#;
    extract_environment_variables(&script).unwrap_or_else(|_| HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_environment_variables() {
        let script = r#"
export TEST_VAR1="Hello"
export TEST_VAR2='World'
export TEST_VAR3=!"#;

        let result = extract_environment_variables(script).unwrap();
        assert_eq!(result.get("TEST_VAR1"), Some(&"Hello".to_string()));
        assert_eq!(result.get("TEST_VAR2"), Some(&"World".to_string()));
        assert_eq!(result.get("TEST_VAR3"), Some(&"!".to_string()));
    }

    #[test]
    fn test_get_distrobox_config() {
        let config = get_distrobox_config();

        assert!(config.contains_key("container_additional_packages"));
        assert!(config.contains_key("container_always_pull"));
        assert!(config.contains_key("container_clone"));
        assert!(config.contains_key("container_generate_entry"));
        assert!(config.contains_key("container_home_prefix"));
        assert!(config.contains_key("container_image"));
        assert!(config.contains_key("container_image_default"));
        assert!(config.contains_key("container_init_hook"));
        assert!(config.contains_key("container_manager"));
        assert!(config.contains_key("container_manager_additional_flags"));
        assert!(config.contains_key("container_name"));
        assert!(config.contains_key("container_name_default"));
        assert!(config.contains_key("container_pre_init_hook"));
        assert!(config.contains_key("container_user_custom_home"));
        assert!(config.contains_key("container_user_gid"));
        assert!(config.contains_key("container_user_home"));
        assert!(config.contains_key("container_user_name"));
        assert!(config.contains_key("container_user_uid"));
        assert!(config.contains_key("dryrun"));
        assert!(config.contains_key("init"));
        assert!(config.contains_key("non_interactive"));
        assert!(config.contains_key("nvidia"));
        assert!(config.contains_key("unshare_ipc"));
        assert!(config.contains_key("unshare_netns"));
    }
}
