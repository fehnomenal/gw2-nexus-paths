use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{TrailColor, TrailSimplifyEpsilon, TrailWidth};

type Name = String;
type CategoryId = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsV1 {
    version: usize,

    #[serde(default)]
    pub default_trail_color: TrailColor,

    #[serde(default)]
    pub default_trail_width: TrailWidth,

    #[serde(default)]
    pub trail_simplify_epsilon: TrailSimplifyEpsilon,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub marker_presets: HashMap<Name, HashMap<CategoryId, MarkerCategorySetting>>,
}

impl Default for SettingsV1 {
    fn default() -> Self {
        Self {
            version: 1,

            default_trail_color: TrailColor::default(),
            default_trail_width: TrailWidth::default(),
            trail_simplify_epsilon: TrailSimplifyEpsilon::default(),

            marker_presets: HashMap::new(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MarkerCategorySetting {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_color: Option<TrailColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_width: Option<TrailWidth>,
}