use std::collections::HashMap;

#[cfg(debug_assertions)]
use log::trace;
use log_err::LogErrOption;

use crate::markers::{MarkerCategoryTree, Property};

use super::{Settings, TrailColor, TrailWidth};

pub fn backup_marker_category_settings(tree: &MarkerCategoryTree, settings: &mut Settings) {
    let preset = settings
        .marker_presets
        .entry("Default".to_owned())
        .or_default();

    // Read the default values from the root category.
    let root_node = tree.tree.root().log_unwrap();
    let root_category = root_node.data();
    settings.default_trail_color = root_category.trail_color.borrow().get().to_owned();
    settings.default_trail_width = root_category.trail_width.borrow().get().to_owned();

    for node in tree
        .tree
        .root()
        .log_unwrap()
        .traverse_pre_order()
        // Skip the root node itself as it does not represent a real category and will never be persisted.
        .skip(1)
    {
        let category = node.data();
        let id = category.identifier.join(".");

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

            category.is_active.borrow().get() != parent_category.is_active.borrow().get()
        };

        if persist {
            let entry = preset.entry(id).or_default();

            entry.active = category.is_active.borrow().explicitly_set();
            entry.trail_color = category.trail_color.borrow().explicitly_set();
            entry.trail_width = category.trail_width.borrow().explicitly_set();
        } else {
            preset.remove(&id);
        }
    }
}

pub fn apply_marker_category_settings(settings: &Settings, tree: &mut MarkerCategoryTree) {
    let empty_map = HashMap::new();
    let preset = settings.marker_presets.get("Default").unwrap_or(&empty_map);

    // Set the default values to the root category. This way all categories inherit the default values automatically.
    let mut root_node = tree.tree.root_mut().log_unwrap();
    let root_category = root_node.data();
    *root_category.is_active.get_mut() = Property::ExplicitlySet(false);
    *root_category.trail_color.get_mut() = Property::ExplicitlySet(settings.default_trail_color);
    *root_category.trail_width.get_mut() = Property::ExplicitlySet(settings.default_trail_width);

    let mut nodes_to_set = Vec::new();

    for node in tree
        .tree
        .root()
        .log_unwrap()
        .traverse_pre_order()
        // Skip the root node itself as it does not represent a real category and will never be persisted.
        .skip(1)
    {
        let id = node.data().identifier.join(".");

        let temp = if let Some(setting) = preset.get(&id) {
            MarkerCategoryTemp {
                is_active: setting
                    .active
                    .map_or(PropertyTemp::Inherited, PropertyTemp::ExplicitlySet),
                trail_color: setting
                    .trail_color
                    .map_or(PropertyTemp::Inherited, PropertyTemp::ExplicitlySet),
                trail_width: setting
                    .trail_width
                    .map_or(PropertyTemp::Inherited, PropertyTemp::ExplicitlySet),
            }
        } else {
            MarkerCategoryTemp {
                is_active: PropertyTemp::Inherited,
                trail_color: PropertyTemp::Inherited,
                trail_width: PropertyTemp::Inherited,
            }
        };

        nodes_to_set.push((node.node_id(), temp));
    }

    // This is a two step process because we cannot borrow the tree multiple
    // times to set all the nodes.

    for (node_id, setting) in nodes_to_set {
        let mut node = tree.tree.get_mut(node_id).log_unwrap();
        let mut parent = node.parent().log_unwrap();

        let is_active = match setting.is_active {
            PropertyTemp::ExplicitlySet(is_active) => Property::ExplicitlySet(is_active),
            PropertyTemp::Inherited => {
                Property::Inherited(parent.data().is_active.borrow().get().to_owned())
            }
        };

        let trail_color = match setting.trail_color {
            PropertyTemp::ExplicitlySet(color) => Property::ExplicitlySet(color),
            PropertyTemp::Inherited => {
                Property::Inherited(parent.data().trail_color.borrow().get().to_owned())
            }
        };

        let trail_width = match setting.trail_width {
            PropertyTemp::ExplicitlySet(width) => Property::ExplicitlySet(width),
            PropertyTemp::Inherited => {
                Property::Inherited(parent.data().trail_width.borrow().get().to_owned())
            }
        };

        let category = node.data();

        *category.is_active.get_mut() = is_active;
        *category.trail_color.get_mut() = trail_color;
        *category.trail_width.get_mut() = trail_width;

        #[cfg(debug_assertions)]
        {
            trace!(
                "Applied settings to category: {}",
                category.identifier.join(".")
            );
            trace!("  active: {:?}", setting.is_active);
            trace!("  trail color: {:?}", setting.trail_color);
            trace!("  trail width: {:?}", setting.trail_width);
        }
    }
}

#[derive(Debug)]
struct MarkerCategoryTemp {
    pub is_active: PropertyTemp<bool>,
    pub trail_color: PropertyTemp<TrailColor>,
    pub trail_width: PropertyTemp<TrailWidth>,
}

#[derive(Debug)]
enum PropertyTemp<T> {
    ExplicitlySet(T),
    Inherited,
}
