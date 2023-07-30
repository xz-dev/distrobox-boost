mod config;
mod container_tree;
mod distro;
mod distrobox_config_converter;
mod distrobox_parser;
mod oci;
mod utils;

use std::collections::HashMap;

use crate::config::*;
use crate::distrobox_config_converter::build_distrobox_assemble_data;
use crate::distrobox_parser::assemble::{
    assemble_distrobox_to_str, parse_distrobox_assemble, ContainerAssembleData,
};
use crate::oci::command_helper::{pin_image, unpin_image};
use clap::Parser;

fn build(
    distrobox_assemble_data: &HashMap<String, ContainerAssembleData>,
    extra_packages: Option<String>,
) -> HashMap<String, ContainerAssembleData> {
    let mut distrobox_assemble_data = distrobox_assemble_data.clone();
    for value in distrobox_assemble_data.values_mut() {
        if let Some(ref pkgs) = extra_packages {
            let packages = value.packages.get_or_insert(Vec::new());
            packages.push(pkgs.clone());
        }
    }
    let new_distrobox_assemble_data =
        build_distrobox_assemble_data(&get_container_manager(), &distrobox_assemble_data);
    new_distrobox_assemble_data
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(short, long)]
    output: Option<String>,

    #[clap(short, long)]
    output_dir: Option<String>,

    #[clap(short, long)]
    ext_pkg: Option<String>,

    #[clap(short, long)]
    non_distrobox: bool,

    #[clap(short, long)]
    pin: bool,
    #[clap(short, long)]
    unpin: bool,
}

fn main() {
    let args = Args::parse();
    if args.non_distrobox {
        set_distrobox_mode(false);
        println!("Non distrobox mode");
    }

    let content = std::fs::read_to_string(&args.input).unwrap();

    let distrobox_assemble_data = parse_distrobox_assemble(&content);
    let new_distrobox_assemble_data = build(&distrobox_assemble_data, args.ext_pkg);
    let file_content = assemble_distrobox_to_str(&new_distrobox_assemble_data);

    if args.output.is_none() && args.output_dir.is_none() {
        println!("{}", file_content);
        return;
    }
    if let Some(output) = args.output {
        std::fs::write(&output, &file_content).unwrap();
    }
    if let Some(output_dir) = args.output_dir {
        let output_dir_path = std::path::PathBuf::from(output_dir);
        for name in new_distrobox_assemble_data.keys() {
            let mut output_path = output_dir_path.clone();
            output_path.push(name);
            output_path.set_extension("ini");
            let mut data = HashMap::new();
            data.insert(name.clone(), new_distrobox_assemble_data.get(name).unwrap().clone());
            let file_content = assemble_distrobox_to_str(&data);
            std::fs::write(&output_path, &file_content).unwrap();
        }
    }

    if args.pin {
        for data in new_distrobox_assemble_data.values() {
            let result = pin_image(&get_container_manager(), &data.image);
            if let Err(e) = result {
                println!("Pin image {} failed: {}", &data.image, e);
            }
        }
    }
    if args.unpin {
        for data in new_distrobox_assemble_data.values() {
            let result = unpin_image(&get_container_manager(), &data.image);
            if let Err(e) = result {
                println!("Unpin image {} failed: {}", &data.image, e);
            }
        }
    }
}
