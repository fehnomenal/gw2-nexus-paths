use std::{cell::RefCell, path::PathBuf};

#[derive(Clone)]
pub struct MarkerCategory {
    pub identifier: String,
    pub label: String,
    pub is_separator: bool,
    pub is_selected: RefCell<bool>,
    pub points_of_interest: Vec<PointOfInterest>,
    pub trails: Vec<TrailDescription>,
}

#[derive(Clone)]
pub struct PointOfInterest {
    // TODO
}

#[derive(Clone)]
pub struct TrailDescription {
    pub ids: Vec<String>,
    pub pack_file: PathBuf,
    pub binary_file: String,
}
