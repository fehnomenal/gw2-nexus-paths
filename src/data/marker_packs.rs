use std::{
    fs::{create_dir_all, read_dir, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use nary_tree::{NodeId, NodeRef, Tree};
use quick_xml::events::{BytesStart, Event};
use zip::ZipArchive;

pub struct MarkerCategoryTree {
    tree: Tree<MarkerCategoryTreeNode>,

    pub pack_count: usize,
    pub trail_count: usize,
}

impl MarkerCategoryTree {
    fn new(tree: Tree<MarkerCategoryTreeNode>, pack_count: usize) -> Self {
        let trail_count = tree
            .root()
            .unwrap()
            .traverse_pre_order()
            .filter_map(|n| {
                if let MarkerCategoryTreeNode::Category(MarkerCategory { trails, .. }) = n.data() {
                    if !trails.is_empty() {
                        Some(trails.len())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .sum::<usize>();

        Self {
            tree,
            pack_count,
            trail_count,
        }
    }
}

enum MarkerCategoryTreeNode {
    Root,
    Category(MarkerCategory),
}

struct MarkerCategory {
    identifier: String,
    label: String,
    is_separator: bool,
    points_of_interest: Vec<PointOfInterest>,
    trails: Vec<TrailDescription>,
}

impl MarkerCategory {
    fn from_xml_element(element: &BytesStart<'_>) -> Option<Self> {
        let identifier = read_xml_attribute_value_case_insensitive(element, "Name")?;
        let label = read_xml_attribute_value_case_insensitive(element, "DisplayName")
            .unwrap_or(identifier.clone());
        let is_separator = read_xml_attribute_value_case_insensitive(element, "IsSeparator")
            .map(|s| s == "1")
            .unwrap_or_default();

        Some(MarkerCategory {
            identifier,
            label,
            is_separator,
            points_of_interest: vec![],
            trails: vec![],
        })
    }
}

struct PointOfInterest {
    // TODO
}

struct TrailDescription {
    ids: Vec<String>,
    pack_file: PathBuf,
    binary_file: String,
}

impl TrailDescription {
    fn from_xml_element(pack_file: &Path, element: &BytesStart<'_>) -> Option<Self> {
        let id = read_xml_attribute_value_case_insensitive(element, "Type")?;
        let file = read_xml_attribute_value_case_insensitive(element, "TrailData")?;

        Some(Self {
            ids: id.split('.').map(|p| p.to_owned()).collect(),
            pack_file: pack_file.to_path_buf(),
            binary_file: file,
        })
    }
}

pub fn load_all_marker_packs(dir: &Path) -> MarkerCategoryTree {
    create_dir_all(dir).unwrap();

    let mut tree = Tree::new();
    tree.set_root(MarkerCategoryTreeNode::Root);

    let mut pack_count = 0;

    for entry in read_dir(dir).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_file()
                && path
                    .to_str()
                    .is_some_and(|p| p.to_lowercase().ends_with(".taco"))
            {
                load_marker_pack_from_path(&path, &mut tree);

                pack_count += 1;
            }
        }
    }

    MarkerCategoryTree::new(tree, pack_count)
}

fn load_marker_pack_from_path(path: &Path, tree: &mut Tree<MarkerCategoryTreeNode>) {
    let file = File::open(path).unwrap();
    let mut zip = ZipArchive::new(file).unwrap();

    for i in 0..zip.len() {
        let file = zip.by_index(i).unwrap();

        let name = file.name();
        if !name.ends_with(".xml") {
            continue;
        }

        let reader = BufReader::new(file);
        let reader = quick_xml::Reader::from_reader(reader);

        append_from_xml_reader_to_tree(path, reader, tree);
    }
}

fn append_from_xml_reader_to_tree<B: BufRead>(
    pack_file: &Path,
    mut xml: quick_xml::Reader<B>,
    tree: &mut Tree<MarkerCategoryTreeNode>,
) {
    let mut current_parent_node_id = tree.root_id().expect("Tree has no root node");

    let mut buf = vec![];
    loop {
        let event = xml
            .read_event_into(&mut buf)
            // TODO: Error handling
            .expect("Could not read xml event");

        match event {
            Event::Eof => break,

            Event::Start(element) if element.local_name().as_ref() == b"MarkerCategory" => {
                if let Some(category) = MarkerCategory::from_xml_element(&element) {
                    let child_node_id = find_child_node_id_by_path(
                        tree.get(current_parent_node_id).unwrap(),
                        &[&category.identifier],
                    );

                    if let Some(existing_child_node_id) = child_node_id {
                        current_parent_node_id = existing_child_node_id;
                    } else {
                        let mut current_parent_node = tree.get_mut(current_parent_node_id).unwrap();

                        current_parent_node_id = current_parent_node
                            .append(MarkerCategoryTreeNode::Category(category))
                            .node_id();
                    }
                }
            }

            Event::End(element) if element.local_name().as_ref() == b"MarkerCategory" => {
                let mut current_parent_node = tree.get_mut(current_parent_node_id).unwrap();
                current_parent_node_id = current_parent_node.parent().unwrap().node_id();
            }

            Event::Empty(element) if element.local_name().as_ref() == b"MarkerCategory" => {
                if let Some(category) = MarkerCategory::from_xml_element(&element) {
                    let child_node_id = find_child_node_id_by_path(
                        tree.get(current_parent_node_id).unwrap(),
                        &[&category.identifier],
                    );

                    if child_node_id.is_none() {
                        let mut current_parent_node = tree.get_mut(current_parent_node_id).unwrap();

                        current_parent_node.append(MarkerCategoryTreeNode::Category(category));
                    }
                }
            }

            Event::Empty(element) if element.local_name().as_ref() == b"Trail" => {
                if let Some(trail_description) =
                    TrailDescription::from_xml_element(pack_file, &element)
                {
                    let category_node_id = find_child_node_id_by_path(
                        tree.root().unwrap(),
                        trail_description.ids.as_slice(),
                    );

                    let mut category_node = if let Some(category_node_id) = category_node_id {
                        tree.get_mut(category_node_id).unwrap()
                    } else {
                        let mut path = trail_description.ids.as_slice();
                        let mut current_parent_id = tree.root_id().unwrap();

                        loop {
                            if path.is_empty() {
                                break;
                            }

                            let (current_id, rest) = path.split_first().unwrap();

                            current_parent_id = if let Some(node) = find_child_node_with_id(
                                &tree.get(current_parent_id).unwrap(),
                                current_id,
                            ) {
                                node.node_id()
                            } else {
                                let mut current_parent = tree.get_mut(current_parent_id).unwrap();

                                let new_target_node = current_parent.append(
                                    MarkerCategoryTreeNode::Category(MarkerCategory {
                                        identifier: current_id.to_owned(),
                                        label: current_id.to_owned(),
                                        is_separator: false,
                                        points_of_interest: vec![],
                                        trails: vec![],
                                    }),
                                );

                                new_target_node.node_id()
                            };

                            path = rest;
                        }

                        tree.get_mut(current_parent_id).unwrap()
                    };

                    if let MarkerCategoryTreeNode::Category(cat) = category_node.data() {
                        cat.trails.push(trail_description);
                    }
                }
            }
            _ => {}
        };

        buf.clear();
    }
}

fn find_child_node_id_by_path<P: AsRef<str>>(
    sub_tree: NodeRef<'_, MarkerCategoryTreeNode>,
    mut path: &[P],
) -> Option<NodeId> {
    let mut current_parent = sub_tree;

    loop {
        if path.is_empty() {
            break;
        }

        let (current_id, rest) = path.split_first().unwrap();

        current_parent = find_child_node_with_id(&current_parent, current_id)?;
        path = rest;
    }

    Some(current_parent.node_id())
}

fn find_child_node_with_id<'a, 'b, I: AsRef<str>>(
    parent: &NodeRef<'a, MarkerCategoryTreeNode>,
    identifier: &'b I,
) -> Option<NodeRef<'a, MarkerCategoryTreeNode>> {
    let identifier = identifier.as_ref();

    for node in parent.children() {
        match node.data() {
            MarkerCategoryTreeNode::Category(MarkerCategory {
                identifier: cat_id, ..
            }) => {
                if cat_id == identifier {
                    return Some(node);
                }
            }

            MarkerCategoryTreeNode::Root => {}
        }
    }

    None
}

fn read_xml_attribute_value_case_insensitive(
    element: &BytesStart<'_>,
    name: &str,
) -> Option<String> {
    let lower_name = name.to_lowercase();

    for attr in element.attributes() {
        if let Ok(attr) = attr {
            let key =
                String::from_utf8(attr.key.local_name().as_ref().to_ascii_lowercase()).unwrap();

            if key == lower_name {
                return attr.unescape_value().ok().map(|v| v.into_owned());
            }
        }
    }

    None
}
