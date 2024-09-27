use std::io::Write;

use paths_types::settings::{Settings, SettingsV1};
use serde::Deserialize;

#[derive(Deserialize)]
struct OnlyVersion {
    version: usize,
}

pub fn read_settings(bytes: &[u8]) -> Settings {
    match serde_json::from_slice::<OnlyVersion>(bytes) {
        Ok(OnlyVersion { version: 1 }) => match serde_json::from_slice::<SettingsV1>(bytes) {
            Ok(settings) => {
                #[cfg(debug_assertions)]
                println!("Got settings: {:?}", settings);

                settings
            }

            #[cfg(debug_assertions)]
            Err(err) => {
                eprintln!("Could not read settings: {err}");

                Settings::default()
            }

            #[cfg(not(debug_assertions))]
            _ => Settings::default(),
        },

        #[cfg(debug_assertions)]
        Ok(OnlyVersion { version }) => {
            eprintln!("Got settings with unrecognized version: {version}");

            Settings::default()
        }

        #[cfg(debug_assertions)]
        Err(err) => {
            eprintln!("Could not read settings version: {err}");

            Settings::default()
        }

        #[cfg(not(debug_assertions))]
        _ => Settings::default(),
    }
    .into()
}

pub fn write_settings<W: Write>(writer: &mut W, settings: &Settings) {
    serde_json::to_writer_pretty(writer, settings).expect("Could not convert settings to json");
}
