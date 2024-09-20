use egui::{collapsing_header::CollapsingState, Button, Rgba, Ui};
use paths_data::markers::{MarkerCategoryTree, MarkerCategoryTreeNode};

use crate::{
    settings::backup_marker_category_settings,
    state::{
        get_marker_category_tree, load_marker_category_tree_in_background, update_settings,
        BackgroundLoadable,
    },
};

pub fn marker_category_overview(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let is_loading =
            if let BackgroundLoadable::Loaded(tree) = unsafe { get_marker_category_tree() } {
                ui.label(format!(
                    "Loaded {} pack{} with {} route{}",
                    tree.pack_count,
                    if tree.pack_count == 1 { "" } else { "s" },
                    tree.trail_count,
                    if tree.trail_count == 1 { "" } else { "s" }
                ));

                false
            } else {
                ui.label("Loading markers...");

                true
            };

        let reload_button = &ui.add_enabled(!is_loading, Button::new("Reload"));

        if reload_button.clicked() {
            unsafe { load_marker_category_tree_in_background() };
        }

        if is_loading {
            ui.spinner();
        }
    });
}

pub fn marker_category_tree(ui: &mut Ui) {
    if let BackgroundLoadable::Loaded(tree) = unsafe { get_marker_category_tree() } {
        let root = tree.tree.root().expect("Tree has no root node");

        marker_category_nodes(ui, tree, &root, &vec![]);
    }
}

fn marker_category_nodes(
    ui: &mut Ui,
    tree: &MarkerCategoryTree<Rgba>,
    parent: &MarkerCategoryTreeNode<Rgba>,
    parent_path: &[String],
) {
    for child in parent.children() {
        let category = child.data();

        let trail_count: usize = child
            .traverse_pre_order()
            .map(|n| n.data().trails.len())
            .sum();

        if trail_count == 0 && !category.is_separator {
            continue;
        }

        let is_not_expandable = category.is_separator || child.children().count() == 0;

        let row = |ui: &mut Ui| {
            if category.is_separator {
                ui.label(&category.label);
            } else {
                let checkbox = &ui.checkbox(
                    &mut category.is_active.borrow_mut(),
                    format!("{} ({})", &category.label, &trail_count),
                );

                if checkbox.changed() {
                    for child in child.traverse_pre_order().skip(1) {
                        *child.data().is_active.borrow_mut() = *category.is_active.borrow();
                    }

                    // TODO: Trigger loading the selected trails

                    unsafe {
                        update_settings(|settings| {
                            backup_marker_category_settings(tree, settings);
                        })
                    };
                }
            }
        };

        if is_not_expandable {
            row(ui);
        } else {
            let path = category.path(&parent_path);
            let id = ui.make_persistent_id(&path);

            CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    row(ui);
                })
                .body(|ui| {
                    marker_category_nodes(ui, tree, &child, &path);
                });
        }
    }
}
