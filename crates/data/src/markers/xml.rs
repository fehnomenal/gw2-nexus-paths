use std::{cell::RefCell, path::Path};

use paths_types::{MarkerCategory, TrailDescription};
use xml::attribute::OwnedAttribute;

#[derive(Debug)]
pub enum ParseMarkerCategoryError {
    NoId,
}

pub fn marker_category_from_xml(
    attributes: Vec<OwnedAttribute>,
) -> Result<MarkerCategory, ParseMarkerCategoryError> {
    let mut identifier = None;
    let mut label = None;
    let mut is_separator = false;

    for attr in attributes {
        if attr.name.local_name.eq_ignore_ascii_case("Name") {
            identifier = Some(attr.value);
        } else if attr.name.local_name.eq_ignore_ascii_case("DisplayName") {
            label = Some(attr.value);
        } else if attr.name.local_name.eq_ignore_ascii_case("IsSeparator") {
            is_separator = attr.value == "1";
        }
    }

    let identifier = identifier.ok_or(ParseMarkerCategoryError::NoId)?;

    let label = label.unwrap_or_else(|| identifier.clone());

    Ok(MarkerCategory {
        identifier,
        label,
        is_separator,
        is_selected: RefCell::new(false),
        points_of_interest: vec![],
        trails: vec![],
    })
}

#[derive(Debug)]
pub enum ParseTrailDescriptionError {
    NoId,
    NoBinaryFile,
}

pub fn trail_description_from_xml(
    attributes: Vec<OwnedAttribute>,
    pack_file: &Path,
) -> Result<TrailDescription, ParseTrailDescriptionError> {
    let mut identifier = None;
    let mut binary_file = None;

    for attr in attributes {
        if attr.name.local_name.eq_ignore_ascii_case("Type") {
            identifier = Some(attr.value);
        } else if attr.name.local_name.eq_ignore_ascii_case("TrailData") {
            binary_file = Some(attr.value);
        }
    }

    let ids = identifier
        .ok_or(ParseTrailDescriptionError::NoId)
        .map(|id| id.split('.').map(|s| s.to_owned()).collect())?;

    let binary_file = binary_file.ok_or(ParseTrailDescriptionError::NoBinaryFile)?;

    Ok(TrailDescription {
        ids,
        pack_file: pack_file.to_path_buf(),
        binary_file,
    })
}
