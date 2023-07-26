use std::collections::HashMap;

use crate::container_tree::builder::build_container_trees;
use crate::container_tree::distrobox_assemble_tree::{trees_to_distrobox_assemble, ContainerNode};
use crate::distrobox_parser::assemble::ContainerAssembleData;
use crate::oci::image_builder::build_image;

fn build_image_by_tree(container_runner: &str, tree: &ContainerNode) -> HashMap<String, String> {
    let mut image_map = HashMap::new();
    fn tree_to_image_map(
        container_runner: &str,
        tree: &ContainerNode,
        image_map: &mut HashMap<String, String>,
    ) {
        let image = &tree.container_assemble_data.image;
        if !image_map.contains_key(image) {
            let empty_vec = Vec::new();
            let packages = tree
                .container_assemble_data
                .packages
                .as_ref()
                .unwrap_or(&empty_vec);
            let new_image = format!("distrobox-{}", &image);
            println!("Build container name: {} to {}", &tree.container_name, &new_image);
            build_image(container_runner, &new_image, &image, packages).unwrap();
            image_map.insert(image.clone(), new_image);
        }
        for child in &tree.children {
            tree_to_image_map(container_runner, child, image_map);
        }
    }
    tree_to_image_map(container_runner, tree, &mut image_map);
    image_map
}

pub fn build_distrobox_assemble_data(
    container_runner: &str,
    data: &HashMap<String, ContainerAssembleData>,
) -> HashMap<String, ContainerAssembleData> {
    let mut trees = build_container_trees(data);

    fn change_tree_image(tree: &mut ContainerNode, image_map: &HashMap<String, String>) {
        let image = &tree.container_assemble_data.image;
        if image_map.contains_key(image) {
            tree.container_assemble_data.image = image_map[image].clone();
        }
        for child in &mut tree.children {
            change_tree_image(child, image_map);
        }
    }

    for tree in &mut trees {
        let image_map = build_image_by_tree(container_runner, tree);
        change_tree_image(tree, &image_map);
    }

    trees_to_distrobox_assemble(&trees)
}
