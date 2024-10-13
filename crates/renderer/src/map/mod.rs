mod trails;

use std::{cell::RefCell, rc::Rc};

use log_err::LogErrResult;
use paths_core::markers::ActiveMarkerCategories;
use paths_types::settings::Settings;
use trails::TrailPathCache;
use windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{
        Common::D2D_RECT_F, ID2D1DeviceContext, ID2D1Factory1, ID2D1StrokeStyle1,
        D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
    },
};

use super::RenderConfig;

pub struct MapRenderer {
    config: Rc<RefCell<RenderConfig>>,

    d2d1_factory: Rc<ID2D1Factory1>,
    d2d1_device_context: Rc<ID2D1DeviceContext>,

    trail_path_cache: TrailPathCache,
    trail_stroke_style: Option<ID2D1StrokeStyle1>,
}

impl MapRenderer {
    pub unsafe fn new(
        config: Rc<RefCell<RenderConfig>>,
        d2d1_factory: Rc<ID2D1Factory1>,
        d2d1_device_context: Rc<ID2D1DeviceContext>,
    ) -> Self {
        Self {
            config,

            d2d1_factory: d2d1_factory.clone(),
            d2d1_device_context,

            trail_path_cache: TrailPathCache::new(d2d1_factory),
            trail_stroke_style: None,
        }
    }

    pub unsafe fn render(
        &mut self,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories,
        settings: &Settings,
    ) {
        self.d2d1_device_context.BeginDraw();

        if mumble_data.Context.IsMapOpen() > 0 {
            self.draw_map(mumble_data, active_marker_categories, settings);
        } else {
            self.draw_compass(mumble_data, active_marker_categories, settings);
        }

        self.d2d1_device_context
            .EndDraw(None, None)
            .log_expect("could not end drawing");

        self.d2d1_device_context
            .SetTransform(&Matrix3x2::identity());
    }

    unsafe fn draw_map(
        &mut self,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories,
        settings: &Settings,
    ) {
        let world_to_screen_transformation = self.get_world_to_screen_transformation(
            &mumble_data.Context.Compass,
            // Move map center to screen center.
            Matrix3x2::translation(
                self.config.borrow().half_screen_width,
                self.config.borrow().half_screen_height,
            ),
        );

        self.draw_trails(
            &world_to_screen_transformation,
            &active_marker_categories
                .all_trails
                .iter()
                .flat_map(|(map_id, trails)| trails.iter().map(move |trail| (map_id, trail)))
                .collect::<Vec<_>>(),
            settings,
        );
    }

    unsafe fn draw_compass(
        &mut self,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories,
        settings: &Settings,
    ) {
        let compass_rect = self.get_compass_rect(&mumble_data.Context);
        let world_to_screen_transformation = self.get_world_to_screen_transformation(
            &mumble_data.Context.Compass,
            // Move map center to compass center.
            Matrix3x2::translation(
                (compass_rect.right + compass_rect.left) / 2.0,
                (compass_rect.bottom + compass_rect.top) / 2.0,
            ),
        );

        if mumble_data.Context.IsCompassRotating() > 0 {
            // TODO: Handle rotating compass (with matrix transformation?).
        }

        self.d2d1_device_context
            .PushAxisAlignedClip(&compass_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);

        self.draw_trails(
            &world_to_screen_transformation,
            &active_marker_categories
                .active_trails()
                .iter()
                .map(|trail| (&active_marker_categories.active_map_id, trail))
                .collect::<Vec<_>>(),
            settings,
        );

        self.d2d1_device_context.PopAxisAlignedClip();
    }

    fn get_world_to_screen_transformation(
        &self,
        compass: &api::Mumble_Compass,
        translate_to_screen: Matrix3x2,
    ) -> Matrix3x2 {
        let map_scale = {
            let compass_scale = compass.Scale;

            compass_scale / self.config.borrow().ui_scale_factor
        };

        // Move map center to 0,0
        let translate_map_center = Matrix3x2::translation(-compass.Center.X, -compass.Center.Y);

        let scale = Matrix3x2 {
            M11: 1.0 / map_scale,
            M12: 0.0,
            M21: 0.0,
            M22: 1.0 / map_scale,
            M31: 0.0,
            M32: 0.0,
        };

        translate_map_center * scale * translate_to_screen
    }

    fn get_compass_rect(&self, mumble_context: &api::Mumble_Context) -> D2D_RECT_F {
        let compass_width = mumble_context.Compass.Width as f32;
        let compass_height = mumble_context.Compass.Height as f32;

        let left = self.config.borrow().screen_width
            - (compass_width * self.config.borrow().ui_scale_factor);
        let right = self.config.borrow().screen_width;

        let (top, bottom) = if mumble_context.IsCompassTopRight() > 0 {
            let top = 1.0;
            let bottom = compass_height * self.config.borrow().ui_scale_factor + 1.0;

            (top, bottom)
        } else {
            const DISTANCE_FROM_BOTTOM: f32 = 37.0;

            let scaled_distance = DISTANCE_FROM_BOTTOM * self.config.borrow().ui_scale_factor;

            let top = self.config.borrow().screen_height
                - compass_height * self.config.borrow().ui_scale_factor
                - scaled_distance;
            let bottom = self.config.borrow().screen_height - scaled_distance;

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
