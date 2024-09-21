use paths_data::maps::MAP_TO_WORLD_TRANSFORMATION_MATRICES;
use windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{
        Common::{D2D1_COLOR_F, D2D_POINT_2F, D2D_RECT_F},
        ID2D1DeviceContext, ID2D1SolidColorBrush, D2D1_ELLIPSE,
    },
};

use super::RenderConfig;

pub struct MapRenderer<'a> {
    config: &'a RenderConfig,

    red_brush: ID2D1SolidColorBrush,
}

impl<'a> MapRenderer<'a> {
    pub unsafe fn new(config: &'a RenderConfig, device_context: &ID2D1DeviceContext) -> Self {
        // TODO: Which color(s)?
        let red_brush = device_context
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
            .expect("Could not create red brush");

        Self { config, red_brush }
    }

    pub unsafe fn render(
        &self,
        device_context: &ID2D1DeviceContext,
        mumble_data: &api::Mumble_Data,
    ) {
        let world_to_screen_transformation = self.get_world_to_screen_transformation(mumble_data);

        device_context.BeginDraw();

        device_context.SetTransform(&world_to_screen_transformation);

        let waypoint = D2D_POINT_2F { x: 40165.6, y: 31856.7 };

        device_context.DrawEllipse(
            &D2D1_ELLIPSE {
                point: waypoint,
                radiusX: 10.0,
                radiusY: 10.0,
            },
            &self.red_brush,
            5.0,
            None,
        );

        let waypoint = D2D_POINT_2F { x: 41275.2, y: 31983.9 } ;

        device_context.DrawEllipse(
            &D2D1_ELLIPSE {
                point: waypoint,
                radiusX: 5.0,
                radiusY: 5.0,
            },
            &self.red_brush,
            2.5,
            None,
        );

        let map_to_world_transformation = MAP_TO_WORLD_TRANSFORMATION_MATRICES
            .get(&54)
            .expect("Could not find map dimensions");

        device_context
            .SetTransform(&(map_to_world_transformation * world_to_screen_transformation));

        let waypoint_relative = D2D_POINT_2F {
            x: 582.412,
            y: 165.874,
        };

        device_context.DrawEllipse(
            &D2D1_ELLIPSE {
                point: waypoint_relative,
                radiusX: 10.0,
                radiusY: 10.0,
            },
            &self.red_brush,
            5.0,
            None,
        );

        device_context
            .EndDraw(None, None)
            // TODO: Error handling
            .expect("Could not end drawing");

        device_context.SetTransform(&Matrix3x2::identity());
    }

    fn get_world_to_screen_transformation(&self, mumble_data: &api::Mumble_Data) -> Matrix3x2 {
        let map_scale = {
            let compass_scale = mumble_data.Context.Compass.Scale;

            compass_scale / self.config.ui_scale_factor
        };

        // Move map center to 0,0
        let translate_map_center = Matrix3x2::translation(
            -mumble_data.Context.Compass.Center.X,
            -mumble_data.Context.Compass.Center.Y,
        );

        let scale = Matrix3x2 {
            M11: 1.0 / map_scale,
            M12: 0.0,
            M21: 0.0,
            M22: 1.0 / map_scale,
            M31: 0.0,
            M32: 0.0,
        };

        let translate_screen_center = if mumble_link.Context.IsMapOpen() > 0 {
            // Move map center to screen center
            Matrix3x2::translation(
                self.config.half_screen_width,
                self.config.half_screen_height,
            )
        } else {
            // Move map center to compass center
            let rect = self.get_compass_rect(&mumble_data.Context);

            Matrix3x2::translation(
                (rect.right + rect.left) / 2.0,
                (rect.bottom + rect.top) / 2.0,
            )
        };

        translate_map_center * scale * translate_screen_center
    }

    fn get_compass_rect(&self, mumble_context: &api::Mumble_Context) -> D2D_RECT_F {
        let compass_width = mumble_context.Compass.Width as f32;
        let compass_height = mumble_context.Compass.Height as f32;

        let left = self.config.screen_width - (compass_width * self.config.ui_scale_factor);
        let right = self.config.screen_width;

        let (top, bottom) = if mumble_context.IsCompassTopRight() > 0 {
            let top = 1.0;
            let bottom = compass_height * self.config.ui_scale_factor + 1.0;

            (top, bottom)
        } else {
            const DISTANCE_FROM_BOTTOM: f32 = 37.0;

            let scaled_distance = DISTANCE_FROM_BOTTOM * self.config.ui_scale_factor;

            let top = self.config.screen_height
                - compass_height * self.config.ui_scale_factor
                - scaled_distance;
            let bottom = self.config.screen_height - scaled_distance;

            (top, bottom)
        };

        D2D_RECT_F {
            left,
            top,
            right,
            bottom,
        }
    }
}
