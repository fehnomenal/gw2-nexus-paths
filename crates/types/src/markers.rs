use std::cell::RefCell;

use crate::Point3;

#[derive(Debug)]
pub struct MarkerCategory<C> {
    pub identifier: Vec<String>,
    pub label: String,
    pub is_separator: bool,
    pub is_active: RefCell<bool>,
    pub points_of_interest: Vec<PointOfInterest>,
    pub trails: Vec<Trail>,
    pub trail_color: Option<C>,
    pub trail_width: Option<f32>,
}

impl<C> MarkerCategory<C> {
    pub fn new(identifier: Vec<String>, label: String, is_separator: bool) -> Self {
        Self {
            identifier,
            label,
            is_separator,
            is_active: RefCell::new(false),
            points_of_interest: vec![],
            trails: vec![],
            trail_color: None,
            trail_width: None,
        }
    }

    pub fn root() -> Self {
        Self::new(vec![], "".to_owned(), false)
    }

    pub fn has_non_default_settings(&self) -> bool {
        self.trail_color.is_some() || self.trail_width.is_some()
    }
}

#[derive(Debug)]
pub struct PointOfInterest {
    // TODO
}

#[derive(Clone, Debug)]
pub struct Trail {
    pub map_id: u32,
    pub points: Vec<Point3>,
}

#[derive(Clone, Debug)]
pub struct TrailDescription {
    pub category_id_path: Vec<String>,
    pub binary_file_name: String,
}
