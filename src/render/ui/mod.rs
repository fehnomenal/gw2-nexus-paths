use egui::{
    collapsing_header::CollapsingState, Button, Context, Pos2, RawInput, Rect, ScrollArea, Ui,
    Vec2, Window,
};
use manager::InputManager;
use paths_core::state::{
    get_marker_category_tree, get_mumble_link, load_marker_category_tree_in_background,
    BackgroundLoadable,
};
use paths_data::markers::MarkerCategoryTreeNode;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
};

use super::RenderConfig;

pub mod manager;

pub struct UiRenderer {
    context: Context,
    egui_renderer: egui_directx11::Renderer,
}

impl UiRenderer {
    pub fn new(d3d11_device: &ID3D11Device, egui_context: &Context) -> Self {
        let egui_renderer = egui_directx11::Renderer::new(d3d11_device)
            // TODO: Error handling
            .expect("Could not create egui dx11 renderer");

        Self {
            context: egui_context.clone(),
            egui_renderer,
        }
    }

    pub fn render(
        &mut self,
        config: &RenderConfig,
        input_manager: &mut InputManager,
        d3d11_device_context: &ID3D11DeviceContext,
        d3d11_render_target_view: &ID3D11RenderTargetView,
    ) {
        let mumble_link = unsafe { get_mumble_link() };

        let input = RawInput {
            events: input_manager.get_events(),

            focused: mumble_link.Context.IsGameFocused() > 0,

            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(config.screen_width, config.screen_height),
            )),

            // TODO: Is this needed?
            time: None,

            ..Default::default()
        };

        let output = self.context.run(input, |ctx| {
            Window::new("Paths")
                .max_height(config.screen_height / 2.0)
                .show(ctx, |ui| {
                    marker_category_overview(ui);

                    ScrollArea::vertical().show(ui, |ui| {
                        marker_category_tree(ui);
                    })
                });
        });

        self.egui_renderer
            .render(
                d3d11_device_context,
                d3d11_render_target_view,
                &self.context,
                egui_directx11::split_output(output).0,
                1.0,
            )
            .expect("Could not render ui");
    }
}

fn marker_category_overview(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let is_loading =
            if let BackgroundLoadable::Loaded(tree) = unsafe { get_marker_category_tree() } {
                ui.label(format!(
                    "Loaded {} pack{} with {} route{}",
                    tree.pack_count,
                    if tree.pack_count == 1 { "" } else { "s" },
                    tree.trail_count,
                    if tree.trail_count == 1 { "" } else { "s" }
                ));

                false
            } else {
                ui.label("Loading markers...");

                true
            };

        let reload_button = &ui.add_enabled(!is_loading, Button::new("Reload"));

        if reload_button.clicked() {
            unsafe { load_marker_category_tree_in_background() };
        }

        if is_loading {
            ui.spinner();
        }
    });
}

fn marker_category_tree(ui: &mut Ui) {
    if let BackgroundLoadable::Loaded(tree) = unsafe { get_marker_category_tree() } {
        let root = tree.tree.root().expect("Tree has no root node");

        marker_category_nodes::<&str>(ui, &root, &vec![]);
    }
}

fn marker_category_nodes<P: AsRef<str>>(
    ui: &mut Ui,
    parent: &MarkerCategoryTreeNode<'_>,
    parent_path: &[P],
) {
    for child in parent.children() {
        let category = child.data();

        let trail_count: usize = child
            .traverse_pre_order()
            .map(|n| n.data().trails.len())
            .sum();

        if trail_count == 0 && !category.is_separator {
            continue;
        }

        let is_not_expandable = category.is_separator || child.children().count() == 0;

        let row = |ui: &mut Ui| {
            if category.is_separator {
                ui.label(&category.label);
            } else {
                let checkbox = &ui.checkbox(
                    &mut category.is_selected.borrow_mut(),
                    format!("{} ({})", &category.label, &trail_count),
                );

                if checkbox.changed() {
                    for child in child.traverse_pre_order().skip(1) {
                        *child.data().is_selected.borrow_mut() = *category.is_selected.borrow();
                    }

                    // TODO: Trigger loading the selected trails
                }
            }
        };

        if is_not_expandable {
            row(ui);
        } else {
            let mut path = Vec::from_iter(parent_path.iter().map(|s| s.as_ref()));
            path.push(&category.identifier);

            let id = ui.make_persistent_id(&path);

            CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    row(ui);
                })
                .body(|ui| {
                    marker_category_nodes(ui, &child, &path);
                });
        }
    }
}
