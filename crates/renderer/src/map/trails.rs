use std::{collections::HashMap, rc::Rc};

use egui::Color32;
use log_err::LogErrResult;
use nalgebra::{distance, Point2};
use paths_core::markers::ActiveTrail;
use paths_data::{maps::MAP_TO_WORLD_TRANSFORMATION_MATRICES, markers::simplify_line_string};
use paths_types::settings::Settings;
use windows::{
    core::Interface,
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{
        Common::{D2D1_COLOR_F, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_OPEN, D2D_POINT_2F},
        ID2D1Factory1, ID2D1Geometry, ID2D1SolidColorBrush, ID2D1StrokeStyle1,
        D2D1_CAP_STYLE_ROUND, D2D1_ELLIPSE, D2D1_LINE_JOIN_ROUND, D2D1_STROKE_STYLE_PROPERTIES1,
    },
};

use super::MapRenderer;

const STARTING_CIRCLE_RADIUS_FACTOR: f32 = 5.0;

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
            let color_key = Color32::from(*trail.trail_color).to_hex();

            trails_by_color_key
                .entry(color_key.clone())
                .or_default()
                .push((map_id, trail));
            colors.insert(color_key, trail.trail_color);
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

        let path = self.trail_path_cache.get_trail_geometry(trail, settings);

        let stroke_style: &ID2D1StrokeStyle1 = self.trail_stroke_style.get_or_insert_with(|| {
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

        self.d2d1_device_context
            .DrawGeometry(path, brush, *trail.trail_width, stroke_style);

        let radius = *trail.trail_width * STARTING_CIRCLE_RADIUS_FACTOR;

        self.d2d1_device_context.DrawEllipse(
            &D2D1_ELLIPSE {
                point: D2D_POINT_2F {
                    x: trail.points[0].x,
                    y: trail.points[0].y,
                },
                radiusX: radius,
                radiusY: radius,
            },
            brush,
            *trail.trail_width,
            stroke_style,
        );

        self.d2d1_device_context.FillEllipse(
            &D2D1_ELLIPSE {
                point: D2D_POINT_2F {
                    x: trail.points[0].x,
                    y: trail.points[0].y,
                },
                radiusX: radius / 2.0,
                radiusY: radius / 2.0,
            },
            brush,
        );
    }
}

pub struct TrailPathCache {
    cache: HashMap<u64, ID2D1Geometry>,
    d2d1_factory: Rc<ID2D1Factory1>,
}

impl TrailPathCache {
    pub fn new(d2d1_factory: Rc<ID2D1Factory1>) -> Self {
        Self {
            cache: HashMap::new(),
            d2d1_factory,
        }
    }

    unsafe fn get_trail_geometry(
        &mut self,
        trail: &ActiveTrail,
        settings: &Settings,
    ) -> &ID2D1Geometry {
        self.cache.entry(trail.hash).or_insert_with(|| {
            let starting_point_circle = {
                let radius = *trail.trail_width * STARTING_CIRCLE_RADIUS_FACTOR;

                D2D1_ELLIPSE {
                    point: D2D_POINT_2F {
                        x: trail.points[0].x,
                        y: trail.points[0].y,
                    },
                    radiusX: radius,
                    radiusY: radius,
                }
            };

            Self::build_path_geometry(
                &self.d2d1_factory,
                &starting_point_circle.point,
                trail,
                settings,
            )
        })
    }

    unsafe fn build_path_geometry(
        d2d1_factory: &ID2D1Factory1,
        starting_point: &D2D_POINT_2F,
        trail: &ActiveTrail,
        settings: &Settings,
    ) -> ID2D1Geometry {
        let path = d2d1_factory
            .CreatePathGeometry()
            .log_expect("could not create path geometry");

        let sink = path.Open().log_expect("could not open path geometry");

        let starting_point2 = Point2::new(starting_point.x, starting_point.y);

        let points = simplify_line_string(trail.points, *settings.trail_simplify_epsilon);
        let radius = *trail.trail_width * STARTING_CIRCLE_RADIUS_FACTOR;

        let mut began_figure = false;

        for point in &points {
            if distance(&starting_point2, &point.xy()).abs() > radius {
                let point = D2D_POINT_2F {
                    x: point.x,
                    y: point.y,
                };

                if !began_figure {
                    began_figure = true;

                    sink.BeginFigure(point, D2D1_FIGURE_BEGIN_HOLLOW);
                } else {
                    sink.AddLine(point);
                }
            } else {
                // TODO: The point is (partially) inside the starting circle. Detect the part inside and draw it?
            }
        }

        // As a fallback if the trail is very short.
        if !began_figure {
            sink.BeginFigure(*starting_point, D2D1_FIGURE_BEGIN_HOLLOW);

            for point in points.iter().skip(1) {
                sink.AddLine(D2D_POINT_2F {
                    x: point.x,
                    y: point.y,
                });
            }
        }

        sink.EndFigure(D2D1_FIGURE_END_OPEN);

        sink.Close().log_expect("could not close path geometry");

        path.cast().log_unwrap()
    }
}
