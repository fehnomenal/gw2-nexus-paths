#[derive(Clone, Debug)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    z: f32,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
