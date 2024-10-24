mod active;
mod packs;
mod parse_trail;
mod ramer_douglas_peucker;
mod tree;
mod xml;

use std::cell::RefCell;

use crate::points::Point3;
use crate::settings::{TrailColor, TrailWidth};

pub use self::active::*;
pub use self::parse_trail::parse_trail;
pub use self::ramer_douglas_peucker::simplify_line_string;
pub use self::tree::{MarkerCategoryTree, MarkerCategoryTreeNode, NodeId};

#[derive(Debug)]
pub struct MarkerCategory {
    pub identifier: Vec<String>,
    pub label: String,
    pub is_separator: bool,
    pub is_active: RefCell<Option<bool>>,
    pub points_of_interest: Vec<PointOfInterest>,
    pub trails: Vec<Trail>,
    pub trail_color: RefCell<Option<TrailColor>>,
    pub trail_width: RefCell<Option<TrailWidth>>,
}

impl MarkerCategory {
    pub fn new(identifier: Vec<String>, label: String, is_separator: bool) -> Self {
        Self {
            identifier,
            label,
            is_separator,
            is_active: RefCell::new(None),
            points_of_interest: vec![],
            trails: vec![],
            trail_color: RefCell::new(None),
            trail_width: RefCell::new(None),
        }
    }

    pub fn root() -> Self {
        Self::new(vec![], "".to_owned(), false)
    }

    pub fn has_non_default_settings(&self) -> bool {
        self.trail_color.borrow().is_some() || self.trail_width.borrow().is_some()
    }
}

#[derive(Debug)]
pub struct PointOfInterest {
    // TODO
}

#[derive(Debug)]
pub struct Trail {
    pub map_id: u32,
    pub points: Vec<Point3>,
}

#[derive(Debug)]
pub struct TrailDescription {
    pub category_id_path: Vec<String>,
    pub binary_file_name: String,
}
