use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TrailWidth(pub f32);

impl TrailWidth {
    pub const MIN: f32 = 1.5;
    pub const MAX: f32 = 10.0;
}

impl Default for TrailWidth {
    fn default() -> Self {
        Self(2.5)
    }
}

impl Deref for TrailWidth {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
