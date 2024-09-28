use std::collections::HashMap;

use egui::{Color32, Rgba};
use paths_core::markers::ActiveTrail;
use paths_data::maps::MAP_TO_WORLD_TRANSFORMATION_MATRICES;
use paths_types::settings::Settings;
use windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{
        Common::{D2D1_COLOR_F, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_OPEN, D2D_POINT_2F},
        ID2D1PathGeometry1, ID2D1SolidColorBrush,
    },
};

use super::MapRenderer;

impl MapRenderer {
    pub unsafe fn draw_trails(
        &mut self,
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

            let brush = self
                .d2d1_device_context
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
        &mut self,
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

        let path = self.trail_path_cache.entry(trail.hash).or_insert_with(|| {
            let path = self
                .d2d1_factory
                .CreatePathGeometry()
                // TODO: Error handling
                .expect("Could not create path geometry");

            let sink = path
                .Open()
                // TODO: Error handling
                .expect("Could not open path geometry");

            sink.BeginFigure(D2D_POINT_2F::default(), D2D1_FIGURE_BEGIN_HOLLOW);

            // TODO: Draw the first point specially.

            sink.AddLines(
                &trail
                    .points
                    .iter()
                    .map(|point| D2D_POINT_2F {
                        x: point.x,
                        y: point.y,
                    })
                    .collect::<Vec<_>>(),
            );

            sink.EndFigure(D2D1_FIGURE_END_OPEN);
            sink.Close()
                // TODO: Error handling
                .expect("Could not close path geometry");

            path
        });

        self.d2d1_device_context
            .SetTransform(&(map_to_world_transformation * world_to_screen_transformation));

        self.d2d1_device_context.DrawGeometry(
            path as &ID2D1PathGeometry1,
            brush,
            trail.trail_width.unwrap_or(default_trail_width),
            None,
        );
    }
}
