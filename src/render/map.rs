use std::cell::LazyCell;

use windows::{
    core::Interface,
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::{
        Direct2D::{
            Common::{D2D1_ALPHA_MODE_IGNORE, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
            D2D1CreateFactory, ID2D1DeviceContext, ID2D1Factory1, ID2D1SolidColorBrush,
            D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1,
            D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
        },
        Direct3D11::ID3D11Device,
        Dxgi::{Common::DXGI_FORMAT_R8G8B8A8_UNORM, IDXGIDevice, IDXGISurface, IDXGISwapChain},
    },
};

use crate::{
    data::{get_map_dimensions, Point2},
    state::{get_mumble_link, get_render_state},
};

pub struct MapRenderer {
    swap_chain: IDXGISwapChain,
    device_context: ID2D1DeviceContext,

    red_brush: ID2D1SolidColorBrush,
}

impl MapRenderer {
    pub fn new(swap_chain: &IDXGISwapChain, d3d11_device: &ID3D11Device) -> Self {
        let factory = unsafe {
            D2D1CreateFactory::<ID2D1Factory1>(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                // TODO: Error handling
                .expect("Could not create d2d1 factory")
        };

        let dxgi_device = (*d3d11_device)
            .cast::<IDXGIDevice>()
            // TODO: Error handling
            .expect("Could not obtain underlying dxgi device");

        let device = unsafe {
            factory
                .CreateDevice(&dxgi_device)
                // TODO: Error handling
                .expect("Could not create d2d1 device")
        };

        let device_context = unsafe {
            device
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
                // TODO: Error handling
                .expect("Could not create d2d1 device context")
        };

        // TODO: Which color(s)?
        let red_brush = unsafe {
            device_context
                .CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    None,
                )
                // TODO: Error handling
                .expect("Could not create red brush")
        };

        Self {
            swap_chain: swap_chain.clone(),
            device_context,
            red_brush,
        }
    }

    unsafe fn init_render_target_view(&self) {
        let bb = self
            .swap_chain
            .GetBuffer::<IDXGISurface>(0)
            // TODO: Error handling
            .expect("Could not get back buffer");

        let render_target = self
            .device_context
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
            .expect("Could not create d2d1 bitmap");

        self.device_context.SetTarget(&render_target);
    }

    pub unsafe fn render_path_on_map(&self) {
        self.init_render_target_view();

        let world_to_screen_transformation =
            LazyCell::new(|| Self::get_world_to_screen_transformation());

        self.device_context.BeginDraw();

        self.device_context
            .SetTransform(&*world_to_screen_transformation);

        let waypoint = Point2::new(40165.6, 31856.7);

        self.device_context.DrawEllipse(
            &D2D1_ELLIPSE {
                point: waypoint.as_d2d_point_2f(),
                radiusX: 10.0,
                radiusY: 10.0,
            },
            &self.red_brush,
            5.0,
            None,
        );

        let waypoint = Point2::new(41275.2, 31983.9);

        self.device_context.DrawEllipse(
            &D2D1_ELLIPSE {
                point: waypoint.as_d2d_point_2f(),
                radiusX: 5.0,
                radiusY: 5.0,
            },
            &self.red_brush,
            2.5,
            None,
        );

        let map_dimensions = get_map_dimensions(54).unwrap();

        self.device_context.SetTransform(
            &(map_dimensions.map_to_world_transformation * *world_to_screen_transformation),
        );

        let waypoint_relative = Point2::new(582.412, 165.874);

        self.device_context.DrawEllipse(
            &D2D1_ELLIPSE {
                point: waypoint_relative.as_d2d_point_2f(),
                radiusX: 10.0,
                radiusY: 10.0,
            },
            &self.red_brush,
            5.0,
            None,
        );

        self.device_context
            .EndDraw(None, None)
            // TODO: Error handling
            .expect("Could not end drawing");

        self.device_context.SetTransform(&Matrix3x2::identity());
    }

    fn get_world_to_screen_transformation() -> Matrix3x2 {
        let mumble_link = unsafe { get_mumble_link() };
        let render_state = unsafe { get_render_state() };

        let map_scale = {
            let compass_scale = mumble_link.Context.Compass.Scale;

            compass_scale * render_state.map_scale_factor
        };

        let translate_map_center = Matrix3x2::translation(
            -mumble_link.Context.Compass.Center.X,
            -mumble_link.Context.Compass.Center.Y,
        );

        let scale = Matrix3x2 {
            M11: 1.0 / map_scale,
            M12: 0.0,
            M21: 0.0,
            M22: 1.0 / map_scale,
            M31: 0.0,
            M32: 0.0,
        };

        let translate_screen_center = Matrix3x2::translation(
            render_state.half_screen_width,
            render_state.half_screen_height,
        );

        translate_map_center * scale * translate_screen_center
    }
}
