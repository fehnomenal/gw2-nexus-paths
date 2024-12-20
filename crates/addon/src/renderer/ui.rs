use std::{cell::OnceCell, rc::Rc, sync::Mutex};

use egui::{Context, Event, Pos2, RawInput, Rect, Vec2};
use log_err::{LogErrOption, LogErrResult};
use paths_core::{
    loadable::BackgroundLoadable,
    markers::{ActiveMarkerCategories, MarkerCategoryTree},
    settings::Settings,
    ui::{UiActions, UiState},
};
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
};

use super::RenderConfig;

pub struct UiRenderer {
    config: Rc<Mutex<RenderConfig>>,
    context: Context,
    egui_renderer: egui_directx11::Renderer,

    d3d11_device_context: Rc<ID3D11DeviceContext>,
    d3d11_render_target_view: Rc<OnceCell<ID3D11RenderTargetView>>,
}

impl UiRenderer {
    pub fn new(
        config: Rc<Mutex<RenderConfig>>,
        d3d11_device: &ID3D11Device,
        d3d11_device_context: Rc<ID3D11DeviceContext>,
        d3d11_render_target_view: Rc<OnceCell<ID3D11RenderTargetView>>,
        egui_context: Context,
    ) -> Self {
        let egui_renderer = egui_directx11::Renderer::new(d3d11_device)
            .log_expect("could not create egui dx11 renderer");

        Self {
            config,
            context: egui_context,
            egui_renderer,

            d3d11_device_context,
            d3d11_render_target_view,
        }
    }

    pub fn render<A: UiActions>(
        &mut self,
        state: &mut UiState<A>,
        events: Vec<Event>,

        mumble_data: &api::Mumble_Data,
        nexus_link_data: &api::NexusLinkData,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        settings: &mut Settings,
        active_marker_categories: &ActiveMarkerCategories,
    ) {
        let (screen_width, screen_height) = {
            let config = self.config.lock().log_unwrap();

            (config.screen_width, config.screen_height)
        };

        let input = RawInput {
            events,

            focused: mumble_data.Context.IsGameFocused() > 0,

            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(screen_width, screen_height),
            )),

            // TODO: Is this needed?
            time: None,

            ..Default::default()
        };

        let output = self.context.run(input, |ctx| {
            state.render(
                screen_width,
                screen_height,
                ctx,
                tree,
                nexus_link_data.IsGameplay,
                settings,
                active_marker_categories,
            );
        });

        self.egui_renderer
            .render(
                &self.d3d11_device_context,
                self.d3d11_render_target_view
                    .get()
                    .log_expect("did not initialize render target view"),
                &self.context,
                egui_directx11::split_output(output).0,
                1.0,
            )
            .log_expect("could not render ui");
    }
}
