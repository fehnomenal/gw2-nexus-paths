use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use paths_types::MarkerCategory;
use xml::{reader::XmlEvent, EventReader};
use zip::ZipArchive;

use super::{
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

            read_xml_file(path, parser, self);
        }

        self.pack_count += 1;
    }
}

fn read_xml_file<R: BufRead, C>(
    zip_path: &Path,
    mut parser: EventReader<R>,
    tree: &mut MarkerCategoryTree<C>,
) {
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
                match trail_description_from_xml(attributes, zip_path) {
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
                            .push(trail_description);

                        tree.trail_count += 1;
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
