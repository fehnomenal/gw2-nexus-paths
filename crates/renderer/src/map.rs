use std::{cell::RefCell, collections::HashMap, rc::Rc};

use egui::{Color32, Rgba};
use paths_core::markers::{ActiveMarkerCategories, ActiveTrail};
use paths_data::maps::MAP_TO_WORLD_TRANSFORMATION_MATRICES;
use paths_types::settings::Settings;
use windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{
        Common::{D2D1_COLOR_F, D2D_POINT_2F, D2D_RECT_F},
        ID2D1DeviceContext, ID2D1SolidColorBrush,
    },
};

use super::RenderConfig;

pub struct MapRenderer {
    config: Rc<RefCell<RenderConfig>>,
}

impl MapRenderer {
    pub unsafe fn new(config: Rc<RefCell<RenderConfig>>) -> Self {
        Self { config }
    }

    pub unsafe fn render(
        &self,
        device_context: &ID2D1DeviceContext,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories<Rgba>,
        settings: &Settings,
    ) {
        device_context.BeginDraw();

        if mumble_data.Context.IsMapOpen() > 0 {
            self.draw_map(
                device_context,
                mumble_data,
                active_marker_categories,
                settings,
            );
        } else {
            self.draw_compass(
                device_context,
                mumble_data,
                active_marker_categories,
                settings,
            );
        }

        device_context
            .EndDraw(None, None)
            // TODO: Error handling
            .expect("Could not end drawing");

        device_context.SetTransform(&Matrix3x2::identity());
    }

    unsafe fn draw_map(
        &self,
        device_context: &ID2D1DeviceContext,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories<Rgba>,
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
            device_context,
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
        &self,
        device_context: &ID2D1DeviceContext,
        mumble_data: &api::Mumble_Data,
        active_marker_categories: &ActiveMarkerCategories<Rgba>,
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

        self.draw_trails(
            device_context,
            &world_to_screen_transformation,
            &active_marker_categories
                .active_trails()
                .iter()
                .map(|trail| (&active_marker_categories.active_map_id, trail))
                .collect::<Vec<_>>(),
            settings,
        );
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

    unsafe fn draw_trails(
        &self,
        device_context: &ID2D1DeviceContext,
        world_to_screen_transformation: &Matrix3x2,
        trails: &[(&u32, &ActiveTrail<Rgba>)],
        settings: &Settings,
    ) {
        // Group trails by color.
        let mut trails_by_color_key = HashMap::<String, Vec<_>>::new();
        let mut colors = HashMap::new();

        for (map_id, trail) in trails {
            let color = trail
                .trail_color
                .unwrap_or_else(|| settings.default_trail_color);
            let color_key = Color32::from(color).to_hex();

            trails_by_color_key
                .entry(color_key.clone())
                .or_default()
                .push((map_id, trail));
            colors.insert(color_key, color);
        }

        for (color_key, trails) in trails_by_color_key {
            let color = colors
                .get(&color_key)
                .unwrap_or_else(|| &settings.default_trail_color);

            let brush = device_context
                .CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: color.r(),
                        g: color.g(),
                        b: color.b(),
                        a: color.a(),
                    },
                    None,
                )
                // TODO: Error handling
                .expect("Could not create trail brush");

            for (map_id, trail) in trails {
                self.draw_trail(
                    device_context,
                    world_to_screen_transformation,
                    map_id,
                    trail,
                    &brush,
                    settings.default_trail_width,
                );
            }
        }
    }

    unsafe fn draw_trail(
        &self,
        device_context: &ID2D1DeviceContext,
        world_to_screen_transformation: &Matrix3x2,
        map_id: &u32,
        trail: &ActiveTrail<Rgba>,
        brush: &ID2D1SolidColorBrush,
        default_trail_width: f32,
    ) {
        if trail.points.len() < 2 {
            return;
        }

        let Some(map_to_world_transformation) = MAP_TO_WORLD_TRANSFORMATION_MATRICES.get(map_id)
        else {
            return;
        };

        device_context
            .SetTransform(&(map_to_world_transformation * world_to_screen_transformation));

        for line in trail.points.windows(2) {
            // TODO: Only draw if at least one point is visible.
            // TODO: Draw the first point specially.

            // SAFETY: slice::windows(N) is guaranteed to yield a slice with exactly N elements.
            let from = line.get_unchecked(0);
            let to = line.get_unchecked(1);

            device_context.DrawLine(
                D2D_POINT_2F {
                    x: from.x,
                    y: from.y,
                },
                D2D_POINT_2F { x: to.x, y: to.y },
                brush,
                trail.trail_width.unwrap_or(default_trail_width),
                None,
            );
        }
    }
}
