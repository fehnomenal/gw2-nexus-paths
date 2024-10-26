use egui::{Context, Slider, Ui, Window};
use log_err::LogErrOption;

use crate::{
    loadable::BackgroundLoadable,
    markers::{ActiveMarkerCategories, MarkerCategoryTree},
    settings::{Settings, TrailColor, TrailWidth},
};

use super::{
    utils::{format_categories, format_points, format_trails},
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
        marker_tree_window_open: &mut bool,
    ) {
        Window::new("Paths")
            .open(&mut self.open)
            .auto_sized()
            .show(ctx, |ui| {
                let is_loading_settings = matches!(tree, BackgroundLoadable::Loading);

                active_markers_info(
                    ui,
                    is_loading_settings,
                    active_marker_categories,
                    marker_tree_window_open,
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
                    trail_color_selector(&self.actions, ui, tree);
                    trail_width_selector(&self.actions, ui, tree);
                }
            });
    }
}

fn active_markers_info(
    ui: &mut Ui,
    is_loading: bool,
    active_marker_categories: &ActiveMarkerCategories,
    marker_tree_window_open: &mut bool,
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
                *marker_tree_window_open = true;
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

fn trail_color_selector<A: UiActions>(actions: &A, ui: &mut Ui, tree: &MarkerCategoryTree) {
    ui.horizontal(|ui| {
        ui.label("Route color:");

        let mut color = *tree
            .tree
            .root()
            .log_unwrap()
            .data()
            .trail_color
            .borrow()
            .log_unwrap();

        let resp = ui.color_edit_button_srgba_premultiplied(&mut color);
        if resp.changed() {
            *tree
                .tree
                .root()
                .log_unwrap()
                .data()
                .trail_color
                .borrow_mut() = Some(TrailColor(color));

            actions.update_active_marker_categories();
            actions.save_settings();
        }
    });
}

fn trail_width_selector<A: UiActions>(actions: &A, ui: &mut Ui, tree: &MarkerCategoryTree) {
    ui.horizontal(|ui| {
        ui.label("Route width:");

        let mut width = *tree
            .tree
            .root()
            .log_unwrap()
            .data()
            .trail_width
            .borrow()
            .log_unwrap();

        let resp = ui.add(Slider::new(&mut width, 1.0..=25.0));
        if resp.changed() {
            *tree
                .tree
                .root()
                .log_unwrap()
                .data()
                .trail_width
                .borrow_mut() = Some(TrailWidth(width));

            actions.update_active_marker_categories();
            actions.save_settings();
        }
    });
}
