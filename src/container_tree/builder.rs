use crate::container_tree::distrobox_assemble_tree::*;
use crate::container_tree::merge::*;
use crate::distrobox::parser::assemble::ContainerAssembleData;
use std::collections::HashMap;

pub fn build_container_trees(
    distrobox_assemble_map: &HashMap<String, ContainerAssembleData>,
) -> Vec<ContainerNode> {
    // Parse the content and build the initial tree
    let trees = distrobox_assemble_to_trees(&distrobox_assemble_map);

    // Convert the tree into a map of PackageNodes
    let mut package_map = container_vec_to_package_map(&trees);

    // Process the map using process_trees function
    process_trees(&mut package_map);

    // Convert the processed map back into a vector of ContainerNodes
    let processed_trees = package_map_to_container_vec(&package_map, &trees);

    processed_trees
}

// Function to convert Vec<ContainerNode> to HashMap<String, PackageNode>
fn container_vec_to_package_map(nodes: &Vec<ContainerNode>) -> HashMap<String, PackageNode> {
    let mut map = HashMap::new();
    for node in nodes {
        let package_node = container_to_package(&node); // Use the function defined earlier
        map.insert(node.container_name.clone(), package_node);
    }
    map
}
// Function to convert ContainerNode to PackageNode
fn container_to_package(node: &ContainerNode) -> PackageNode {
    let packages = match &node.container_assemble_data.packages {
        Some(p) => p.iter().map(|s| s.clone()).collect(),
        None => Vec::new(),
    };

    // Transform each child ContainerNode into a PackageNode recursively
    let mut children = HashMap::new();
    for child in &node.children {
        children.insert(child.container_name.clone(), container_to_package(child));
    }

    PackageNode { packages, children }
}

