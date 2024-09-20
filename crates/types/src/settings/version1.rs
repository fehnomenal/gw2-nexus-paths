use std::collections::HashMap;

use egui::Rgba;
use serde::{Deserialize, Serialize};

type Name = String;
type CategoryId = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsV1 {
    version: usize,

    #[serde(
        default = "default_trail_color",
        skip_serializing_if = "skip_default_trail_color"
    )]
    pub default_trail_color: Rgba,

    #[serde(
        default = "default_trail_width",
        skip_serializing_if = "skip_default_trail_width"
    )]
    pub default_trail_width: f32,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub marker_categories: HashMap<Name, HashMap<CategoryId, MarkerCategorySetting>>,
}

impl Default for SettingsV1 {
    fn default() -> Self {
        Self {
            version: 1,

            default_trail_color: default_trail_color(),
            default_trail_width: default_trail_width(),
            marker_categories: Default::default(),
        }
    }
}

fn default_trail_color() -> Rgba {
    Rgba::from_rgba_unmultiplied(1.0, 0.0, 0.0, 0.8)
}

fn skip_default_trail_color(col: &Rgba) -> bool {
    col == &default_trail_color()
}

fn default_trail_width() -> f32 {
    2.5
}

fn skip_default_trail_width(w: &f32) -> bool {
    w == &default_trail_width()
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MarkerCategorySetting {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_color: Option<Rgba>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_width: Option<f32>,
}
