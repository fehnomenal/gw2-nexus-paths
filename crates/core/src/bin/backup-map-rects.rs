use std::{fs::File, io::Write, path::PathBuf};

use paths_core::data::maps::{fetch_map_dimensions, fetch_maps_index};

fn main() {
    let target_file_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/data/maps/map-dimension-fallback.json");

    let target_file =
        File::create(target_file_path).expect("Could not open fallback map dimensions file");

    let maps = fetch_maps_index().expect("Could not load maps");

    for (idx, map_id) in maps.iter().enumerate() {
        let dimensions =
            fetch_map_dimensions(*map_id).expect(&format!("Could not load map {map_id}"));
        let json = serde_json::to_string(&dimensions)
            .expect(&format!("Could not convert map details {map_id} to json"));

        let line = format!(r#"{}"{map_id}":{json}"#, if idx == 0 { "{" } else { "," });

        writeln!(&target_file, "{line}").expect(&format!("Could not write line for map {map_id}"));

        println!("Wrote line for map {map_id}");
    }

    writeln!(&target_file, "}}").expect("Could not write last line");
}
