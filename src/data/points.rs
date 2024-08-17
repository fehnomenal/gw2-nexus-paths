use windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F;

#[derive(Debug)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn as_d2d_point_2f(&self) -> D2D_POINT_2F {
        D2D_POINT_2F {
            x: self.x,
            y: self.y,
        }
    }
}
