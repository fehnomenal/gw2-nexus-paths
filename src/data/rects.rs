use super::Point2;

pub struct MapRect {
    pub bottom_left: Point2,
    pub top_right: Point2,
}

impl MapRect {
    pub const fn from_array(coords: [[f32; 2]; 2]) -> Self {
        Self {
            bottom_left: Point2::from_array(coords[0]),
            top_right: Point2::from_array(coords[1]),
        }
    }
}

pub struct ContinentRect {
    pub top_left: Point2,
    pub bottom_right: Point2,
}

impl ContinentRect {
    pub const fn from_array(coords: [[f32; 2]; 2]) -> Self {
        Self {
            top_left: Point2::from_array(coords[0]),
            bottom_right: Point2::from_array(coords[1]),
        }
    }
}
