mod main_window;
mod marker_tree;

use egui::Context;
pub use main_window::MainWindow;
use paths_data::markers::MarkerCategoryTree;

use crate::loadable::BackgroundLoadable;

pub struct UiState {
    pub ui_was_displayed_once: bool,
    pub main_window: MainWindow,
}

pub fn render_ui<ReloadFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
    state: &mut UiState,
    _screen_width: f32,
    screen_height: f32,
    ctx: &Context,
    tree: &BackgroundLoadable<MarkerCategoryTree>,
    reload: ReloadFn,
    update_marker_settings: UpdateMarkerSettingsFn,
) {
    state
        .main_window
        .render(ctx, screen_height, tree, reload, update_marker_settings);
}
