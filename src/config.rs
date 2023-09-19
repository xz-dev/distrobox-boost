use crate::distrobox::parser::config::get_distrobox_config;
use lazy_static::lazy_static;
use std::sync::Mutex;

use std::process::Command;

lazy_static! {
    static ref DISTROBOX_MODE: Mutex<bool> = Mutex::new(true);
    static ref DISTROBOX_BOOST_IMAGE_PREFIX: Mutex<String> =
        Mutex::new("distrobox-boost".to_string());
}

pub fn get_distrobox_mode() -> bool {
    DISTROBOX_MODE.lock().unwrap().clone()
}

pub fn set_distrobox_mode(mode: bool) {
    *DISTROBOX_MODE.lock().unwrap() = mode;
}

pub fn get_distrobox_boost_image_prefix() -> String {
    DISTROBOX_BOOST_IMAGE_PREFIX.lock().unwrap().clone()
}

pub fn set_distrobox_boost_image_prefix(image_prefix: &str) {
    *DISTROBOX_BOOST_IMAGE_PREFIX.lock().unwrap() = image_prefix.to_string();
}

pub fn get_distrobox_boost_test_image_prefix() -> String {
    format!("{}-test", &get_distrobox_boost_image_prefix())
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
