use std::{cell::RefCell, rc::Rc};

use egui::{Context, Event, Pos2, RawInput, Rect, Vec2};
use paths_core::{loadable::BackgroundLoadable, ui::render_ui};
use paths_data::markers::MarkerCategoryTree;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
};

use super::RenderConfig;

pub struct UiRenderer {
    config: Rc<RefCell<RenderConfig>>,
    context: Context,
    egui_renderer: egui_directx11::Renderer,

    d3d11_device_context: Rc<ID3D11DeviceContext>,
}

impl UiRenderer {
    pub fn new(
        config: Rc<RefCell<RenderConfig>>,
        d3d11_device: &ID3D11Device,
        d3d11_device_context: Rc<ID3D11DeviceContext>,
        egui_context: Context,
    ) -> Self {
        let egui_renderer = egui_directx11::Renderer::new(d3d11_device)
            // TODO: Error handling
            .expect("Could not create egui dx11 renderer");

        Self {
            config,
            context: egui_context,
            egui_renderer,

            d3d11_device_context,
        }
    }

    pub fn render<ReloadTreeFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
        &mut self,
        events: Vec<Event>,
        d3d11_render_target_view: &ID3D11RenderTargetView,

        mumble_data: &api::Mumble_Data,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        reload_tree: ReloadTreeFn,
        update_marker_settings: UpdateMarkerSettingsFn,
    ) {
        let input = RawInput {
            events,

            focused: mumble_data.Context.IsGameFocused() > 0,

            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(
                    self.config.borrow().screen_width,
                    self.config.borrow().screen_height,
                ),
            )),

            // TODO: Is this needed?
            time: None,

            ..Default::default()
        };

        let output = self.context.run(input, |ctx| {
            render_ui(
                self.config.borrow().screen_width,
                self.config.borrow().screen_height,
                ctx,
                tree,
                reload_tree,
                update_marker_settings,
            );
        });

        self.egui_renderer
            .render(
                &self.d3d11_device_context,
                d3d11_render_target_view,
                &self.context,
                egui_directx11::split_output(output).0,
                1.0,
            )
            .expect("Could not render ui");
    }
}
