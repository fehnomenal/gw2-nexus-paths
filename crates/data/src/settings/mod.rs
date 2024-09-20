use std::io::Write;

use paths_types::settings::{Settings, SettingsV1};
use serde::Deserialize;

#[derive(Deserialize)]
struct OnlyVersion {
    version: usize,
}

pub fn read_settings(bytes: &[u8]) -> Settings {
    let version = serde_json::from_slice::<OnlyVersion>(bytes);
    match version {
        Ok(OnlyVersion { version: 1 }) => {
            serde_json::from_slice::<SettingsV1>(bytes).unwrap_or_default()
        }

        _ => return Settings::default(),
    }
    .into()
}

pub fn write_settings<W: Write>(writer: &mut W, settings: &Settings) {
    serde_json::to_writer_pretty(writer, settings).expect("Could not convert settings to json");
}
