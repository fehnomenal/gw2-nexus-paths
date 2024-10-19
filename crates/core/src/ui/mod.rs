mod marker_tree;

use egui::{Context, ScrollArea, Window};
use marker_tree::{marker_category_overview, marker_category_tree};

use crate::loadable::BackgroundLoadable;
use crate::markers::MarkerCategoryTree;

pub fn render_ui<ReloadFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
    _screen_width: f32,
    screen_height: f32,
    ctx: &Context,
    tree: &BackgroundLoadable<MarkerCategoryTree>,
    reload: ReloadFn,
    update_marker_settings: UpdateMarkerSettingsFn,
) {
    Window::new("Paths")
        .max_height(screen_height / 2.0)
        .show(ctx, |ui| {
            marker_category_overview(ui, tree, &reload);

            ScrollArea::vertical().show(ui, |ui| {
                marker_category_tree(ui, tree, &update_marker_settings);
            })
        });
}
