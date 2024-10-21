use egui::{Context, ScrollArea, Window};

use crate::{loadable::BackgroundLoadable, markers::MarkerCategoryTree};

use super::marker_tree::{marker_category_overview, marker_category_tree};

pub struct MainWindow {
    pub open: bool,
}

impl MainWindow {
    pub fn render<ReloadFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
        &mut self,
        ctx: &Context,
        screen_height: f32,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        reload: ReloadFn,
        update_marker_settings: UpdateMarkerSettingsFn,
    ) {
        Window::new("Paths")
            .open(&mut self.open)
            .max_height(screen_height / 2.0)
            .show(ctx, |ui| {
                marker_category_overview(ui, tree, &reload);

                ScrollArea::vertical().show(ui, |ui| {
                    marker_category_tree(ui, tree, &update_marker_settings);
                })
            });
    }
}
