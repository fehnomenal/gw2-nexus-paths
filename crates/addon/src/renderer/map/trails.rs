use std::{collections::HashMap, f32, rc::Rc};

use egui::{Color32, Rgba};
use log_err::LogErrResult;
use nalgebra::distance;
use paths_core::{
    maps::MAP_TO_WORLD_TRANSFORMATION_MATRICES,
    markers::{simplify_line_string, ActiveTrail},
    points::Point3,
    settings::{Settings, TrailWidth},
};
use windows::{
    core::Interface,
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{
        Common::{
            D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_BEGIN_HOLLOW,
            D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN, D2D_POINT_2F,
        },
        ID2D1Factory1, ID2D1Geometry, ID2D1SolidColorBrush, ID2D1StrokeStyle1,
        D2D1_CAP_STYLE_ROUND, D2D1_LINE_JOIN_ROUND, D2D1_STROKE_STYLE_PROPERTIES1,
    },
};

use super::MapRenderer;

const TRAIL_OUTLINE_WIDTH_FACTOR: f32 = 0.5;
const DISTANCE_BETWEEN_ARROWS: f32 = 500.0;
const ARROW_WIDTH_FACTOR: f32 = 4.0;
const ARROW_LENGTH_FACTOR: f32 = 4.0;

impl MapRenderer {
    pub unsafe fn draw_trails<'a, Trails: Iterator<Item = (&'a u32, &'a ActiveTrail<'a>)>>(
        &mut self,
        world_to_screen_transformation: &Matrix3x2,
        trails: Trails,
        settings: &Settings,
    ) {
        // Group trails by color.
        let mut trails_by_color = HashMap::<_, Vec<_>>::new();

        for (map_id, trail) in trails {
            if trail.points.len() >= 2 {
                trails_by_color
                    .entry(trail.color)
                    .or_default()
                    .push((map_id, trail));
            }
        }

        for (color, trails) in trails_by_color {
            let color: Rgba = Color32::from_rgb(color[0], color[1], color[2]).into();

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
                    color.intensity() < 0.5,
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
        bg_is_white: bool,
        settings: &Settings,
    ) {
        let Some(map_to_world_transformation) = MAP_TO_WORLD_TRANSFORMATION_MATRICES.get(map_id)
        else {
            return;
        };

        let geometries = self.trail_path_cache.get_trail_geometries(trail, settings);

        let bg_brush: &ID2D1SolidColorBrush = if bg_is_white {
            self.white_brush.get_or_insert_with(|| {
                self.d2d1_device_context
                    .CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                        None,
                    )
                    .log_expect("could not create white brush")
            })
        } else {
            self.black_brush.get_or_insert_with(|| {
                self.d2d1_device_context
                    .CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                        None,
                    )
                    .log_expect("could not create black brush")
            })
        };

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

        self.d2d1_device_context.DrawGeometry(
            &geometries.path,
            bg_brush,
            *trail.width * (1.0 + TRAIL_OUTLINE_WIDTH_FACTOR),
            stroke_style,
        );

        self.d2d1_device_context
            .DrawGeometry(&geometries.path, brush, *trail.width, stroke_style);

        for arrow in &geometries.arrows {
            self.d2d1_device_context.DrawGeometry(
                arrow,
                bg_brush,
                *trail.width * TRAIL_OUTLINE_WIDTH_FACTOR,
                stroke_style,
            );

            self.d2d1_device_context.FillGeometry(arrow, brush, None);
        }
    }
}

pub struct TrailPathCache {
    cache: HashMap<u64, TrailGeometries>,
    d2d1_factory: Rc<ID2D1Factory1>,
}

impl TrailPathCache {
    pub fn new(d2d1_factory: Rc<ID2D1Factory1>) -> Self {
        Self {
            cache: HashMap::new(),
            d2d1_factory,
        }
    }

