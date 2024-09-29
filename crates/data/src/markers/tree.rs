use std::{fs::read_dir, path::Path};

use log::debug;
use log_err::{LogErrOption, LogErrResult};
pub use nary_tree::NodeId;
use nary_tree::{NodeRef, Tree};
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

        tree.set_root(MarkerCategory::root());

        Self {
            tree,
            pack_count: 0,
            trail_count: 0,
        }
    }

    pub fn from_all_packs_in_dir(dir: &Path) -> Self {
        let mut tree = Self::new();

        if dir.exists() {
            for entry in read_dir(dir).log_expect("could not read dir contents") {
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

pub fn ensure_category_path<F: Fn(&String) -> MarkerCategory>(
    tree: &mut Tree<MarkerCategory>,
    start_node_id: NodeId,
    path: &[String],
    create_category: F,
) -> NodeId {
    let result = traverse_path(tree.get(start_node_id).log_unwrap(), path);

    match result {
        TraverseResult::Found(node_id) => node_id,

        TraverseResult::NotFound {
            mut current_node_id,
            remaining_path,
        } => {
            debug!("need to create categories {remaining_path:?}");

            for id in remaining_path {
                let category = create_category(&id);

                let mut current_parent_node = tree.get_mut(current_node_id).log_unwrap();
                let next_parent_node = current_parent_node.append(category);
                current_node_id = next_parent_node.node_id();
            }

            current_node_id
        }
    }
}

pub enum TraverseResult {
    Found(NodeId),
    NotFound {
        current_node_id: NodeId,
        remaining_path: Vec<String>,
    },
}

fn traverse_path(sub_tree: MarkerCategoryTreeNode, mut path: &[String]) -> TraverseResult {
    let mut current = sub_tree;

    loop {
        if path.is_empty() {
            return TraverseResult::Found(current.node_id());
        }

        let (current_id, rest) = path.split_first().log_unwrap();

        if let Some(child) = find_child_node_with_id(&current, current_id) {
            current = child;
            path = rest;
        } else {
            return TraverseResult::NotFound {
                current_node_id: current.node_id(),
                remaining_path: path.to_owned(),
            };
        }
    }
}

fn find_child_node_with_id<'a>(
    parent: &MarkerCategoryTreeNode<'a>,
    identifier: &str,
) -> Option<MarkerCategoryTreeNode<'a>> {
    for child in parent.children() {
        if child
            .data()
            .identifier
            .last()
            .log_expect("could not get last segment of identifier")
            == identifier
        {
            return Some(child);
        }
    }

    None
}
