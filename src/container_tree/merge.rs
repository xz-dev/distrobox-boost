use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone)]
pub struct PackageNode {
    pub packages: Vec<String>,
    pub children: HashMap<String, PackageNode>,
}

pub fn process_trees(trees: &mut HashMap<String, PackageNode>) {
    for (_, tree) in trees.iter_mut() {
        move_common_packages_up(tree);
    }
}

fn move_common_packages_up(node: &mut PackageNode) {
    for (_, child) in node.children.iter_mut() {
        move_common_packages_up(child);
    }

    if !node.children.is_empty() {
        let children_packages: Vec<&mut Vec<String>> = node
            .children
            .iter_mut()
            .map(|(_, child)| &mut child.packages)
            .collect();

        move_common_packages_to_parent(&mut node.packages, children_packages);
    }
}

fn move_common_packages_to_parent(
    parent_packages: &mut Vec<String>,
    children_packages: Vec<&mut Vec<String>>,
) {
    let mut common_packages: HashSet<String, RandomState> = if children_packages.len() > 1 {
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

    let mut combined_packages: HashSet<String> =
        HashSet::from_iter(parent_packages.iter().cloned());
    combined_packages.extend(common_packages);
    *parent_packages = combined_packages.into_iter().collect();
    parent_packages.sort_unstable();
    parent_packages.dedup();

    for child_packages in children_packages {
        child_packages.retain(|package| !parent_packages.contains(package));
        child_packages.sort_unstable();
        child_packages.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> HashMap<String, PackageNode> {
        HashMap::from_iter(vec![(
            "A".to_string(),
            PackageNode {
                packages: vec!["pkg1".to_string()],
                children: HashMap::from_iter(vec![
                    (
                        "B".to_string(),
                        PackageNode {
                            packages: vec!["pkg2".to_string(), "pkg3".to_string()],
                            children: HashMap::from_iter(vec![(
                                "D".to_string(),
                                PackageNode {
                                    packages: vec!["pkg2".to_string(), "pkg5".to_string()],
                                    children: HashMap::new(),
                                },
                            )]),
                        },
                    ),
                    (
                        "C".to_string(),
                        PackageNode {
                            packages: vec!["pkg2".to_string(), "pkg4".to_string()],
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

        assert_eq!(d_node.packages, vec!["pkg5".to_string()]);
        assert_eq!(b_node.packages, vec!["pkg3".to_string()]);
        assert_eq!(c_node.packages, vec!["pkg4".to_string()]);
        assert_eq!(
            a_node.packages,
            vec!["pkg1".to_string(), "pkg2".to_string()]
        );
    }

    #[test]
    fn test_move_common_packages_to_parent_case1() {
        let mut parent_packages = vec![];
        let mut child1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut child2 = vec!["b".to_string(), "c".to_string(), "d".to_string()];
        let mut child3 = vec!["a".to_string(), "b".to_string(), "e".to_string()];

        move_common_packages_to_parent(
            &mut parent_packages,
            vec![&mut child1, &mut child2, &mut child3],
        );

        assert_eq!(parent_packages, vec!["b".to_string()]);
        assert_eq!(child1, vec!["a".to_string(), "c".to_string()]);
        assert_eq!(child2, vec!["c".to_string(), "d".to_string()]);
        assert_eq!(child3, vec!["a".to_string(), "e".to_string()]);
    }

    #[test]
    fn test_move_common_packages_to_parent_case2() {
        let mut parent_packages = vec![];
        let mut child1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut child2 = vec!["d".to_string(), "e".to_string(), "f".to_string()];

        move_common_packages_to_parent(&mut parent_packages, vec![&mut child1, &mut child2]);

        assert_eq!(parent_packages, vec![] as Vec<String>);
        assert_eq!(
            child1,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(
            child2,
            vec!["d".to_string(), "e".to_string(), "f".to_string()]
        );
    }

    #[test]
    fn test_move_common_packages_to_parent_case3() {
        let mut parent_packages = vec![];
        let mut child1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut child2 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut child3 = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        move_common_packages_to_parent(
            &mut parent_packages,
            vec![&mut child1, &mut child2, &mut child3],
        );

        assert_eq!(
            parent_packages,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(
            parent_packages,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert_eq!(child1, vec![] as Vec<String>);
        assert_eq!(child2, vec![] as Vec<String>);
        assert_eq!(child3, vec![] as Vec<String>);
    }

    #[test]
    fn test_move_common_packages_to_parent_case4() {
        let mut parent_packages = vec!["b".to_string()];
        let mut child1 = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let mut child2 = vec![
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let mut child3 = vec![
            "a".to_string(),
            "b".to_string(),
            "d".to_string(),
            "f".to_string(),
        ];

        move_common_packages_to_parent(
            &mut parent_packages,
            vec![&mut child1, &mut child2, &mut child3],
        );

        assert_eq!(parent_packages, vec!["b".to_string(), "d".to_string()]);
        assert_eq!(child1, vec!["a".to_string(), "c".to_string()]);
        assert_eq!(child2, vec!["c".to_string(), "e".to_string()]);
        assert_eq!(child3, vec!["a".to_string(), "f".to_string()]);
    }
}
