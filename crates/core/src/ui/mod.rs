mod main_window;
mod marker_tree_window;
mod utils;

use egui::{Context, Visuals};

use crate::markers::ActiveMarkerCategories;
use crate::settings::Settings;
use crate::{loadable::BackgroundLoadable, markers::MarkerCategoryTree};

pub use self::main_window::MainWindow;
pub use self::marker_tree_window::MarkerTreeWindow;

pub struct UiState {
    pub ui_was_displayed_once: bool,
    pub main_window: MainWindow,
    pub marker_tree_window: MarkerTreeWindow,
}

impl UiState {
    pub fn render<ReloadFn: Fn(), OnUpdateSettingsFn: Fn()>(
        &mut self,
        _screen_width: f32,
        _screen_height: f32,
        ctx: &Context,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        is_in_gameplay: bool,
        settings: &mut Settings,
        active_marker_categories: &ActiveMarkerCategories,
        reload: ReloadFn,
        on_update_settings: OnUpdateSettingsFn,
    ) {
        self.main_window.render(
            ctx,
            tree,
            is_in_gameplay,
            active_marker_categories,
            settings,
            &mut self.marker_tree_window.open,
            &on_update_settings,
        );

        self.marker_tree_window
            .render(ctx, tree, &reload, &on_update_settings);
    }
}

pub fn prepare_egui_context(ctx: Context) -> Context {
    ctx.set_visuals(Visuals::light());
    ctx.style_mut(|style| {
        style.interaction.selectable_labels = false;
    });

    ctx
}
