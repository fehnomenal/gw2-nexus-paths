use std::{fs::read_dir, path::Path};

pub use nary_tree::NodeId;
use nary_tree::{NodeRef, Tree};
use paths_types::MarkerCategory;

pub type MarkerCategoryTreeNode<'a, C> = NodeRef<'a, MarkerCategory<C>>;

pub struct MarkerCategoryTree<C> {
    pub tree: Tree<MarkerCategory<C>>,
    pub pack_count: usize,
    pub trail_count: usize,
}

impl<C> MarkerCategoryTree<C> {
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

pub fn ensure_category_path<C, F: Fn(&String) -> MarkerCategory<C>>(
    tree: &mut Tree<MarkerCategory<C>>,
    start_node_id: NodeId,
    path: &[String],
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
                let category = create_category(&id);

                let mut current_parent_node = tree.get_mut(current_node_id).unwrap();
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

fn traverse_path<C>(sub_tree: MarkerCategoryTreeNode<C>, mut path: &[String]) -> TraverseResult {
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
                remaining_path: path.to_owned(),
            };
        }
    }
}

fn find_child_node_with_id<'a, C>(
    parent: &MarkerCategoryTreeNode<'a, C>,
    identifier: &str,
) -> Option<MarkerCategoryTreeNode<'a, C>> {
    for child in parent.children() {
        if child.data().identifier == identifier {
            return Some(child);
        }
    }

    None
}
