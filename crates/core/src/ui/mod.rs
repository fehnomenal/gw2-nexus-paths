mod category_properties_window;
mod main_window;
mod marker_tree_window;
mod utils;

use egui::{Context, Visuals};
use nary_tree::NodeId;

use crate::markers::ActiveMarkerCategories;
use crate::settings::Settings;
use crate::{loadable::BackgroundLoadable, markers::MarkerCategoryTree};

pub use self::category_properties_window::CategoryPropertiesWindow;
pub use self::main_window::MainWindow;
pub use self::marker_tree_window::MarkerTreeWindow;

pub struct UiState<'a, A: UiActions> {
    pub actions: A,
    pub ui_was_displayed_once: bool,
    pub main_window: MainWindow<A>,
    pub marker_tree_window: MarkerTreeWindow<A>,
    pub category_properties_window: CategoryPropertiesWindow<'a, A>,
}

impl<A: UiActions + Copy> UiState<'_, A> {
    pub fn new(actions: A) -> Self {
        Self {
            actions,
            ui_was_displayed_once: false,
            main_window: MainWindow {
                actions,
                open: false,
            },
            marker_tree_window: MarkerTreeWindow {
                actions,
                open: false,
            },
            category_properties_window: CategoryPropertiesWindow {
                actions,
                current_category_node: None,
            },
        }
    }
}

impl<A: UiActions> UiState<'_, A> {
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
        );

        self.marker_tree_window.render(ctx, tree);

        self.category_properties_window.render(ctx);
    }
}

pub trait UiActions {
    fn reload_settings(&self);
    fn save_settings(&self);
    fn update_active_marker_categories(&self);
    fn display_marker_tree_window(&self);
    fn display_category_properties_window(&self, node_id: NodeId);
}

pub fn prepare_egui_context(ctx: Context) -> Context {
    ctx.set_visuals(Visuals::light());
    ctx.style_mut(|style| {
        style.interaction.selectable_labels = false;
    });

    ctx
}