    unsafe fn get_trail_geometries(
        &mut self,
        trail: &ActiveTrail,
        settings: &Settings,
    ) -> &TrailGeometries {
        self.cache
            .entry(trail.hash)
            .and_modify(|geometries| {
                if geometries.last_trail_width != trail.width {
                    let simplified_points =
                        simplify_line_string(&trail.points, *settings.trail_simplify_epsilon);

                    geometries.last_trail_width = trail.width;
                    geometries.arrows = TrailGeometries::build_arrows(
                        &self.d2d1_factory,
                        &simplified_points,
                        trail.width,
                    );
                }
            })
            .or_insert_with(|| {
                let simplified_points =
                    simplify_line_string(&trail.points, *settings.trail_simplify_epsilon);

                TrailGeometries {
                    last_trail_width: trail.width,
                    path: TrailGeometries::build_path(&self.d2d1_factory, &simplified_points),
                    arrows: TrailGeometries::build_arrows(
                        &self.d2d1_factory,
                        &simplified_points,
                        trail.width,
                    ),
                }
            })
    }
}

struct TrailGeometries {
    last_trail_width: TrailWidth,
    path: ID2D1Geometry,
    arrows: Vec<ID2D1Geometry>,
}

impl TrailGeometries {
    unsafe fn build_path(d2d1_factory: &ID2D1Factory1, points: &[Point3]) -> ID2D1Geometry {
        let path = d2d1_factory
            .CreatePathGeometry()
            .log_expect("could not create path geometry");

        let sink = path.Open().log_expect("could not open path geometry");

        sink.BeginFigure(
            D2D_POINT_2F {
                x: points[0].x,
                y: points[0].y,
            },
            D2D1_FIGURE_BEGIN_HOLLOW,
        );

        for point in points.iter().skip(1) {
            sink.AddLine(D2D_POINT_2F {
                x: point.x,
                y: point.y,
            });
        }

        sink.EndFigure(D2D1_FIGURE_END_OPEN);

        sink.Close().log_expect("could not close path geometry");

        path.cast().log_unwrap()
    }

    unsafe fn build_arrows(
        d2d1_factory: &ID2D1Factory1,
        points: &[Point3],
        trail_width: TrailWidth,
    ) -> Vec<ID2D1Geometry> {
        let base_arrow = Self::build_arrow(d2d1_factory, trail_width);

        let mut geometries = Vec::new();

        let mut last_point = None;
        let mut distance_to_next_arrow = 0.0;

        for (idx, point) in points.iter().enumerate() {
            // Subtract the distance between the last and the current point.
            if let Some(last_point) = last_point.replace(point) {
                let dist = distance(last_point, point);
                distance_to_next_arrow -= dist;
            }

            if distance_to_next_arrow <= 0.0 {
                distance_to_next_arrow = DISTANCE_BETWEEN_ARROWS;

                // Only draw an arrow if there is another point left.
                let Some(next_point) = points.get(idx + 1) else {
                    continue;
                };

                let rotation = Matrix3x2::rotation(
                    {
                        let delta_x = next_point.x - point.x;
                        let delta_y = next_point.y - point.y;

                        let radians = delta_y.atan2(delta_x);
                        let degrees = radians * 180.0 / f32::consts::PI;

                        degrees
                    },
                    0.0,
                    0.0,
                );
                let translation = Matrix3x2::translation(point.x, point.y);

                let arrow = d2d1_factory
                    .CreateTransformedGeometry(&base_arrow, &(rotation * translation))
                    .log_expect("could not transform arrow geometry");

                geometries.push(arrow.cast().log_unwrap());
            }
        }

        geometries
    }

    unsafe fn build_arrow(d2d1_factory: &ID2D1Factory1, trail_width: TrailWidth) -> ID2D1Geometry {
        let path = d2d1_factory
            .CreatePathGeometry()
            .log_expect("could not create arrow geometry");

        let sink = path.Open().log_expect("could not open path geometry");

        sink.BeginFigure(
            D2D_POINT_2F {
                x: 0.0,
                y: *trail_width * ARROW_WIDTH_FACTOR / 2.0,
            },
            D2D1_FIGURE_BEGIN_FILLED,
        );

        sink.AddLines(&[
            D2D_POINT_2F {
                x: *trail_width * ARROW_LENGTH_FACTOR,
                y: 0.0,
            },
            D2D_POINT_2F {
                x: 0.0,
                y: -*trail_width * ARROW_WIDTH_FACTOR / 2.0,
            },
        ]);

        sink.EndFigure(D2D1_FIGURE_END_CLOSED);

        sink.Close().log_expect("could not close path geometry");

        path.cast().log_unwrap()
    }
}
