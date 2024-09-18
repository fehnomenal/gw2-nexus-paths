use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use paths_data::maps::{fetch_map_dimensions, fetch_maps_index, FetchError, FetchResult};
use paths_types::MapDimensions;

fn main() {
    let target_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("map-dimensions.json");

    let mut existing = load_existing_data(&target_file_path);

    let map_ids = fetch_maps_index().expect("Could not load maps");

    for map_ids in map_ids.chunks(30) {
        match fetch_map_dimensions_with_retry(map_ids, 3) {
            Ok(dimensions) => {
                for dim in dimensions {
                    existing.insert(dim.map_id, dim);
                }

                println!("Fetched map dimensions {:?}", map_ids);
            }

            Err(err) => eprintln!("Could not load maps {:?}: {err}", map_ids),
        }
    }

    let json = existing
        .iter()
        .filter_map(
            |(map_id, dimensions)| match serde_json::to_string(dimensions) {
                Ok(json) => Some(format!(r#""{map_id}":{json}"#)),

                Err(err) => {
                    eprintln!("Could not convert map dimensions {map_id} to json: {err}");
                    None
                }
            },
        )
        .collect::<Vec<_>>()
        .join("\n,");

    let target_file =
        File::create(target_file_path).expect("Could not open map dimensions file for writing");

    writeln!(&target_file, "{{{json}\n}}").expect("Could not write json to file");
}

fn load_existing_data(file_path: &Path) -> BTreeMap<u32, MapDimensions> {
    File::open(file_path)
        .ok()
        .map(|file| BufReader::new(file))
        .and_then(|reader| serde_json::from_reader(reader).ok())
        .unwrap_or_default()
}

fn fetch_map_dimensions_with_retry(
    map_ids: &[u32],
    attempts_left: u8,
) -> FetchResult<Vec<MapDimensions>> {
    let result = fetch_map_dimensions(map_ids);

    if let Err(FetchError::NonOkStatus {
        status_code: 429, ..
    }) = result
    {
        if attempts_left > 0 {
            // Wait 5 seconds.
            thread::sleep(Duration::from_secs(5));

            return fetch_map_dimensions_with_retry(map_ids, attempts_left - 1);
        }
    }

    result
}
