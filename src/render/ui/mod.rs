use egui::{Context, Pos2, RawInput, Rect, Vec2, Window};
use manager::InputManager;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
};

use crate::state::get_mumble_link;

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
            Window::new("Paths").show(ctx, |ui| ui.label("Paths option window"));
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