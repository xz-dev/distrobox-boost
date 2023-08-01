use std::collections::HashMap;

use crate::container_tree::builder::build_container_trees;
use crate::container_tree::distrobox_assemble_tree::{trees_to_distrobox_assemble, ContainerNode};
use crate::distrobox::parser::assemble::ContainerAssembleData;
use crate::oci::image_builder::build_image;

fn build_image_by_tree(container_runner: &str, tree: &mut ContainerNode) {
    fn tree_to_image_map(container_runner: &str, tree: &mut ContainerNode, node_level: usize) {
        let image = &tree.container_assemble_data.image.clone();
        println!("Build image: {}", &image);
        let empty_vec = vec![];
        let packages = tree
            .container_assemble_data
            .packages
            .as_ref()
            .unwrap_or(&empty_vec);
        println!("Packages: {:?}", &packages);
        let new_image = format!("distrobox-{}-{}", node_level, &image);
        println!(
            "Build container name: {} to {}",
            &tree.container_name, &new_image
        );
        build_image(container_runner, &new_image, &image, packages).unwrap();
        tree.container_assemble_data.image = new_image.clone();
        for child in &mut tree.children {
            tree_to_image_map(container_runner, child, node_level + 1);
        }
    }
    tree_to_image_map(container_runner, tree, 0);
}

pub fn build_distrobox_assemble_data(
    container_runner: &str,
    data: &HashMap<String, ContainerAssembleData>,
) -> HashMap<String, ContainerAssembleData> {
    let mut trees = build_container_trees(data);

    for tree in &mut trees {
        build_image_by_tree(container_runner, tree);
    }

    let mut new_data = trees_to_distrobox_assemble(&trees);
    for (key, value) in new_data.iter_mut() {
        value.pull = Some(false);
        value.packages = data[key].packages.clone();
    }
    new_data
}
