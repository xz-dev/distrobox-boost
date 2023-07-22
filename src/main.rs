mod container_tree;
mod distro;
mod distrobox_config_converter;
mod distrobox_parser;
mod ini_utils;
mod oci;

use crate::distrobox_config_converter::build_distrobox_assemble_data;
use crate::distrobox_parser::assemble::{assemble_distrobox_to_str, parse_distrobox_assemble};

fn build(assemble_file_content: &str) -> String {
    let distrobox_assemble_data = parse_distrobox_assemble(assemble_file_content);
    let new_distrobox_assemble_data =
        build_distrobox_assemble_data("podman", &distrobox_assemble_data);
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
}

fn main() {
    let args = Args::parse();

    let content = std::fs::read_to_string(&args.input).unwrap();

    let result = build(&content);

    match args.output {
        Some(path) => std::fs::write(&path, result).unwrap(),
        None => println!("{}", result),
    }
}
