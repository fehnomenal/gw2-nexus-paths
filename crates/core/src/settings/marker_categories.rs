use egui::Rgba;
use paths_data::markers::{MarkerCategoryTree, MarkerCategoryTreeNode, NodeId};
use paths_types::settings::{MarkerCategorySetting, Settings};

pub fn backup_marker_category_settings(tree: &MarkerCategoryTree<Rgba>, settings: &mut Settings) {
    let preset = settings
        .marker_presets
        .entry("Default".to_owned())
        .or_default();

    for node in tree
        .tree
        .root()
        .unwrap()
        .traverse_pre_order()
        // Skip the root node itself as it does not represent a real category and will never be persisted.
        .skip(1)
    {
        let category = node.data();

        let persist = if category.has_non_default_settings() {
            true
        } else {
            // Detect whether the active state is different than its parent's.

            let parent_node = node
                .parent()
                .expect("We always have a parent as we skip the root node");

            let parent_category = parent_node.data();

            // The root category is always NOT selected. So if this hits
            // the root category the process is still correct as only
            // active categories would be persisted which is the expected
            // behavior.

            category.is_active != parent_category.is_active
        };

        let id = get_category_id(&node);

        if persist {
            let entry = preset.entry(id).or_default();

            entry.active = category.is_active.borrow().to_owned();
            entry.trail_color = category.trail_color;
            entry.trail_width = category.trail_width;
        } else {
            preset.remove(&id);
        }
    }
}

pub fn apply_marker_category_settings(settings: &Settings, tree: &mut MarkerCategoryTree<Rgba>) {
    let preset = settings.marker_presets.get("Default");

    if preset.is_none() {
        return;
    }

    let preset = preset.expect("We just checked this!");

    let mut nodes_to_set = Vec::<(NodeId, MarkerCategorySetting)>::new();

    for node in tree
        .tree
        .root()
        .unwrap()
        .traverse_pre_order()
        // Skip the root node itself as it does not represent a real category and will never be persisted.
        .skip(1)
    {
        let id = get_category_id(&node);

        if let Some(setting) = preset.get(&id) {
            // Set all info for the referenced category.
            nodes_to_set.push((node.node_id(), setting.clone()));

            // Set only the active state for all child categories.
            nodes_to_set.append(
                &mut node
                    .traverse_pre_order()
                    .map(|n| {
                        (
                            n.node_id(),
                            MarkerCategorySetting {
                                active: setting.active,
                                ..Default::default()
                            },
                        )
                    })
                    .collect(),
            );
        }
    }

    // This is a two step process because we cannot borrow the tree multiple
    // times to set all the nodes.

    for (node_id, setting) in nodes_to_set {
        let mut node_mut = tree.tree.get_mut(node_id).unwrap();
        let category = node_mut.data();

        *category.is_active.get_mut() = setting.active;
        category.trail_color = setting.trail_color;
        category.trail_width = setting.trail_width;
    }
}

fn get_category_id<C>(node: &MarkerCategoryTreeNode<C>) -> String {
    let mut parent_path = node
        .ancestors()
        .map(|n| n.data().identifier.to_owned())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    parent_path.reverse();

    node.data().path(&parent_path).join(".")
}
