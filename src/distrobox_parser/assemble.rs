use crate::distrobox_parser::config::get_distrobox_config;
use crate::utils::ini::{from_ini, merge_ini, to_ini};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ContainerAssembleData {
    pub flags: Option<Vec<String>>,
    pub packages: Option<Vec<String>>,
    pub home: Option<String>,
    pub image: String,
    pub init_hooks: Option<Vec<String>>,
    pub pre_init_hooks: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub entry: Option<bool>,
    pub start_now: Option<bool>,
    pub init: Option<bool>,
    pub nvidia: Option<bool>,
    pub pull: Option<bool>,
    pub root: Option<bool>,
    pub unshare_ipc: Option<bool>,
    pub unshare_netns: Option<bool>,

    // extra fields for the tool
    pub package_manager: Option<String>,
}
impl Default for ContainerAssembleData {
    fn default() -> Self {
        Self {
            flags: None,
            packages: None,
            home: None,
            image: String::new(),
            init_hooks: None,
            pre_init_hooks: None,
            volumes: None,
            entry: None,
            start_now: None,
            init: None,
            nvidia: None,
            pull: None,
            root: None,
            unshare_ipc: None,
            unshare_netns: None,
            package_manager: None,
        }
    }
}

pub fn parse_distrobox_assemble(content: &str) -> HashMap<String, ContainerAssembleData> {
    let parsed = from_ini(content);
    let merged = merge_ini(parsed);
    let config = get_distrobox_config();
    println!("config: {:?}", config);
    let default_image = config
        .get("container_image_default")
        .expect("No default image found");

    merged
        .into_iter()
        .map(|(name, entry)| {
            (
                name,
                ContainerAssembleData {
                    flags: entry.get("flags").map(|i| i.clone()),
                    packages: entry.get("packages").map(|i| {
                        i.iter()
                            .flat_map(|pkg_str| pkg_str.split_whitespace())
                            .map(|pkg| pkg.to_string())
                            .collect::<Vec<String>>()
                    }),
                    home: entry.get("home").map(|h| h.join(" ")),
                    image: entry
                        .get("image")
                        .map(|i| i.join(" "))
                        .unwrap_or(default_image.clone()),
                    init_hooks: entry.get("init_hooks").map(|i| i.clone()),
                    pre_init_hooks: entry.get("pre_init_hooks").map(|i| i.clone()),
                    volumes: entry.get("volumes").map(|i| {
                        i.iter()
                            .map(|volume_str| {
                                let mut split = volume_str.split(':');
                                let key = split.next().unwrap_or("").to_string();
                                let value = split.next().unwrap_or("").to_string();
                                (key, value)
                            })
                            .collect()
                    }),
                    entry: get_value_as_bool_with_default(&entry, "entry"),
                    start_now: get_value_as_bool_with_default(&entry, "start_now"),
                    init: get_value_as_bool_with_default(&entry, "init"),
                    nvidia: get_value_as_bool_with_default(&entry, "nvidia"),
                    pull: get_value_as_bool_with_default(&entry, "pull"),
                    root: get_value_as_bool_with_default(&entry, "root"),
                    unshare_ipc: get_value_as_bool_with_default(&entry, "unshare_ipc"),
                    unshare_netns: get_value_as_bool_with_default(&entry, "unshare_netns"),
                    package_manager: entry.get("package_manager").map(|h| h.join(" ")),
                },
            )
        })
        .collect()
}

