use std::collections::HashMap;

use egui::Color32;
use log_err::LogErrResult;
use paths_core::markers::ActiveTrail;
use paths_data::{maps::MAP_TO_WORLD_TRANSFORMATION_MATRICES, markers::simplify_line_string};
use paths_types::settings::Settings;
use windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{
        Common::{D2D1_COLOR_F, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_OPEN, D2D_POINT_2F},
        ID2D1SolidColorBrush, D2D1_CAP_STYLE_ROUND, D2D1_LINE_JOIN_ROUND,
        D2D1_STROKE_STYLE_PROPERTIES1,
    },
};

use super::MapRenderer;

impl MapRenderer {
    pub unsafe fn draw_trails(
        &mut self,
        world_to_screen_transformation: &Matrix3x2,
        trails: &[(&u32, &ActiveTrail)],
        settings: &Settings,
    ) {
        // Group trails by color.
        let mut trails_by_color_key = HashMap::<String, Vec<_>>::new();
        let mut colors = HashMap::new();

        for (map_id, trail) in trails {
            let color = trail
                .trail_color
                .unwrap_or_else(|| settings.default_trail_color);
            let color_key = Color32::from(*color).to_hex();

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
                .log_expect("could not create trail brush");

            for (map_id, trail) in trails {
                self.draw_trail(
                    world_to_screen_transformation,
                    map_id,
                    trail,
                    &brush,
                    settings,
                );
            }
        }
    }

    unsafe fn draw_trail(
        &mut self,
        world_to_screen_transformation: &Matrix3x2,
        map_id: &u32,
        trail: &ActiveTrail,
        brush: &ID2D1SolidColorBrush,
        settings: &Settings,
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
                .log_expect("could not create path geometry");

            let sink = path.Open().log_expect("could not open path geometry");

            sink.BeginFigure(D2D_POINT_2F::default(), D2D1_FIGURE_BEGIN_HOLLOW);

            // TODO: Draw the first point specially.

            let points = simplify_line_string(trail.points, *settings.trail_simplify_epsilon);

            sink.AddLines(
                &points
                    .iter()
                    .map(|point| D2D_POINT_2F {
                        x: point.x,
                        y: point.y,
                    })
                    .collect::<Vec<_>>(),
            );

            sink.EndFigure(D2D1_FIGURE_END_OPEN);

            sink.Close().log_expect("could not close path geometry");

            path
        });

        let stroke_style = self.trail_stroke_style.get_or_insert_with(|| {
            self.d2d1_factory
                .CreateStrokeStyle(
                    &D2D1_STROKE_STYLE_PROPERTIES1 {
                        startCap: D2D1_CAP_STYLE_ROUND,
                        endCap: D2D1_CAP_STYLE_ROUND,
                        lineJoin: D2D1_LINE_JOIN_ROUND,
                        ..Default::default()
                    },
                    None,
                )
                .log_expect("could not create trail stroke style")
        });

        self.d2d1_device_context
            .SetTransform(&(map_to_world_transformation * world_to_screen_transformation));

        self.d2d1_device_context.DrawGeometry(
            path as &_,
            brush,
            *trail.trail_width.unwrap_or(settings.default_trail_width),
            stroke_style as &_,
        );
    }
}
