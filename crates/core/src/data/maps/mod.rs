mod dimensions;
mod fetch;

use std::collections::HashMap;

use dimensions::MapDimensions;
pub use fetch::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MapDetails(HashMap<u32, MapDimensions>);
