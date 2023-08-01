mod config;
mod container_tree;
mod distro;
mod distrobox;
mod distrobox_config_converter;
mod oci;
mod utils;

use std::collections::HashMap;

use crate::config::*;
use crate::distrobox::command_helper::*;
use crate::distrobox::parser::assemble::{
    assemble_distrobox_to_str, parse_distrobox_assemble, ContainerAssembleData,
};
use crate::distrobox_config_converter::build_distrobox_assemble_data;
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
    #[clap(index = 1)]
    package: Option<String>,
    #[clap(
        index = 2,
        allow_hyphen_values = true,
        value_terminator = ";",
        required_unless_present = "package"
    )]
    package_params: Option<Vec<String>>,

    #[clap(short, long, allow_hyphen_values = true, value_terminator = ";")]
    assemble_arg: Option<Vec<String>>,

    #[clap(short, long)]
    input: Option<String>,

    #[clap(short, long)]
    output: Option<String>,

    #[clap(short, long)]
    output_dir: Option<String>,

    #[clap(short, long)]
    pkg: Option<String>,

    #[clap(short, long)]
    non_distrobox: bool,

    #[clap(short, long)]
    pin: bool,
    #[clap(short, long)]
    unpin: bool,

    #[clap(long, allow_hyphen_values = true, value_terminator = ";")]
    enter_arg: Option<Vec<String>>,

    #[clap(long)]
    no_run: bool,

    #[arg(num_args(0..))]
    #[clap(short, long, allow_hyphen_values = true, value_terminator = ";")]
    run: Option<Vec<String>>,
}

fn main() {
    let args = Args::parse();
    if args.package.is_none() && args.input.is_none() && args.assemble_arg.is_none() {
        println!("Use --help to get help");
        return
    }
    println!("assblemble args: {:?}", args.assemble_arg);

    if args.non_distrobox {
        set_distrobox_mode(false);
        println!("Non distrobox mode");
    }

    let mut distrobox_assemble_data_map = HashMap::new();
    if let Some(ref package) = args.package {
        let mut assemble_data = HashMap::new();
        assemble_data.insert(
            package.clone(),
            ContainerAssembleData {
                packages: Some(vec![package.to_string()]),
                ..Default::default()
            },
        );
        let mut assemble_content = assemble_distrobox_to_str(&assemble_data);
        if let Some(ref assemple_data) = args.assemble_arg {
            assemble_content.push_str(
                &assemple_data
                    .iter()
                    .filter(|line| !line.starts_with("-"))
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        distrobox_assemble_data_map.extend(parse_distrobox_assemble(&assemble_content));
    }

    if let Some(input) = args.input {
        let content = std::fs::read_to_string(&input).unwrap();
        distrobox_assemble_data_map.extend(parse_distrobox_assemble(&content));
    }

    let new_distrobox_assemble_data = build(&distrobox_assemble_data_map, args.pkg);
    let file_content = assemble_distrobox_to_str(&new_distrobox_assemble_data);

    if args.no_run {
        if args.output.is_none() && args.output_dir.is_none() {
            println!("{}", file_content);
            return;
        }
    }

    let mut file_path_map = HashMap::new();

    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &file_content).unwrap();
        for name in new_distrobox_assemble_data.keys() {
            file_path_map.insert(name.to_string(), output_path.clone());
        }
    }
    if let Some(output_dir) = args.output_dir {
        let output_dir_path = std::path::PathBuf::from(output_dir);
        for name in new_distrobox_assemble_data.keys() {
            let mut output_path = output_dir_path.clone();
            output_path.push(name);
            output_path.set_extension("ini");
            let mut data = HashMap::new();
            data.insert(
                name.clone(),
                new_distrobox_assemble_data.get(name).unwrap().clone(),
            );
            let file_content = assemble_distrobox_to_str(&data);
            std::fs::write(&output_path, &file_content).unwrap();
            file_path_map.insert(name.to_string(), output_path.to_str().unwrap().to_string());
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

    if !args.no_run {
        if let Some(ref package) = args.package {
            let mut tmp_assemble_file: Option<String> = None;
            let assemble_file_path = if let Some(path) = file_path_map.get(package) {
                path.clone()
            } else {
                let assemble_file_path = format!(".distrobox_assemble_{}.tmp.ini", package);
                tmp_assemble_file = Some(assemble_file_path.clone());
                std::fs::write(&assemble_file_path, &file_content).unwrap();
                assemble_file_path
            };
            let assemble_args = if let Some(ref assemble_args) = args.assemble_arg {
                let assemble_args = &assemble_args
                    .iter()
                    .filter(|line| line.starts_with("-"))
                    .cloned()
                    .collect::<Vec<_>>();
                if !assemble_args.contains(&"rm".to_string())
                    && !assemble_args.contains(&"create".to_string())
                {
                    assemble_args.clone()
                } else {
                    let mut args = vec!["create".to_string()];
                    args.extend(assemble_args.clone());
                    args
                }
            } else {
                vec!["create".to_string()]
            };

            let (stdout, stderr) = distrobox_assemble(
                &assemble_file_path,
                &assemble_args.first().unwrap(),
                &assemble_args[1..]
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>(),
            )
            .unwrap();
            println!("{}", stdout);
            if !stderr.is_empty() {
                println!("Error: {}", stderr);
            }
            if let Some(tmp_assemble_file) = tmp_assemble_file {
                std::fs::remove_file(&tmp_assemble_file).unwrap();
            }

            let enter_args = if let Some(enter_args) = args.enter_arg {
                enter_args
            } else {
                vec![]
            };

            let cmds = if let Some(run_args) = args.run {
                run_args.clone()
            } else {
                let mut cmds = vec![package.to_string()];
                if let Some(ref args) = args.package_params {
                    cmds.extend(args.clone());
                }
                cmds
            };

            let _ = distrobox_enter(
                &package,
                &enter_args.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                &cmds.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            )
            .unwrap();
        }
    }
}
