use paths_types::{MarkerCategory, TrailDescription};
use xml::attribute::OwnedAttribute;

#[derive(Debug)]
pub enum ParseMarkerCategoryError {
    NoId,
}

pub fn marker_category_from_xml<C>(
    attributes: &[OwnedAttribute],
    parent_path: &[String],
) -> Result<MarkerCategory<C>, ParseMarkerCategoryError> {
    let mut name = None;
    let mut label = None;
    let mut is_separator = false;

    for attr in attributes {
        if attr.name.local_name.eq_ignore_ascii_case("Name") {
            name = Some(attr.value.clone());
        } else if attr.name.local_name.eq_ignore_ascii_case("DisplayName") {
            label = Some(attr.value.clone());
        } else if attr.name.local_name.eq_ignore_ascii_case("IsSeparator") {
            is_separator = attr.value == "1";
        }
    }

    let name = name.ok_or(ParseMarkerCategoryError::NoId)?;

    let mut identifier = parent_path.to_owned();
    identifier.push(name.clone());

    let label = label.unwrap_or(name);

    Ok(MarkerCategory::new(identifier, label, is_separator))
}

#[derive(Debug)]
pub enum ParseTrailDescriptionError {
    NoId,
    NoBinaryFile,
}

pub fn trail_description_from_xml(
    attributes: Vec<OwnedAttribute>,
) -> Result<TrailDescription, ParseTrailDescriptionError> {
    let mut identifier = None;
    let mut binary_file_name = None;

    for attr in attributes {
        if attr.name.local_name.eq_ignore_ascii_case("Type") {
            identifier = Some(attr.value);
        } else if attr.name.local_name.eq_ignore_ascii_case("TrailData") {
            binary_file_name = Some(attr.value);
        }
    }

    let ids = identifier
        .ok_or(ParseTrailDescriptionError::NoId)
        .map(|id| id.split('.').map(|s| s.to_owned()).collect())?;

    let binary_file_name = binary_file_name.ok_or(ParseTrailDescriptionError::NoBinaryFile)?;

    Ok(TrailDescription {
        category_id_path: ids,
        binary_file_name,
    })
}
