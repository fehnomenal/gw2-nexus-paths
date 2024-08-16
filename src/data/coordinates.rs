use crate::{render::RenderState, state::get_mumble_link};

use super::{IPoint2, Point2};

pub struct WorldCoordinatesToScreenCoordinatesMapper<'a> {
    map_center: Point2,
    map_scale: f32,
    render_state: &'a RenderState,
}

impl<'a> WorldCoordinatesToScreenCoordinatesMapper<'a> {
    pub fn new(render_state: &'a RenderState) -> Self {
        let mumble_link = unsafe { get_mumble_link() };

        let map_center = Point2 {
            x: mumble_link.Context.Compass.Center.X,
            y: mumble_link.Context.Compass.Center.Y,
        };

        let map_scale = {
            let compass_scale = mumble_link.Context.Compass.Scale;

            compass_scale * render_state.map_scale_factor()
        };

        Self {
            map_center,
            map_scale,
            render_state,
        }
    }

    pub fn map_world_coordinates_to_screen_coordinates<P: IPoint2>(&self, point: &P) -> Point2 {
        Point2 {
            x: (point.x() - self.map_center.x) / self.map_scale
                + self.render_state.screen_width() / 2.0,
            y: (point.y() - self.map_center.y) / self.map_scale
                + self.render_state.screen_height() / 2.0,
        }
    }
}
