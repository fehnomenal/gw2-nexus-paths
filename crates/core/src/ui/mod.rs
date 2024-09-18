mod marker_tree;

use egui::{Context, ScrollArea, Window};
use marker_tree::{marker_category_overview, marker_category_tree};

pub fn render_ui(_screen_width: f32, screen_height: f32, ctx: &Context) {
    Window::new("Paths")
        .max_height(screen_height / 2.0)
        .show(ctx, |ui| {
            marker_category_overview(ui);

            ScrollArea::vertical().show(ui, |ui| {
                marker_category_tree(ui);
            })
        });
}
