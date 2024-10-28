use egui::{Context, Ui, Window};
use log_err::LogErrOption;

use crate::{
    loadable::BackgroundLoadable,
    markers::{ActiveMarkerCategories, MarkerCategoryTree},
    settings::Settings,
};

use super::{
    utils::{
        format_categories, format_points, format_trails, trail_color_selector, trail_width_selector,
    },
    UiActions,
};

pub struct MainWindow<A: UiActions> {
    pub actions: A,
    pub open: bool,
}

impl<A: UiActions> MainWindow<A> {
    pub fn render(
        &mut self,
        ctx: &Context,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        is_in_gameplay: bool,
        active_marker_categories: &ActiveMarkerCategories,
        settings: &mut Settings,
    ) {
        Window::new("Paths")
            .open(&mut self.open)
            .auto_sized()
            .show(ctx, |ui| {
                let is_loading_settings = matches!(tree, BackgroundLoadable::Loading);

                active_markers_info(
                    &self.actions,
                    ui,
                    is_loading_settings,
                    active_marker_categories,
                );

                limit_to_current_map_checkbox(
                    &self.actions,
                    ui,
                    is_loading_settings,
                    is_in_gameplay,
                    active_marker_categories,
                    &mut settings.limit_markers_to_current_map,
                );

                if let BackgroundLoadable::Loaded(tree) = tree {
                    let root_node = tree.tree.root().log_unwrap();

                    trail_color_selector(
                        &self.actions,
                        ui,
                        "Default route color:",
                        &root_node,
                        false,
                    );
                    trail_width_selector(
                        &self.actions,
                        ui,
                        "Default route width:",
                        &root_node,
                        false,
                    );
                }
            });
    }
}

fn active_markers_info<A: UiActions>(
    actions: &A,
    ui: &mut Ui,
    is_loading: bool,
    active_marker_categories: &ActiveMarkerCategories,
) {
    ui.horizontal(|ui| {
        let label = "Active markers".to_owned();

        if is_loading {
            ui.label(label);
            ui.spinner();
        } else {
            ui.label(format!(
                "{label}: {}; {}; {}",
                format_categories(active_marker_categories.active_category_count),
                format_points(
                    active_marker_categories
                        .all_active_points_of_interest()
                        .count()
                ),
                format_trails(active_marker_categories.all_active_trails().count()),
            ));

            if ui.link("change...").clicked() {
                actions.display_marker_tree_window();
            }
        }
    });
}

fn limit_to_current_map_checkbox<A: UiActions>(
    actions: &A,
    ui: &mut Ui,
    is_loading_settings: bool,
    is_in_gameplay: bool,
    active_marker_categories: &ActiveMarkerCategories,
    limit_markers_to_current_map: &mut bool,
) {
    let mut label = "limit to current map".to_owned();

    if !is_loading_settings && is_in_gameplay {
        label = format!(
            "{label} ({}; {})",
            format_points(
                active_marker_categories
                    .active_points_of_interest_of_current_map()
                    .count(),
            ),
            format_trails(
                active_marker_categories
                    .active_trails_of_current_map()
                    .count(),
            ),
        );
    }

    if ui.checkbox(limit_markers_to_current_map, label).changed() {
        actions.save_settings();
    }
}
