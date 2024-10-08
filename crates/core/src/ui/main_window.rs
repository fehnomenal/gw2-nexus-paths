use egui::{Align2, Context, ScrollArea, Window};
use paths_data::markers::MarkerCategoryTree;

use crate::loadable::BackgroundLoadable;

use super::marker_tree::{marker_category_overview, marker_category_tree};

pub struct MainWindow {
    pub open: bool,
}

impl MainWindow {
    pub fn render<ReloadTreeFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
        &mut self,
        ctx: &Context,
        screen_height: f32,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        reload_tree: ReloadTreeFn,
        update_marker_settings: UpdateMarkerSettingsFn,
    ) {
        Window::new("Paths")
            .open(&mut self.open)
            .pivot(Align2::CENTER_CENTER)
            .max_height(screen_height / 2.0)
            .show(ctx, |ui| {
                marker_category_overview(ui, tree, &reload_tree);

                ScrollArea::vertical().show(ui, |ui| {
                    marker_category_tree(ui, tree, &update_marker_settings);
                })
            });
    }
}
