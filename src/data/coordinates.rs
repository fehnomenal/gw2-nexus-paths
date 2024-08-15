use crate::state::{get_mumble_identity, get_mumble_link, get_nexus_link};

use super::{IPoint2, Point2};

pub struct WorldCoordinatesToScreenCoordinatesMapper {
    map_center: Point2,
    map_scale: f32,
    screen_width: f32,
    screen_height: f32,
}

impl WorldCoordinatesToScreenCoordinatesMapper {
    pub fn new() -> Self {
        let mumble_identity = unsafe { get_mumble_identity() };
        let mumble_link = unsafe { get_mumble_link() };
        let nexus_link = unsafe { get_nexus_link() };

        let map_center = Point2 {
            x: mumble_link.Context.Compass.Center.X,
            y: mumble_link.Context.Compass.Center.Y,
        };

        let map_scale = {
            let compass_scale = mumble_link.Context.Compass.Scale;

            let factor = mumble_identity.map_or(1.0, |identity| match identity.UISize {
                0 => 1.111,
                1 => 1.0,
                2 => 0.9,
                3 => 0.82,
                _ => 1.0,
            });

            compass_scale * factor
        };

        Self {
            map_center,
            map_scale,
            screen_width: nexus_link.Width as f32,
            screen_height: nexus_link.Height as f32,
        }
    }

    pub fn map_world_coordinates_to_screen_coordinates<P: IPoint2>(&self, point: &P) -> Point2 {
        Point2 {
            x: (point.x() - self.map_center.x) / self.map_scale + self.screen_width / 2.0,
            y: (point.y() - self.map_center.y) / self.map_scale + self.screen_height / 2.0,
        }
    }
}
