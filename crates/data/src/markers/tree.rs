use std::{cell::RefCell, fs::read_dir, path::Path};

use nary_tree::{NodeId, NodeRef, Tree};
use paths_types::MarkerCategory;

pub type MarkerCategoryTreeNode<'a> = NodeRef<'a, MarkerCategory>;

pub struct MarkerCategoryTree {
    pub tree: Tree<MarkerCategory>,
    pub pack_count: usize,
    pub trail_count: usize,
}

impl MarkerCategoryTree {
    pub fn new() -> Self {
        let mut tree = Tree::new();

        tree.set_root(MarkerCategory {
            identifier: "".to_owned(),
            label: "".to_owned(),
            is_separator: false,
            is_selected: RefCell::new(false),
            points_of_interest: vec![],
            trails: vec![],
        });

        Self {
            tree,
            pack_count: 0,
            trail_count: 0,
        }
    }

    pub fn from_all_packs_in_dir(dir: &Path) -> Self {
        let mut tree = Self::new();

        if dir.exists() {
            for entry in read_dir(dir).expect("Could not read dir contents") {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    if path.is_file()
                        && path
                            .to_str()
                            .is_some_and(|p| p.to_lowercase().ends_with(".taco"))
                    {
                        tree.load_marker_pack_from_path(&path);
                    }
                }
            }
        }

        tree
    }
}

pub fn ensure_category_path<P: AsRef<str>, F: Fn(&P) -> MarkerCategory>(
    tree: &mut Tree<MarkerCategory>,
    start_node_id: NodeId,
    path: &[P],
    create_category: F,
) -> NodeId {
    let result = traverse_path(tree.get(start_node_id).unwrap(), path);

    match result {
        TraverseResult::Found(node_id) => node_id,

        TraverseResult::NotFound {
            mut current_node_id,
            remaining_path,
        } => {
            for id in remaining_path {
                let category = create_category(id);

                let mut current_parent_node = tree.get_mut(current_node_id).unwrap();
                let next_parent_node = current_parent_node.append(category);
                current_node_id = next_parent_node.node_id();
            }

            current_node_id
        }
    }
}

pub enum TraverseResult<'a, P: AsRef<str>> {
    Found(NodeId),
    NotFound {
        current_node_id: NodeId,
        remaining_path: &'a [P],
    },
}

fn traverse_path<'a, 'b, P: AsRef<str>>(
    sub_tree: MarkerCategoryTreeNode<'b>,
    mut path: &'a [P],
) -> TraverseResult<'a, P> {
    let mut current = sub_tree;

    loop {
        if path.is_empty() {
            return TraverseResult::Found(current.node_id());
        }

        let (current_id, rest) = path.split_first().unwrap();

        if let Some(child) = find_child_node_with_id(&current, current_id) {
            current = child;
            path = rest;
        } else {
            return TraverseResult::NotFound {
                current_node_id: current.node_id(),
                remaining_path: path,
            };
        }
    }
}

fn find_child_node_with_id<'a, I: AsRef<str>>(
    parent: &MarkerCategoryTreeNode<'a>,
    identifier: &I,
) -> Option<MarkerCategoryTreeNode<'a>> {
    let identifier = identifier.as_ref();

    for child in parent.children() {
        if child.data().identifier == identifier {
            return Some(child);
        }
    }

    None
}
