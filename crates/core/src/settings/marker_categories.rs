use std::collections::HashMap;

#[cfg(debug_assertions)]
use log::trace;
use log_err::LogErrOption;

use crate::{
    markers::{MarkerCategoryTree, MarkerCategoryTreeNode},
    settings::MarkerCategorySetting,
};

use super::Settings;

pub fn backup_marker_category_settings(tree: &MarkerCategoryTree, settings: &mut Settings) {
    let preset = settings
        .marker_presets
        .entry("Default".to_owned())
        .or_default();

    // Read the default values from the root category.
    let root_node = tree.tree.root().log_unwrap();
    let root_category = root_node.data();
    settings.default_trail_color = root_category.trail_color.borrow().log_unwrap();
    settings.default_trail_width = root_category.trail_width.borrow().log_unwrap();

    fn persist_non_default_categories(
        parent: &MarkerCategoryTreeNode,
        parent_is_active: bool,
        preset: &mut HashMap<String, MarkerCategorySetting>,
    ) {
        for child in parent.children() {
            let category = child.data();
            let id = category.identifier.join(".");
            let child_is_active = category.is_active.borrow().unwrap_or(parent_is_active);

            let persist = if category.has_non_default_settings() {
                true
            } else {
                // Either the current category is explicitly set to active or inactive or the value is
                // inherited by the parent.
                // Only interesting if it differs from the parent's state.

                child_is_active != parent_is_active
            };

            if persist {
                let entry = preset.entry(id).or_default();

                entry.active = *category.is_active.borrow();
                entry.trail_color = *category.trail_color.borrow();
                entry.trail_width = *category.trail_width.borrow();
            } else {
                preset.remove(&id);
            }

            persist_non_default_categories(&child, child_is_active, preset);
        }
    }

    persist_non_default_categories(&root_node, false, preset);
}

pub fn apply_marker_category_settings(settings: &Settings, tree: &mut MarkerCategoryTree) {
    let empty_map = HashMap::new();
    let preset = settings.marker_presets.get("Default").unwrap_or(&empty_map);

    // Set the default values to the root category. This way all categories inherit the default values automatically.
    let mut root_node = tree.tree.root_mut().log_unwrap();
    let root_category = root_node.data();
    *root_category.is_active.get_mut() = Some(false);
    *root_category.trail_color.get_mut() = Some(settings.default_trail_color);
    *root_category.trail_width.get_mut() = Some(settings.default_trail_width);

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
            (setting.active, setting.trail_color, setting.trail_width);

            let category = node.data();

            *category.is_active.borrow_mut() = setting.active;
            *category.trail_color.borrow_mut() = setting.trail_color;
            *category.trail_width.borrow_mut() = setting.trail_width;

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
}
