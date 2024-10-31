use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TrailColor(pub [u8; 4]);

impl Deref for TrailColor {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for TrailColor {
    fn default() -> Self {
        Self([255, 0, 0, 205])
    }
}