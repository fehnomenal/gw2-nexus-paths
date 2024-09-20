use egui::Rgba;
use paths_data::markers::{MarkerCategoryTree, MarkerCategoryTreeNode};
use paths_types::settings::Settings;

pub fn backup_marker_category_settings(tree: &MarkerCategoryTree<Rgba>, settings: &mut Settings) {
    let preset = settings
        .marker_presets
        .entry("Default".to_owned())
        .or_default();

    let root_node = tree.tree.root().unwrap();

    for node in root_node
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

        let id = category.path(&get_category_parent_path(&node)).join(".");

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

fn get_category_parent_path<C>(node: &MarkerCategoryTreeNode<C>) -> Vec<String> {
    node.ancestors()
        .map(|n| n.data().identifier.to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}
