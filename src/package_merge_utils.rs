use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone)]
struct ContainerNode {
    packages: Vec<&'static str>,
    children: HashMap<&'static str, ContainerNode>,
}

fn move_common_packages_up(node: &mut ContainerNode) {
    for (_, child) in node.children.iter_mut() {
        move_common_packages_up(child);
    }

    if !node.children.is_empty() {
        let children_packages: Vec<&mut Vec<&'static str>> = node
            .children
            .iter_mut()
            .map(|(_, child)| &mut child.packages)
            .collect();

        move_common_packages_to_parent(&mut node.packages, children_packages);
    }
}

fn process_trees(trees: &mut HashMap<&'static str, ContainerNode>) {
    for (_, tree) in trees.iter_mut() {
        move_common_packages_up(tree);
    }
}

fn move_common_packages_to_parent(
    parent_packages: &mut Vec<&'static str>,
    children_packages: Vec<&mut Vec<&'static str>>,
) {
    let mut common_packages: HashSet<&str, RandomState> = if children_packages.len() > 1 {
        HashSet::from_iter(children_packages[0].iter().cloned())
    } else {
        HashSet::new()
    };

    for child_packages in &children_packages[1..] {
        common_packages = common_packages
            .intersection(&HashSet::from_iter(child_packages.iter().cloned()))
            .cloned()
            .collect();
    }

    let mut combined_packages: HashSet<&'static str> =
        HashSet::from_iter(parent_packages.iter().cloned());
    combined_packages.extend(common_packages);
    *parent_packages = combined_packages.into_iter().collect();
    parent_packages.sort_unstable();

    for child_packages in children_packages {
        child_packages.retain(|package| !parent_packages.contains(package));
        child_packages.sort_unstable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> HashMap<&'static str, ContainerNode> {
        HashMap::from_iter(vec![(
            "A",
            ContainerNode {
                packages: vec!["pkg1"],
                children: HashMap::from_iter(vec![
                    (
                        "B",
                        ContainerNode {
                            packages: vec!["pkg2", "pkg3"],
                            children: HashMap::from_iter(vec![(
                                "D",
                                ContainerNode {
                                    packages: vec!["pkg2", "pkg5"],
                                    children: HashMap::new(),
                                },
                            )]),
                        },
                    ),
                    (
                        "C",
                        ContainerNode {
                            packages: vec!["pkg2", "pkg4"],
                            children: HashMap::new(),
                        },
                    ),
                ]),
            },
        )])
    }
    #[test]
    fn test_process_trees() {
        let mut test_tree = create_test_tree();

        process_trees(&mut test_tree);
        let a_node = test_tree.get("A").unwrap();
        let b_node = a_node.children.get("B").unwrap();
        let c_node = a_node.children.get("C").unwrap();
        let d_node = b_node.children.get("D").unwrap();

        assert_eq!(d_node.packages, vec!["pkg5"]);
        assert_eq!(b_node.packages, vec!["pkg3"]);
        assert_eq!(c_node.packages, vec!["pkg4"]);
        assert_eq!(a_node.packages, vec!["pkg1", "pkg2"]);
    }

    #[test]
    fn test_move_common_packages_to_parent_case1() {
        let mut parent_packages = vec![];
        let mut child1 = vec!["a", "b", "c"];
        let mut child2 = vec!["b", "c", "d"];
        let mut child3 = vec!["a", "b", "e"];

        move_common_packages_to_parent(
            &mut parent_packages,
            vec![&mut child1, &mut child2, &mut child3],
        );

        assert_eq!(parent_packages, vec!["b"]);
        assert_eq!(child1, vec!["a", "c"]);
        assert_eq!(child2, vec!["c", "d"]);
        assert_eq!(child3, vec!["a", "e"]);
    }

    #[test]
    fn test_move_common_packages_to_parent_case2() {
        let mut parent_packages = vec![];
        let mut child1 = vec!["a", "b", "c"];
        let mut child2 = vec!["d", "e", "f"];

        move_common_packages_to_parent(&mut parent_packages, vec![&mut child1, &mut child2]);

        assert_eq!(parent_packages, vec![] as Vec<&str>);
        assert_eq!(child1, vec!["a", "b", "c"]);
        assert_eq!(child2, vec!["d", "e", "f"]);
    }

    #[test]
    fn test_move_common_packages_to_parent_case3() {
        let mut parent_packages = vec![];
        let mut child1 = vec!["a", "b", "c"];
        let mut child2 = vec!["a", "b", "c"];
        let mut child3 = vec!["a", "b", "c"];

        move_common_packages_to_parent(
            &mut parent_packages,
            vec![&mut child1, &mut child2, &mut child3],
        );

        assert_eq!(parent_packages, vec!["a", "b", "c"]);
        assert_eq!(parent_packages, vec!["a", "b", "c"]);
        assert_eq!(child1, vec![] as Vec<&str>);
        assert_eq!(child2, vec![] as Vec<&str>);
        assert_eq!(child3, vec![] as Vec<&str>);
    }

    #[test]
    fn test_move_common_packages_to_parent_case4() {
        let mut parent_packages = vec!["b"];
        let mut child1 = vec!["a", "b", "c", "d"];
        let mut child2 = vec!["b", "c", "d", "e"];
        let mut child3 = vec!["a", "b", "d", "f"];

        move_common_packages_to_parent(
            &mut parent_packages,
            vec![&mut child1, &mut child2, &mut child3],
        );

        assert_eq!(parent_packages, vec!["b", "d"]);
        assert_eq!(child1, vec!["a", "c"]);
        assert_eq!(child2, vec!["c", "e"]);
        assert_eq!(child3, vec!["a", "f"]);
    }
}
