use egui::{collapsing_header::CollapsingState, Align, Context, Layout, ScrollArea, Ui, Window};
use log_err::LogErrOption;

use crate::{
    loadable::BackgroundLoadable,
    markers::{MarkerCategoryTree, MarkerCategoryTreeNode},
};

use super::utils::{format_categories, format_points, format_trails};

pub struct MarkerTreeWindow {
    pub open: bool,
}

impl MarkerTreeWindow {
    pub fn render<ReloadFn: Fn(), OnUpdateSettingsFn: Fn()>(
        &mut self,
        ctx: &Context,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        reload: ReloadFn,
        on_update_settings: OnUpdateSettingsFn,
    ) {
        Window::new("Active markers")
            .open(&mut self.open)
            .show(ctx, |ui| {
                marker_category_overview(ui, tree, &reload, &on_update_settings);

                if let BackgroundLoadable::Loaded(tree) = tree {
                    ui.separator();

                    ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            marker_category_tree(ui, tree, &on_update_settings);
                        });
                }
            });
    }
}

fn marker_category_overview<ReloadFn: Fn(), OnUpdateSettingsFn: Fn()>(
    ui: &mut Ui,
    tree: &BackgroundLoadable<MarkerCategoryTree>,
    reload: &ReloadFn,
    on_update_settings: &OnUpdateSettingsFn,
) {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label("Total markers:");

            ui.indent("marker_overview", |ui| {
                if let BackgroundLoadable::Loaded(tree) = tree {
                    ui.label(format_categories(tree.category_count));
                    ui.label(format_points(tree.point_of_interest_count));
                    ui.label(format_trails(tree.trail_count));
                } else {
                    ui.spinner();
                }
            });
        });

        ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
            let is_loading = matches!(tree, BackgroundLoadable::Loading);

            ui.add_enabled_ui(!is_loading, |ui| {
                if ui.button("Reload").clicked() {
                    reload();
                }

                if ui.button("Deselect all").clicked() {
                    if let BackgroundLoadable::Loaded(tree) = tree {
                        for node in tree.tree.root().log_unwrap().traverse_level_order().skip(1) {
                            // Each node inherits the false state from the root.
                            *node.data().is_active.borrow_mut() = None;
                        }

                        on_update_settings();
                    }
                }
            });
        });
    });
}

fn marker_category_tree<F: Fn()>(ui: &mut Ui, tree: &MarkerCategoryTree, on_update_settings: &F) {
    let root = tree.tree.root().log_expect("tree has no root node");

    marker_category_nodes(ui, tree, &root, false, on_update_settings);
}

fn marker_category_nodes<F: Fn()>(
    ui: &mut Ui,
    tree: &MarkerCategoryTree,
    parent: &MarkerCategoryTreeNode,
    parent_is_active: bool,
    on_update_settings: &F,
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

        let mut child_is_active = category.is_active.borrow().unwrap_or(parent_is_active);

        let is_not_expandable = category.is_separator || child.children().count() == 0;

        let mut row = |ui: &mut Ui| {
            if category.is_separator {
                ui.label(&category.label);
            } else {
                let checkbox = &ui.checkbox(
                    &mut child_is_active,
                    format!("{} ({})", &category.label, &trail_count),
                );

                if checkbox.changed() {
                    *category.is_active.borrow_mut() = Some(child_is_active);

                    fn inherit_active_state_if_possible(
                        parent: &MarkerCategoryTreeNode,
                        is_active: bool,
                    ) {
                        for child in parent.children() {
                            let category = child.data();
                            let mut child_is_active = category.is_active.borrow_mut();

                            if let Some(child_is_active_) = *child_is_active {
                                // This category is explicitly enabled or disabled.

                                if child_is_active_ == is_active {
                                    // Now it has the same state as its parent's.
                                    *child_is_active = None;

                                    inherit_active_state_if_possible(&child, is_active);
                                } else {
                                    // The active state is different than its parent's. Skip this sub tree.
                                }
                            } else {
                                inherit_active_state_if_possible(&child, is_active);
                            }
                        }
                    }

                    inherit_active_state_if_possible(&child, child_is_active);

                    on_update_settings();
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
                    marker_category_nodes(ui, tree, &child, child_is_active, on_update_settings);
                });
        }
    }
}
