use log::trace;
use log_err::LogErrOption;
use paths_data::markers::{MarkerCategoryTree, NodeId};
use paths_types::settings::{MarkerCategorySetting, Settings};

pub fn backup_marker_category_settings(tree: &MarkerCategoryTree, settings: &mut Settings) {
    let preset = settings
        .marker_presets
        .entry("Default".to_owned())
        .or_default();

    for node in tree
        .tree
        .root()
        .log_unwrap()
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
                .log_expect("we always have a parent as we skip the root node");

            let parent_category = parent_node.data();

            // The root category is always NOT selected. So if this hits
            // the root category the process is still correct as only
            // active categories would be persisted which is the expected
            // behavior.

            category.is_active != parent_category.is_active
        };

        let id = node.data().identifier.join(".");

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

pub fn apply_marker_category_settings(settings: &Settings, tree: &mut MarkerCategoryTree) {
    let Some(preset) = settings.marker_presets.get("Default") else {
        return;
    };

    let mut nodes_to_set = Vec::<(NodeId, &MarkerCategorySetting)>::new();

    for node in tree
        .tree
        .root()
        .log_unwrap()
        .traverse_pre_order()
        // Skip the root node itself as it does not represent a real category and will never be persisted.
        .skip(1)
    {
        let id = node.data().identifier.join(".");

        if let Some(setting) = preset.get(&id) {
            // Set the current settings for all child categories. If a child has custom settings it will be
            // overwritten by a later invocation.

            nodes_to_set.append(
                &mut node
                    .traverse_pre_order()
                    .map(|n| (n.node_id(), setting))
                    .collect(),
            );
        }
    }

    // This is a two step process because we cannot borrow the tree multiple
    // times to set all the nodes.

    for (node_id, setting) in nodes_to_set {
        let mut node_mut = tree.tree.get_mut(node_id).log_unwrap();
        let category = node_mut.data();

        *category.is_active.get_mut() = setting.active;
        if category.trail_color.is_none() {
            category.trail_color = setting.trail_color;
        }
        if category.trail_width.is_none() {
            category.trail_width = setting.trail_width;
        }

        #[cfg(debug_assertions)]
        {
            trace!(
                "Applied settings to category: {}",
                category.identifier.join(".")
            );
            trace!("  active: {:?}", setting.active);
            trace!("  trail color: {:?}", setting.trail_color);
            trace!("  trail width: {:?}", setting.trail_width);
        }
    }
}
