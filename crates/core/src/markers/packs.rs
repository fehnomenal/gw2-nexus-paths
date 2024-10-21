#[cfg(debug_assertions)]
use std::time::Instant;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    path::Path,
};

use log::{debug, error, warn};
use log_err::{LogErrOption, LogErrResult};
use xml::{reader::XmlEvent, EventReader};
use zip::ZipArchive;

use super::{
    parse_trail,
    tree::{ensure_category_path, MarkerCategoryTree},
    xml::{marker_category_from_xml, trail_description_from_xml},
    MarkerCategory, Trail,
};

impl MarkerCategoryTree {
    pub fn load_marker_pack_from_path(&mut self, path: &Path) {
        debug!(
            "loading marker categories from {}",
            path.to_str().log_unwrap()
        );

        let file = File::open(path).log_expect("could not open file");
        let mut zip =
            ZipArchive::new(BufReader::new(file)).log_expect("could not create zip reader");

        #[cfg(debug_assertions)]
        let now = Instant::now();

        let mut trails = parse_all_trails(&mut zip);

        #[cfg(debug_assertions)]
        debug!(
            "parsed {} trails in {} ms",
            trails.len(),
            now.elapsed().as_millis(),
        );

        #[cfg(debug_assertions)]
        let now = Instant::now();

        for i in 0..zip.len() {
            let file = zip.by_index(i).log_unwrap();

            if !file.name().ends_with(".xml") {
                continue;
            }

            let reader = BufReader::new(file);
            let parser = EventReader::new(reader);

            read_xml_file(parser, self, &mut trails);
        }

        #[cfg(debug_assertions)]
        debug!(
            "loaded marker categories in {} ms",
            now.elapsed().as_millis(),
        );

        self.pack_count += 1;
    }
}

fn parse_all_trails<R: Read + Seek>(zip: &mut ZipArchive<R>) -> HashMap<String, Trail> {
    let mut trails = HashMap::new();

    for idx in 0..zip.len() {
        let mut file = zip
            .by_index(idx)
            .log_expect("zip archive does not contain file by index???");

        let normalized_name = normalize_file_name(file.name());

        if normalized_name.ends_with(".trl") {
            let mut bytes = Vec::new();
            bytes.reserve_exact(file.size() as usize);
            file.read_to_end(&mut bytes)
                .log_expect("could not read binary trail data");

            if let Ok((_, trail)) = parse_trail(&bytes) {
                trails.insert(normalized_name, trail);
            }
        }
    }

    trails
}

fn read_xml_file<R: BufRead>(
    mut parser: EventReader<R>,
    tree: &mut MarkerCategoryTree,
    trails: &mut HashMap<String, Trail>,
) {
    let mut current_parent_node_id = tree.tree.root_id().log_expect("tree has no root node");
    let mut current_parent_path = Vec::<String>::new();
    let mut go_to_parent = false;

    loop {
        match parser.next() {
            Err(err) => {
                error!("Could not get xml event: {err}");

                break;
            }

            Ok(XmlEvent::EndDocument) => {
                // Sanity check.
                debug_assert_eq!(current_parent_node_id, tree.tree.root_id().log_unwrap());

                break;
            }

            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) if name.local_name.eq_ignore_ascii_case("MarkerCategory") => {
                match marker_category_from_xml(&attributes, &current_parent_path) {
                    Ok(category) => {
                        let identifier = category.identifier.clone();
                        let label = category.label.clone();
                        let is_separator = category.is_separator;

                        current_parent_node_id = ensure_category_path(
                            &mut tree.tree,
                            current_parent_node_id,
                            &[category.identifier.last().log_unwrap().clone()],
                            |_| {
                                MarkerCategory::new(identifier.clone(), label.clone(), is_separator)
                            },
                        );
                        current_parent_path = identifier;

                        go_to_parent = true;
                    }

                    Err(err) => {
                        debug!("could not parse marker category: {:?}", attributes);

                        // TODO: Is it ok to just skip this subtree?
                        // We could not create and thus insert a category. So we have nothing to insert to
                        // and cannot get the parent when visiting the end tag.
                        parser
                            .skip()
                            .log_expect(&format!("error while skipping marker category sub tree (marker category tag was invalid: {:?})", err));

                        go_to_parent = false;
                    }
                }
            }

            Ok(XmlEvent::EndElement { name })
                if name.local_name.eq_ignore_ascii_case("MarkerCategory") =>
            {
                if go_to_parent {
                    current_parent_node_id = tree
                        .tree
                        .get(current_parent_node_id)
                        .log_unwrap()
                        .parent()
                        .log_unwrap()
                        .node_id();

                    current_parent_path.pop();
                }
            }

            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) if name.local_name.eq_ignore_ascii_case("Trail") => {
                match trail_description_from_xml(attributes) {
                    Ok(trail_description) => {
                        let normalized_file_name =
                            normalize_file_name(&trail_description.binary_file_name);

                        if let Some(trail) = trails.remove(&normalized_file_name) {
                            let path = trail_description.category_id_path.as_slice();

                            let root_id = tree.tree.root_id().log_unwrap();
                            let category_node_id =
                                ensure_category_path(&mut tree.tree, root_id, path, |id| {
                                    MarkerCategory::new(path.to_owned(), id.to_owned(), false)
                                });

                            tree.tree
                                .get_mut(category_node_id)
                                .log_unwrap()
                                .data()
                                .trails
                                .push(trail);

                            tree.trail_count += 1;
                        }
                    }

                    Err(err) => {
                        warn!("could not parse trail description: {err:?}");
                    }
                }
            }

            Ok(_) => {}
        }
    }
}

fn normalize_file_name(file_name: &str) -> String {
    file_name.to_lowercase().replace('\\', "/")
}
