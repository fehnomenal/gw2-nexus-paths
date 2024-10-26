mod main_window;
mod marker_tree_window;
mod utils;

use egui::{Context, Visuals};

use crate::markers::ActiveMarkerCategories;
use crate::settings::Settings;
use crate::{loadable::BackgroundLoadable, markers::MarkerCategoryTree};

pub use self::main_window::MainWindow;
pub use self::marker_tree_window::MarkerTreeWindow;

pub struct UiState<A: UiActions> {
    pub actions: A,
    pub ui_was_displayed_once: bool,
    pub main_window: MainWindow<A>,
    pub marker_tree_window: MarkerTreeWindow<A>,
}

impl<A: UiActions> UiState<A> {
    pub fn render(
        &mut self,
        _screen_width: f32,
        _screen_height: f32,
        ctx: &Context,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        is_in_gameplay: bool,
        settings: &mut Settings,
        active_marker_categories: &ActiveMarkerCategories,
    ) {
        self.main_window.render(
            ctx,
            tree,
            is_in_gameplay,
            active_marker_categories,
            settings,
            &mut self.marker_tree_window.open,
        );

        self.marker_tree_window.render(ctx, tree);
    }
}

pub trait UiActions {
    fn reload_settings(&self);
    fn save_settings(&self);
    fn update_active_marker_categories(&self);
}

pub fn prepare_egui_context(ctx: Context) -> Context {
    ctx.set_visuals(Visuals::light());
    ctx.style_mut(|style| {
        style.interaction.selectable_labels = false;
    });

    ctx
}
