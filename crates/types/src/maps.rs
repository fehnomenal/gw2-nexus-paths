use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MapDimensions {
    #[serde(rename = "cr")]
    pub continent_rect: MapRect,
    #[serde(rename = "mr")]
    pub map_rect: MapRect,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MapRect {
    #[serde(rename = "tl")]
    pub top_left: [f32; 2],
    #[serde(rename = "w")]
    pub width: f32,
    #[serde(rename = "h")]
    pub height: f32,
}
