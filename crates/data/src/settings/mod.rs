use std::io::Read;

use paths_types::settings::{Settings, SettingsV1};
use serde::Deserialize;

#[derive(Deserialize)]
struct OnlyVersion {
    version: usize,
}

pub fn read_settings<R: Read + Clone>(reader: R) -> Settings {
    let version = serde_json::from_reader::<R, OnlyVersion>(reader.clone());
    match version {
        Ok(OnlyVersion { version: 1 }) => {
            serde_json::from_reader::<R, SettingsV1>(reader).unwrap_or_default()
        }

        _ => return Settings::default(),
    }
    .into()
}
