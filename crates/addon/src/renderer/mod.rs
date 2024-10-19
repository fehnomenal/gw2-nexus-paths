mod map;
mod shaders;
mod ui;
mod world;

use std::{cell::OnceCell, mem::MaybeUninit, rc::Rc, sync::Mutex};

use egui::{Context, Event};
use log_err::{LogErrOption, LogErrResult};
use map::MapRenderer;
use paths_core::{loadable::BackgroundLoadable, markers::ActiveMarkerCategories};
use paths_data::markers::MarkerCategoryTree;
use paths_types::settings::Settings;
use ui::UiRenderer;
use windows::Win32::Graphics::{
    Direct2D::{
        Common::{D2D1_ALPHA_MODE_IGNORE, D2D1_PIXEL_FORMAT},
        D2D1CreateFactory, ID2D1Bitmap1, ID2D1DeviceContext, ID2D1Factory1,
        D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1,
        D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    },
    Direct3D11::{
        ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D, D3D11_VIEWPORT,
    },
    Dxgi::{Common::DXGI_FORMAT_R8G8B8A8_UNORM, IDXGIDevice, IDXGISurface, IDXGISwapChain},
};
use world::WorldRenderer;

use api::{Mumble_EUIScale, Mumble_EUIScale_Large, Mumble_EUIScale_Larger, Mumble_EUIScale_Small};

pub struct Renderer<'a> {
    pub config: Rc<Mutex<RenderConfig>>,
    swap_chain: &'a IDXGISwapChain,

    map_renderer: MapRenderer,
    world_renderer: WorldRenderer,
    ui_renderer: UiRenderer,

    d2d1_device_context: Rc<ID2D1DeviceContext>,
    d2d1_render_target: Rc<OnceCell<ID2D1Bitmap1>>,

    d3d11_device: Rc<ID3D11Device>,
    d3d11_device_context: Rc<ID3D11DeviceContext>,
    d3d11_render_target_view: Rc<OnceCell<ID3D11RenderTargetView>>,
}

impl<'a> Renderer<'a> {
    pub unsafe fn new(
        config: Rc<Mutex<RenderConfig>>,
        swap_chain: &'a IDXGISwapChain,
        egui_context: Context,
    ) -> Self {
        let dxgi_device = swap_chain
            .GetDevice::<IDXGIDevice>()
            .log_expect("could not get dxgi device from swap chain");

        let d2d1_factory =
            D2D1CreateFactory::<ID2D1Factory1>(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .map(Rc::new)
                .log_expect("could not create d2d1 factory");

        let d2d1_device = d2d1_factory
            .CreateDevice(&dxgi_device)
            .map(Rc::new)
            .log_expect("could not create d2d1 device");

        let d2d1_device_context = d2d1_device
            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .map(Rc::new)
            .log_expect("could not create d2d1 device context");

        let d2d1_render_target = Rc::new(OnceCell::new());

        let d3d11_device = swap_chain
            .GetDevice::<ID3D11Device>()
            .map(Rc::new)
            .log_expect("could not get d3d11 device from swap chain");

        let d3d11_device_context = d3d11_device
            .GetImmediateContext()
            .map(Rc::new)
            .log_expect("could not get d3d11 device context");

        let d3d11_render_target_view = Rc::new(OnceCell::new());

        let map_renderer = MapRenderer::new(
            config.clone(),
            d2d1_factory.clone(),
            d2d1_device_context.clone(),
        );
        let world_renderer = WorldRenderer::new(d3d11_device_context.clone());
        let ui_renderer = UiRenderer::new(
            config.clone(),
            &d3d11_device,
            d3d11_device_context.clone(),
            d3d11_render_target_view.clone(),
            egui_context,
        );

        Self {
            config,
            swap_chain,

            map_renderer,
            world_renderer,
            ui_renderer,

            d2d1_device_context,
            d2d1_render_target,

            d3d11_device,
            d3d11_device_context,
            d3d11_render_target_view,
        }
    }

    pub fn rebuild_render_targets(&mut self) {
        drop(Rc::get_mut(&mut self.d2d1_render_target).take());
        drop(Rc::get_mut(&mut self.d3d11_render_target_view).take());
    }

    unsafe fn init_d2d1_render_target(&mut self) {
        let render_target: &ID2D1Bitmap1 = self.d2d1_render_target.get_or_init(|| {
            let bb = self
                .swap_chain
                .GetBuffer::<IDXGISurface>(0)
                .log_expect("could not get back buffer");

            self.d2d1_device_context
                .CreateBitmapFromDxgiSurface(
                    &bb,
                    Some(&D2D1_BITMAP_PROPERTIES1 {
                        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: DXGI_FORMAT_R8G8B8A8_UNORM,
                            alphaMode: D2D1_ALPHA_MODE_IGNORE,
                        },
                        ..Default::default()
                    }),
                )
                .log_expect("could not create d2d1 bitmap")
        });

        self.d2d1_device_context.SetTarget(render_target);
    }

    unsafe fn init_d3d11_render_target(&mut self) {
        let render_target_view = self.d3d11_render_target_view.get_or_init(|| {
            let (screen_width, screen_height) = {
                let config = self.config.lock().log_unwrap();

                (config.screen_width, config.screen_height)
            };

            let viewport = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: screen_width,
                Height: screen_height,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };

            self.d3d11_device_context.RSSetViewports(Some(&[viewport]));

            let bb = self
                .swap_chain
                .GetBuffer::<ID3D11Texture2D>(0)
                .log_expect("could not get back buffer");

            let mut render_target_view = MaybeUninit::uninit();

            self.d3d11_device
                .CreateRenderTargetView(&bb, None, Some(render_target_view.as_mut_ptr()))
                .log_expect("could not create render target view");

            render_target_view
                .assume_init()
                .log_expect("render target view is empty???")
        });

        self.d3d11_device_context
            .OMSetRenderTargets(Some(&[Some(render_target_view.to_owned())]), None);
    }

    pub unsafe fn render_map(
        &mut self,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories,
        settings: &Settings,
    ) {
        self.init_d2d1_render_target();

        self.map_renderer
            .render(mumble_data, active_marker_categories, settings);
    }

    pub unsafe fn render_world(&mut self) {
        self.init_d3d11_render_target();

        self.world_renderer.render();
    }

    pub unsafe fn render_ui<ReloadFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
        &mut self,
        events: Vec<Event>,

        mumble_data: &api::Mumble_Data,
        tree: &BackgroundLoadable<MarkerCategoryTree>,
        reload: ReloadFn,
        update_marker_settings: UpdateMarkerSettingsFn,
    ) {
        self.init_d3d11_render_target();

        self.ui_renderer
            .render(events, mumble_data, tree, reload, update_marker_settings);
    }
}

pub struct RenderConfig {
    pub screen_width: f32,
    pub screen_height: f32,
    pub half_screen_width: f32,
    pub half_screen_height: f32,
    pub ui_scale_factor: f32,
}

impl RenderConfig {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            half_screen_width: screen_width / 2.0,
            half_screen_height: screen_height / 2.0,
            ui_scale_factor: 1.0,
        }
    }

    pub fn update_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
        self.half_screen_width = width / 2.0;
        self.half_screen_height = height / 2.0;
    }

    pub fn update_ui_size(&mut self, ui_size: Mumble_EUIScale) {
        self.ui_scale_factor = if ui_size == Mumble_EUIScale_Small {
            0.9
        } else if ui_size == Mumble_EUIScale_Large {
            1.11
        } else if ui_size == Mumble_EUIScale_Larger {
            1.22
        } else {
            1.0
        };
    }
}
