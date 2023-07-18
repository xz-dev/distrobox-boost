use crate::distrobox_parser::assemble::ContainerAssembleData;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct ContainerNode {
    pub container_name: String,
    pub container_assemble_data: ContainerAssembleData,
    pub children: Vec<ContainerNode>,
}

pub fn distrobox_assemble_to_tree(
    container_assemble_data_map: &HashMap<String, ContainerAssembleData>,
) -> Vec<ContainerNode> {
    pub struct ContainerNodeRef {
        container_name: String,
        container_assemble_data: ContainerAssembleData,
        children: Vec<Rc<RefCell<ContainerNodeRef>>>,
    }
    let mut node_refs: BTreeMap<&str, Rc<RefCell<ContainerNodeRef>>> = BTreeMap::new();

    // First pass: create nodes and establish parent-child relationships
    for (container_name, container_assemble_data) in container_assemble_data_map {
        let image_name = &container_assemble_data.image;

        let node_ref = Rc::new(RefCell::new(ContainerNodeRef {
            container_name: container_name.clone(),
            container_assemble_data: container_assemble_data.clone(),
            children: vec![],
        }));
        node_refs.insert(&container_name, node_ref.clone());

        // Only add the parent node to the map if it doesn't exist yet
        if !container_assemble_data_map.contains_key(image_name) {
            let container_assemble_data = ContainerAssembleData {
                image: image_name.clone(),
                ..Default::default()
            };
            let parent_node_ref = Rc::new(RefCell::new(ContainerNodeRef {
                container_name: image_name.clone(),
                container_assemble_data,
                children: vec![],
            }));
            node_refs.insert(image_name, parent_node_ref);
        }
    }

    // Second pass: flatten the structure
    for (container_name, node_ref) in &node_refs {
        let image_name = node_ref.borrow().container_assemble_data.image.clone();
        if container_name != &image_name {
            let parent_node_name = image_name.as_str();
            let parent_node = node_refs.get(parent_node_name).unwrap();
            parent_node.borrow_mut().children.push(Rc::clone(node_ref));
        }
    }

    // Recursive function to convert ContainerNodeRef into ContainerNode
    fn convert_to_container_node(node_ref: &Rc<RefCell<ContainerNodeRef>>) -> ContainerNode {
        let node_borrow = node_ref.borrow();
        let mut children = node_borrow
            .children
            .iter()
            .map(|child_ref| {
                // Recursive call to convert all children
                convert_to_container_node(child_ref)
            })
            .collect::<Vec<_>>();

        // Sort children by container_name
        children.sort_by(|a, b| a.container_name.cmp(&b.container_name));

        ContainerNode {
            container_name: node_borrow.container_name.clone(),
            container_assemble_data: node_borrow.container_assemble_data.clone(),
            children, // Fill in the sorted children
        }
    }
    // Filter out non-root nodes and convert to ContainerNode
    node_refs
        .values()
        .filter(|node_ref| {
            node_ref.borrow().container_name == node_ref.borrow().container_assemble_data.image
        })
        .map(|node_ref| {
            // Convert each root ContainerNodeRef into ContainerNode
            convert_to_container_node(node_ref)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distrobox_assemble_to_tree() {
        let mut container_assemble_data_map = HashMap::new();
        let mut data1 = ContainerAssembleData::default();
        data1.image = "base".to_string();
        container_assemble_data_map.insert("container1".to_string(), data1);
        let mut data3 = ContainerAssembleData::default();
        // test sort
        data3.image = "container1".to_string();
        container_assemble_data_map.insert("container3".to_string(), data3);
        let mut data2 = ContainerAssembleData::default();
        data2.image = "container1".to_string();
        container_assemble_data_map.insert("container2".to_string(), data2);

        let mut data4 = ContainerAssembleData::default();
        data4.image = "base1".to_string();
        container_assemble_data_map.insert("container4".to_string(), data4);

        let container_tree = distrobox_assemble_to_tree(&container_assemble_data_map);

        assert_eq!(container_tree.len(), 2);
        assert_eq!(container_tree[0].container_name, "base");
        assert_eq!(container_tree[0].container_assemble_data.image, "base");
        assert_eq!(container_tree[0].children.len(), 1);
        assert_eq!(container_tree[0].children[0].container_name, "container1");
        assert_eq!(container_tree[0].children[0].children.len(), 2);
        assert_eq!(
            container_tree[0].children[0].children[0].container_name,
            "container2"
        );
        assert_eq!(
            container_tree[0].children[0].children[1].container_name,
            "container3"
        );

        assert_eq!(container_tree[1].container_name, "base1");
        assert_eq!(container_tree[1].children.len(), 1);
        assert_eq!(container_tree[1].children[0].container_name, "container4");
    }
}
