use std::{cell::RefCell, path::PathBuf};

pub struct MarkerCategory<C> {
    pub identifier: String,
    pub label: String,
    pub is_separator: bool,
    pub is_active: RefCell<bool>,
    pub points_of_interest: Vec<PointOfInterest>,
    pub trails: Vec<TrailDescription>,
    pub trail_color: Option<C>,
    pub trail_width: Option<f32>,
}

impl<C> MarkerCategory<C> {
    pub fn new(identifier: String, label: String, is_separator: bool) -> Self {
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
        Self::new("".to_owned(), "".to_owned(), false)
    }

    pub fn path<P: AsRef<str>>(&self, parent_path: &[P]) -> Vec<String> {
        let mut path = parent_path
            .to_owned()
            .into_iter()
            .map(|s| s.as_ref().to_owned())
            .collect::<Vec<_>>();

        path.push(self.identifier.clone());

        path
    }

    pub fn has_non_default_settings(&self) -> bool {
        self.trail_color.is_some() || self.trail_width.is_some()
    }
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
