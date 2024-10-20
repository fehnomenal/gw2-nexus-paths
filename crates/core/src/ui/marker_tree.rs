use egui::{collapsing_header::CollapsingState, Button, Ui};
use log_err::LogErrOption;

use crate::{
    loadable::BackgroundLoadable,
    markers::{MarkerCategoryTree, MarkerCategoryTreeNode, Property},
};

pub fn marker_category_overview<F: Fn()>(
    ui: &mut Ui,
    tree: &BackgroundLoadable<MarkerCategoryTree>,
    reload: &F,
) {
    ui.horizontal(|ui| {
        let is_loading = if let BackgroundLoadable::Loaded(tree) = tree {
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
            reload();
        }

        if is_loading {
            ui.spinner();
        }
    });
}

pub fn marker_category_tree<F: Fn()>(
    ui: &mut Ui,
    tree: &BackgroundLoadable<MarkerCategoryTree>,
    update_marker_settings: &F,
) {
    if let BackgroundLoadable::Loaded(tree) = tree {
        let root = tree.tree.root().log_expect("tree has no root node");

        marker_category_nodes(ui, tree, &root, update_marker_settings);
    }
}

fn marker_category_nodes<F: Fn()>(
    ui: &mut Ui,
    tree: &MarkerCategoryTree,
    parent: &MarkerCategoryTreeNode,
    update_marker_settings: &F,
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
                let mut is_active = category.is_active.borrow().get().to_owned();
                let checkbox = &ui.checkbox(
                    &mut is_active,
                    format!("{} ({})", &category.label, &trail_count),
                );

                if checkbox.changed() {
                    *category.is_active.borrow_mut() = Property::ExplicitlySet(is_active);

                    // All children inherit the new state unless they have an own value.
                    for sub_child in child.traverse_pre_order().skip(1) {
                        let mut current_is_active = sub_child.data().is_active.borrow_mut();

                        match *current_is_active {
                            Property::ExplicitlySet(is_active)
                                if *current_is_active.get() == is_active =>
                            {
                                // The value is explicitly set to the same value as its parent's value.
                                *current_is_active = Property::Inherited(
                                    sub_child
                                        .parent()
                                        .log_unwrap()
                                        .data()
                                        .is_active
                                        .borrow()
                                        .get()
                                        .to_owned(),
                                );
                            }

                            Property::ExplicitlySet(_) => {
                                // XXX: Optimization: This sub child has an own value so this sub tree could be skipped.
                            }

                            Property::Inherited(_) => {
                                // Update the inherited value.
                                *current_is_active = Property::Inherited(
                                    sub_child
                                        .parent()
                                        .log_unwrap()
                                        .data()
                                        .is_active
                                        .borrow()
                                        .get()
                                        .to_owned(),
                                );
                            }

                            Property::Unset => unreachable!(),
                        }
                    }

                    update_marker_settings();
                };
            }
        };

        if is_not_expandable {
            row(ui);
        } else {
            let id = ui.make_persistent_id(&category.identifier);

            CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    row(ui);
                })
                .body(|ui| {
                    marker_category_nodes(ui, tree, &child, update_marker_settings);
                });
        }
    }
}
