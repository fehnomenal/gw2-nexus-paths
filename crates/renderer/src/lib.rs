mod map;
mod shaders;
mod ui;
mod world;

use std::{cell::RefCell, mem::MaybeUninit, rc::Rc};

use egui::{Context, Event, Rgba};
use map::MapRenderer;
use paths_core::{loadable::BackgroundLoadable, markers::ActiveMarkerCategories};
use paths_data::markers::MarkerCategoryTree;
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
    pub config: Rc<RefCell<RenderConfig>>,
    swap_chain: &'a IDXGISwapChain,

    map_renderer: MapRenderer,
    world_renderer: WorldRenderer,
    ui_renderer: UiRenderer,

    d2d1_device_context: ID2D1DeviceContext,
    d2d1_render_target: Option<ID2D1Bitmap1>,

    d3d11_device: ID3D11Device,
    d3d11_device_context: ID3D11DeviceContext,
    d3d11_render_target_view: Option<ID3D11RenderTargetView>,
}

impl<'a> Renderer<'a> {
    pub unsafe fn new(
        config: Rc<RefCell<RenderConfig>>,
        swap_chain: &'a IDXGISwapChain,
        egui_context: Context,
    ) -> Self {
        let dxgi_device = swap_chain
            .GetDevice::<IDXGIDevice>()
            // TODO: Error handling
            .expect("Could not get dxgi device from swap chain");

        let d2d1_device =
            D2D1CreateFactory::<ID2D1Factory1>(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .and_then(|factory| factory.CreateDevice(&dxgi_device))
                // TODO: Error handling
                .expect("Could not create d2d1 device");

        let d2d1_device_context = d2d1_device
            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            // TODO: Error handling
            .expect("Could not create d2d1 device context");

        let d3d11_device = swap_chain
            .GetDevice::<ID3D11Device>()
            // TODO: Error handling
            .expect("Could not get d3d11 device from swap chain");

        let d3d11_device_context = d3d11_device
            .GetImmediateContext()
            // TODO: Error handling
            .expect("Could not get d3d11 device context");

        let map_renderer = MapRenderer::new(config.clone(), &d2d1_device_context.clone());
        let world_renderer = WorldRenderer::new();
        let ui_renderer = UiRenderer::new(config.clone(), &d3d11_device, egui_context);

        Self {
            config,
            swap_chain,

            map_renderer,
            world_renderer,
            ui_renderer,

            d2d1_device_context,
            d2d1_render_target: None,

            d3d11_device,
            d3d11_device_context,
            d3d11_render_target_view: None,
        }
    }

    pub fn rebuild_render_targets(&mut self) {
        drop(self.d2d1_render_target.take());
        drop(self.d3d11_render_target_view.take());
    }

    unsafe fn init_d2d1_render_target(&mut self) -> &ID2D1Bitmap1 {
        let render_target = self.d2d1_render_target.get_or_insert_with(|| {
            let bb = self
                .swap_chain
                .GetBuffer::<IDXGISurface>(0)
                // TODO: Error handling
                .expect("Could not get back buffer");

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
                // TODO: Error handling
                .expect("Could not create d2d1 bitmap")
        });

        self.d2d1_device_context.SetTarget(&*render_target);

        render_target
    }

    unsafe fn init_d3d11_render_target(&mut self) -> &ID3D11RenderTargetView {
        let render_target_view = self.d3d11_render_target_view.get_or_insert_with(|| {
            let viewport = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: self.config.borrow().screen_width,
                Height: self.config.borrow().screen_height,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };

            self.d3d11_device_context.RSSetViewports(Some(&[viewport]));

            let bb = self
                .swap_chain
                .GetBuffer::<ID3D11Texture2D>(0)
                // TODO: Error handling
                .expect("Could not get back buffer");

            let mut render_target_view = MaybeUninit::uninit();

            self.d3d11_device
                .CreateRenderTargetView(&bb, None, Some(render_target_view.as_mut_ptr()))
                // TODO: Error handling
                .expect("Could not create render target view");

            render_target_view
                .assume_init()
                // TODO: Error handling
                .expect("Render target view is empty???")
        });

        self.d3d11_device_context
            .OMSetRenderTargets(Some(&[Some(render_target_view.clone())]), None);

        render_target_view
    }

    pub unsafe fn render_map(
        &mut self,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories<Rgba>,
    ) {
        self.init_d2d1_render_target();

        self.map_renderer.render(
            &self.d2d1_device_context,
            mumble_data,
            active_marker_categories,
        );
    }

    pub unsafe fn render_world(&mut self) {
        self.init_d3d11_render_target();

        self.world_renderer.render(&self.d3d11_device_context);
    }

    pub unsafe fn render_ui<ReloadTreeFn: Fn(), UpdateMarkerSettingsFn: Fn()>(
        &mut self,
        events: Vec<Event>,

        mumble_data: &api::Mumble_Data,
        tree: &BackgroundLoadable<MarkerCategoryTree<Rgba>>,
        reload_tree: ReloadTreeFn,
        update_marker_settings: UpdateMarkerSettingsFn,
    ) {
        let render_target_view = self.init_d3d11_render_target().clone();

        self.ui_renderer.render(
            events,
            &self.d3d11_device_context,
            &render_target_view,
            mumble_data,
            tree,
            reload_tree,
            update_marker_settings,
        );
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
