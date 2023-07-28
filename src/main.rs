mod config;
mod container_tree;
mod distro;
mod distrobox_config_converter;
mod distrobox_parser;
mod oci;
mod utils;

use crate::config::*;
use crate::distrobox_config_converter::build_distrobox_assemble_data;
use crate::distrobox_parser::assemble::{assemble_distrobox_to_str, parse_distrobox_assemble};

fn build(assemble_file_content: &str, extra_packages: Option<String>) -> String {
    let mut distrobox_assemble_data = parse_distrobox_assemble(assemble_file_content);
    for value in distrobox_assemble_data.values_mut() {
        if let Some(ref pkgs) = extra_packages {
            let packages = value.packages.get_or_insert(Vec::new());
            packages.push(pkgs.clone());
        }
    }
    let container_manager = get_container_manager();
    let new_distrobox_assemble_data =
        build_distrobox_assemble_data(&container_manager, &distrobox_assemble_data);
    assemble_distrobox_to_str(&new_distrobox_assemble_data)
}

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(short, long)]
    output: Option<String>,

    #[clap(short, long)]
    ext_pkg: Option<String>,

    #[clap(short, long)]
    non_distrobox: Option<bool>,
}

fn main() {
    let args = Args::parse();
    set_distrobox_mode(match args.non_distrobox {
        Some(true) => false,
        _ => true,
    });

    let content = std::fs::read_to_string(&args.input).unwrap();

    let result = build(&content, args.ext_pkg);

    match args.output {
        Some(path) => std::fs::write(&path, result).unwrap(),
        None => println!("{}", result),
    }
}
