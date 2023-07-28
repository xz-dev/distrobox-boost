use crate::distrobox_parser::config::get_distrobox_config;

use std::process::Command;

static mut DISTROBOX_MODE: bool = true;

pub fn get_distrobox_mode() -> bool {
    unsafe { DISTROBOX_MODE }
}

pub fn set_distrobox_mode(mode: bool) {
    unsafe {
        DISTROBOX_MODE = mode;
    }
}

fn command_exists(command: &str) -> bool {
    let os = std::env::consts::OS;
    let output = if os == "windows" {
        Command::new("cmd")
            .args(&["/C", "where", command])
            .output()
            .expect("Failed to execute command")
    } else {
        Command::new("sh")
            .args(&["-c", &format!("command -v {}", command)])
            .output()
            .expect("Failed to execute command")
    };
    output.status.success()
}

pub fn get_container_manager() -> String {
    let config = get_distrobox_config();
    fn autodetect() -> String {
        if command_exists("podman") {
            "podman".to_string()
        } else if command_exists("docker") {
            "docker".to_string()
        } else {
            panic!("No container manager found")
        }
    }
    match config.get("container_manager") {
        None => autodetect(),
        Some(container_manager) => match container_manager.as_str() {
            "autodetect" => autodetect(),
            _ => container_manager.clone(),
        },
    }
}