pub fn assemble_distrobox_to_str(data: &HashMap<String, ContainerAssembleData>) -> String {
    let mut ini_data = Vec::new();

    for (name, assemble_data) in data {
        let mut single_ini_data = Vec::new();
        if let Some(flags) = &assemble_data.flags {
            single_ini_data.push(("flags".to_string(), flags.join(" ")));
        }
        if let Some(packages) = &assemble_data.packages {
            if !packages.is_empty() {
                single_ini_data.push(("packages".to_string(), packages.join(" ")));
            }
        }
        if let Some(home) = &assemble_data.home {
            single_ini_data.push(("home".to_string(), home.clone()));
        }
        single_ini_data.push(("image".to_string(), assemble_data.image.clone()));
        if let Some(init_hooks) = &assemble_data.init_hooks {
            for init_hook in init_hooks {
                single_ini_data.push(("init_hooks".to_string(), init_hook.clone()));
            }
        }

        if let Some(pre_init_hooks) = &assemble_data.pre_init_hooks {
            for pre_init_hook in pre_init_hooks {
                single_ini_data.push(("pre_init_hooks".to_string(), pre_init_hook.clone()));
            }
        }

        if let Some(volumes) = &assemble_data.volumes {
            let volume_str = volumes
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            single_ini_data.push(("volumes".to_string(), volume_str));
        }
        if let Some(entry) = assemble_data.entry {
            single_ini_data.push(("entry".to_string(), entry.to_string()));
        }
        if let Some(start_now) = assemble_data.start_now {
            single_ini_data.push(("start_now".to_string(), start_now.to_string()));
        }
        if let Some(init) = assemble_data.init {
            single_ini_data.push(("init".to_string(), init.to_string()));
        }
        if let Some(nvidia) = assemble_data.nvidia {
            single_ini_data.push(("nvidia".to_string(), nvidia.to_string()));
        }
        if let Some(pull) = assemble_data.pull {
            single_ini_data.push(("pull".to_string(), pull.to_string()));
        }
        if let Some(root) = assemble_data.root {
            single_ini_data.push(("root".to_string(), root.to_string()));
        }
        if let Some(unshare_ipc) = assemble_data.unshare_ipc {
            single_ini_data.push(("unshare_ipc".to_string(), unshare_ipc.to_string()));
        }
        if let Some(unshare_netns) = assemble_data.unshare_netns {
            single_ini_data.push(("unshare_netns".to_string(), unshare_netns.to_string()));
        }

        ini_data.push((name.clone(), single_ini_data));
    }
    to_ini(&ini_data)
}

