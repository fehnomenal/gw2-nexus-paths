use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    path::Path,
};

use paths_types::{MarkerCategory, TrailDescription, TrailDescriptionLoaded};
use xml::{reader::XmlEvent, EventReader};
use zip::{
    read::ZipFile,
    result::{ZipError, ZipResult},
    ZipArchive,
};

use super::{
    parse_trail,
    tree::{ensure_category_path, MarkerCategoryTree},
    xml::{marker_category_from_xml, trail_description_from_xml},
};

impl<C> MarkerCategoryTree<C> {
    pub fn load_marker_pack_from_path(&mut self, path: &Path) {
        let file = File::open(path).expect("Could not open file");
        let mut zip = ZipArchive::new(BufReader::new(file)).expect("Could not create zip reader");

        for i in 0..zip.len() {
            let file = zip.by_index(i).unwrap();

            if !file.name().ends_with(".xml") {
                continue;
            }

            let reader = BufReader::new(file);
            let parser = EventReader::new(reader);

            read_xml_file(parser, self);
        }

        for node in self.tree.root().unwrap().traverse_pre_order() {
            let category = node.data();

            let trails = category.trails.take();

            let loaded_trails = trails
                .into_iter()
                .filter_map(|trail_desc| {
                    if let TrailDescription::Reference(reference) = trail_desc {
                        let trail = resolve_zip_file(&mut zip, &reference.binary_file_name)
                            .ok()
                            .and_then(|mut file| {
                                let mut bytes = Vec::new();
                                bytes.reserve_exact(file.size() as usize);
                                file.read_to_end(&mut bytes)
                                    .expect("Could not read binary trail data");

                                parse_trail(&bytes).ok().map(|(_, trail)| trail)
                            });

                        if trail.is_some() {
                            self.trail_count += 1;
                        }

                        trail
                            .map(|trail| TrailDescription::Loaded(TrailDescriptionLoaded { trail }))
                    } else {
                        Some(trail_desc)
                    }
                })
                .collect::<Vec<_>>();

            *category.trails.borrow_mut() = loaded_trails;
        }

        self.pack_count += 1;
    }
}

fn read_xml_file<R: BufRead, C>(mut parser: EventReader<R>, tree: &mut MarkerCategoryTree<C>) {
    let mut current_parent_node_id = tree.tree.root_id().expect("Tree has no root node");
    let mut current_parent_path = Vec::<String>::new();
    let mut go_to_parent = false;

    loop {
        match parser.next() {
            Err(err) => {
                // TODO: Error handling
                panic!("Could not get xml event: {err}");
            }

            Ok(XmlEvent::EndDocument) => {
                // Sanity check
                assert_eq!(current_parent_node_id, tree.tree.root_id().unwrap());

                break;
            }

            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) if name.local_name.eq_ignore_ascii_case("MarkerCategory") => {
                match marker_category_from_xml::<C>(&attributes, &current_parent_path) {
                    Ok(category) => {
                        let identifier = category.identifier.clone();
                        let label = category.label.clone();
                        let is_separator = category.is_separator;

                        current_parent_node_id = ensure_category_path(
                            &mut tree.tree,
                            current_parent_node_id,
                            &[category.identifier.last().unwrap().clone()],
                            |_| {
                                MarkerCategory::new(identifier.clone(), label.clone(), is_separator)
                            },
                        );
                        current_parent_path = identifier;

                        go_to_parent = true;
                    }

                    Err(err) => {
                        // TODO: Is it ok to just skip this subtree?
                        // We could not create and thus insert a category. So we have nothing to insert to
                        // and cannot get the parent when visiting the end tag.
                        parser
                            .skip()
                            .expect(&format!("Error while skipping marker category sub tree (marker category tag was invalid: {:?})", err));

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
                        .unwrap()
                        .parent()
                        .unwrap()
                        .node_id();

                    current_parent_path.pop();
                }
            }

            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) if name.local_name.eq_ignore_ascii_case("Trail") => {
                match trail_description_from_xml(attributes) {
                    Ok(trail_description) => {
                        let path = trail_description.category_id_path.as_slice();

                        let root_id = tree.tree.root_id().unwrap();
                        let category_node_id =
                            ensure_category_path(&mut tree.tree, root_id, path, |id| {
                                MarkerCategory::new(path.to_owned(), id.to_owned(), false)
                            });

                        tree.tree
                            .get_mut(category_node_id)
                            .unwrap()
                            .data()
                            .trails
                            .get_mut()
                            .push(TrailDescription::Reference(trail_description));
                    }

                    Err(err) => {
                        eprintln!("Could not parse trail description: {:?}", err);
                    }
                }
            }

            Ok(_) => {}
        }
    }
}

fn normalize_zip_entry_file_name<R: Read + Seek>(
    zip: &ZipArchive<R>,
    file_name: &str,
) -> Option<String> {
    // Normalize file name.
    let file_name = file_name.replace('\\', "/");

    zip.file_names()
        .into_iter()
        .find(|name| name.eq_ignore_ascii_case(&file_name))
        .map(|name| name.to_owned())
}

fn resolve_zip_file<'a, R: Read + Seek>(
    zip: &'a mut ZipArchive<R>,
    file_name: &'a str,
) -> ZipResult<ZipFile<'a>> {
    let Some(real_file_name) = normalize_zip_entry_file_name(zip, file_name) else {
        return Err(ZipError::FileNotFound);
    };

    zip.by_name(&real_file_name)
}
