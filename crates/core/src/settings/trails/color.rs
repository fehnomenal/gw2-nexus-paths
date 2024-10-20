use egui::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TrailColor(pub Rgba);

impl std::ops::Deref for TrailColor {
    type Target = Rgba;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for TrailColor {
    fn default() -> Self {
        Self(Rgba::from_rgba_unmultiplied(1.0, 0.0, 0.0, 0.8))
    }
}