fn get_value_as_bool_with_default(map: &HashMap<String, Vec<String>>, key: &str) -> Option<bool> {
    map.get(key)
        .and_then(|value| value.first()?.parse::<bool>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_distrobox_assemble_single_section() {
        let content = r#"
[test_section]
flags="--net host"
packages="vim curl"
home=/home/test_user
image=docker.io/library/ubuntu:20.04
init_hooks=hook1
init_hooks=hook2
pre_init_hooks=pre_hook1
pre_init_hooks=pre_hook2
volumes=volume1:/mnt/volume1
volumes=volume2:/mnt/volume2
entry=true
start_now=false
init=true
nvidia=false
pull=true
root=true
unshare_ipc=true
unshare_netns=false
"#;

        let result = parse_distrobox_assemble(content);

        assert_eq!(result.len(), 1);
        assert!(result.contains_key("test_section"));

        let entry = &result["test_section"];
        assert_eq!(entry.flags.as_ref().unwrap(), &["--net host"]);
        assert_eq!(entry.packages.as_ref().unwrap(), &["vim", "curl"]);
        assert_eq!(entry.home.as_ref().unwrap(), "/home/test_user");
        assert_eq!(entry.image, "docker.io/library/ubuntu:20.04");
        assert_eq!(entry.init_hooks.as_ref().unwrap(), &["hook1", "hook2"]);
        assert_eq!(
            entry.pre_init_hooks.as_ref().unwrap(),
            &["pre_hook1", "pre_hook2"]
        );

        let expected_volumes: HashMap<String, String> = vec![
            ("volume1".to_string(), "/mnt/volume1".to_string()),
            ("volume2".to_string(), "/mnt/volume2".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(entry.volumes.as_ref().unwrap(), &expected_volumes);

        assert_eq!(entry.entry.unwrap(), true);
        assert_eq!(entry.start_now.unwrap(), false);
        assert_eq!(entry.init.unwrap(), true);
        assert_eq!(entry.nvidia.unwrap(), false);
        assert_eq!(entry.pull.unwrap(), true);
        assert_eq!(entry.root.unwrap(), true);
        assert_eq!(entry.unshare_ipc.unwrap(), true);
        assert_eq!(entry.unshare_netns.unwrap(), false);
    }

    #[test]
    fn test_parse_distrobox_assemble_multiple_sections() {
        let content = r#"
[section1]
flags=--net host
home=/home/user1
image=docker.io/library/ubuntu:20.04

[section2]
flags="--net" "bridge"
home=/home/user2
image=docker.io/library/debian:10
"#;

        let result = parse_distrobox_assemble(content);

        assert_eq!(result.len(), 2);
        assert!(result.contains_key("section1"));
        assert!(result.contains_key("section2"));

        let entry1 = &result["section1"];
        assert_eq!(entry1.flags.as_ref().unwrap(), &["--net host"]);
        assert_eq!(entry1.home.as_ref().unwrap(), "/home/user1");
        assert_eq!(entry1.image, "docker.io/library/ubuntu:20.04");

        let entry2 = &result["section2"];
        assert_eq!(entry2.flags.as_ref().unwrap(), &["\"--net\" \"bridge\""]);
        assert_eq!(entry2.home.as_ref().unwrap(), "/home/user2");
        assert_eq!(entry2.image, "docker.io/library/debian:10");
    }

    #[test]
    fn test_parse_distrobox_assemble_missing_values() {
        let content = r#"
[test_section]
flags=--net host
packages=vim curl
packages="nano wget"
home=/home/test_user
"#;

        let result = parse_distrobox_assemble(content);

        assert_eq!(result.len(), 1);
        assert!(result.contains_key("test_section"));

        let entry = &result["test_section"];
        assert_eq!(entry.flags.as_ref().unwrap(), &["--net host"]);
        assert_eq!(
            entry.packages.as_ref().unwrap(),
            &["vim", "curl", "nano", "wget"]
        );
        assert_eq!(entry.home.as_ref().unwrap(), "/home/test_user");
        assert!(!entry.image.is_empty());
    }

    #[test]
    fn test_parse_distrobox_assemble_empty_input() {
        let content = "";

        let result = parse_distrobox_assemble(content);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_distrobox_assemble_whitespace_only_input() {
        let content = "       \n   \t   ";

        let result = parse_distrobox_assemble(content);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_assemble_distrobox_to_str_normal() {
        let mut data = HashMap::new();
        data.insert("foo".to_string(), ContainerAssembleData::default());

        let result = assemble_distrobox_to_str(&data);

        assert!(result.contains("[foo]"));
    }

    #[test]
    fn test_assemble_distrobox_to_str_with_fields() {
        let mut data = HashMap::new();

        let mut foo = ContainerAssembleData::default();
        foo.flags = Some(vec!["--net".to_string()]);
        foo.packages = Some(vec!["vim".to_string()]);

        data.insert("foo".to_string(), foo);

        let result = assemble_distrobox_to_str(&data);

        assert!(result.contains("[foo]"));
        assert!(result.contains("flags=--net"));
        assert!(result.contains("packages=vim"));
    }
    #[test]
    fn test_assemble_distrobox_to_str_multiple_sections() {
        let mut data = HashMap::new();

        data.insert("foo".to_string(), ContainerAssembleData::default());
        data.insert("bar".to_string(), ContainerAssembleData::default());

        let result = assemble_distrobox_to_str(&data);

        assert!(result.contains("[foo]"));
        assert!(result.contains("[bar]"));
    }

    #[test]
    fn test_assemble_distrobox_to_str_with_pre_init_hooks() {
        let mut data = HashMap::new();

        let mut foo = ContainerAssembleData::default();
        foo.pre_init_hooks = Some(vec![
            r#"do something 'funny' here"#.to_string(),
            r#"echo 'haha'"#.to_string(),
        ]);

        data.insert("foo".to_string(), foo);

        let result = assemble_distrobox_to_str(&data);

        assert!(result.contains("[foo]"));
        assert!(result.contains(r#"pre_init_hooks=do something 'funny' here"#));
        assert!(result.contains(r#"pre_init_hooks=echo 'haha'"#));
    }

    #[test]
    fn test_assemble_distrobox_to_str_empty() {
        let data: HashMap<String, ContainerAssembleData> = HashMap::new();

        let result = assemble_distrobox_to_str(&data);

        assert_eq!(result, "");
    }
}