// Function to convert HashMap<String, PackageNode> back to Vec<ContainerNode>
fn package_map_to_container_vec(
    map: &HashMap<String, PackageNode>,
    original_nodes: &Vec<ContainerNode>,
) -> Vec<ContainerNode> {
    let mut updated_nodes = Vec::new();
    for node in original_nodes {
        let package_node = map.get(&node.container_name).unwrap();
        let updated_node = merge_package_to_container(&package_node, &node);
        updated_nodes.push(updated_node);
    }
    updated_nodes
}
// Function to add PackageNode to ContainerNode, generating a new ContainerNode
fn merge_package_to_container(
    package_node: &PackageNode,
    container_node: &ContainerNode,
) -> ContainerNode {
    let mut new_node = container_node.clone();
    new_node.container_assemble_data.packages = Some(
        package_node
            .packages
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );

    // Process children nodes
    for child_node in &mut new_node.children {
        let child_package = package_node
            .children
            .get(&child_node.container_name)
            .unwrap();
        *child_node = merge_package_to_container(&child_package, &child_node);
    }

    new_node
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a helper function to create a ContainerAssembleData for testing
    fn create_test_container_assemble_data(
        image_name: String,
        packages: Vec<String>,
    ) -> ContainerAssembleData {
        let mut data = ContainerAssembleData::default();
        data.image = image_name;
        data.packages = Some(packages);
        data
    }

    #[test]
    fn test_build_container_trees() {
        // Create a test map of ContainerAssembleData
        let mut distrobox_assemble_map = HashMap::new();
        distrobox_assemble_map.insert(
            "Node1".to_string(),
            create_test_container_assemble_data(
                "Image1".to_string(),
                vec![
                    "Package1".to_string(),
                    "Package2".to_string(),
                    "Package3".to_string(),
                ],
            ),
        );
        distrobox_assemble_map.insert(
            "Node2".to_string(),
            create_test_container_assemble_data(
                "Image1".to_string(),
                vec![
                    "Package1".to_string(),
                    "Package2".to_string(),
                    "Package4".to_string(),
                    "Package5".to_string(),
                ],
            ),
        );
        distrobox_assemble_map.insert(
            "Node3".to_string(),
            create_test_container_assemble_data("Image0".to_string(), vec!["Package1".to_string()]),
        );
        distrobox_assemble_map.insert(
            "Node4".to_string(),
            create_test_container_assemble_data(
                "Image0".to_string(),
                vec![
                    "Package2".to_string(),
                    "Package3".to_string(),
                    "Package3".to_string(),
                ],
            ),
        );

        // Call build_container_tree function
        let trees = build_container_trees(&distrobox_assemble_map);

        // Check if the returned vector is not empty
        assert!(!trees.is_empty());

        // Check for the expected tree structure
        assert_eq!(trees.len(), 2);

        // Since the tree is sorted by container name, we can assert on each index
        assert_eq!(trees[0].container_name, "Image0");
        assert_eq!(trees[0].container_assemble_data.packages, Some(vec![]));
        assert_eq!(trees[0].children[0].container_name, "Node3");
        assert_eq!(
            trees[0].children[0].container_assemble_data.packages,
            Some(vec!["Package1".to_string()])
        );
        assert_eq!(trees[0].children[1].container_name, "Node4");
        assert_eq!(
            trees[0].children[1].container_assemble_data.packages,
            Some(vec!["Package2".to_string(), "Package3".to_string()])
        );

        assert_eq!(trees[1].container_name, "Image1");
        assert_eq!(
            trees[1].container_assemble_data.packages,
            Some(vec!["Package1".to_string(), "Package2".to_string()])
        );

        assert_eq!(trees[1].children[0].container_name, "Node1");
        assert_eq!(
            trees[1].children[0].container_assemble_data.packages,
            Some(vec!["Package3".to_string()])
        );

        assert_eq!(trees[1].children[1].container_name, "Node2");
        assert_eq!(
            trees[1].children[1].container_assemble_data.packages,
            Some(vec!["Package4".to_string(), "Package5".to_string()])
        );
    }

    // Define a helper function to create a ContainerNode for testing
    fn create_test_container_node(
        container_name: String,
        packages: Vec<String>,
        children: Vec<ContainerNode>,
    ) -> ContainerNode {
        let mut container_assemble_data = ContainerAssembleData::default();
        container_assemble_data.packages = Some(packages);
        ContainerNode {
            container_name,
            virtual_container: false,
            container_assemble_data,
            children,
        }
    }

    #[test]
    fn test_container_vec_to_package_map() {
        // Create a test vector of ContainerNodes
        let container_nodes = vec![
            create_test_container_node(
                "Node1".to_string(),
                vec!["Package1".to_string()],
                Vec::new(),
            ),
            create_test_container_node(
                "Node2".to_string(),
                vec!["Package2".to_string()],
                Vec::new(),
            ),
        ];

        let map = container_vec_to_package_map(&container_nodes);

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("Node1"));
        assert!(map.contains_key("Node2"));
    }

    #[test]
    fn test_package_map_to_container_vec() {
        // Create a test map of PackageNodes
        let mut package_map = HashMap::new();
        package_map.insert(
            "Node1".to_string(),
            PackageNode {
                packages: vec!["Package1".to_string()],
                children: HashMap::new(),
            },
        );
        package_map.insert(
            "Node2".to_string(),
            PackageNode {
                packages: vec!["Package2".to_string()],
                children: HashMap::new(),
            },
        );

        // Create original ContainerNodes
        let original_nodes = vec![
            create_test_container_node("Node1".to_string(), Vec::new(), Vec::new()),
            create_test_container_node("Node2".to_string(), Vec::new(), Vec::new()),
        ];

        let updated_nodes = package_map_to_container_vec(&package_map, &original_nodes);

        assert_eq!(updated_nodes.len(), 2);
        for node in updated_nodes {
            assert!(node.container_assemble_data.packages.is_some());
            assert!(!node.container_assemble_data.packages.unwrap().is_empty());
        }
    }
}
