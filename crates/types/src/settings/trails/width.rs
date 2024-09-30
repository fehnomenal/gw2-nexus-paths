use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TrailWidth(pub f32);

impl TrailWidth {
    pub fn is_default(&self) -> bool {
        self.0 == Self::default().0
    }
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
