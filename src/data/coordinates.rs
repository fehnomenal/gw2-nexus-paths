use crate::nexus::api::mumble::Mumble_Compass;

use super::{IPoint2, Point2};

pub struct Coordinates {
    map_center: Point2,
    map_scale: f32,
    screen_width: f32,
    screen_height: f32,
}

impl Coordinates {
    pub fn new(compass: &Mumble_Compass, screen_width: f32, screen_height: f32) -> Self {
        Self {
            map_center: Point2 {
                x: compass.Center.X,
                y: compass.Center.Y,
            },
            map_scale: compass.Scale * 0.82,
            screen_width,
            screen_height,
        }
    }

    pub fn map_world_coordinates_to_screen_coordinates<P: IPoint2>(&self, point: &P) -> Point2 {
        Point2 {
            x: (point.x() - self.map_center.x) / self.map_scale + self.screen_width / 2.0,
            y: (point.y() - self.map_center.y) / self.map_scale + self.screen_height / 2.0,
        }
    }
}
