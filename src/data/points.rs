use windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F;

pub trait IPoint2 {
    fn x(&self) -> f32;
    fn y(&self) -> f32;

    fn as_d2d_point_2f(&self) -> D2D_POINT_2F {
        D2D_POINT_2F {
            x: self.x(),
            y: self.y(),
        }
    }
}

#[derive(Debug)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub const fn from_array(coords: [f32; 2]) -> Self {
        Self {
            x: coords[0],
            y: coords[1],
        }
    }
}

impl IPoint2 for Point2 {
    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}
