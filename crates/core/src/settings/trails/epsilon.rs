use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TrailSimplifyEpsilon(pub f32);

impl Default for TrailSimplifyEpsilon {
    fn default() -> Self {
        Self(0.001)
    }
}

impl Deref for TrailSimplifyEpsilon {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
