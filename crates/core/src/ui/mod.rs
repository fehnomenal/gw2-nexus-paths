mod marker_tree;

use egui::{Context, ScrollArea, Window};
use marker_tree::{marker_category_overview, marker_category_tree};
use paths_data::markers::MarkerCategoryTree;

use crate::loadable::BackgroundLoadable;

pub fn render_ui<ReloadTreeFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
    _screen_width: f32,
    screen_height: f32,
    ctx: &Context,
    tree: &BackgroundLoadable<MarkerCategoryTree>,
    reload_tree: ReloadTreeFn,
    update_marker_settings: UpdateMarkerSettingsFn,
) {
    Window::new("Paths")
        .max_height(screen_height / 2.0)
        .show(ctx, |ui| {
            marker_category_overview(ui, tree, &reload_tree);

            ScrollArea::vertical().show(ui, |ui| {
                marker_category_tree(ui, tree, &update_marker_settings);
            })
        });
}
