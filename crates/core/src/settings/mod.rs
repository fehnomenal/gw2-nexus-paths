mod marker_categories;

use std::{fmt::Debug, io::Write};

use log::{debug, warn};
use log_err::LogErrResult;
use paths_types::settings::{Settings, SettingsV1};
use serde::Deserialize;

pub use self::marker_categories::*;

#[derive(Deserialize)]
struct OnlyVersion {
    version: usize,
}

pub fn read_settings(bytes: &[u8]) -> Settings {
    match serde_json::from_slice::<OnlyVersion>(bytes) {
        Ok(OnlyVersion { version }) => match version {
            1 => parse_and_unwrap_settings::<SettingsV1>(bytes),

            _ => {
                warn!("got settings with unrecognized version: {version}");

                Settings::default()
            }
        },

        Err(err) => {
            debug!("could not read settings version: {err}");

            Settings::default()
        }
    }
}

fn parse_and_unwrap_settings<'de, S: Debug + Deserialize<'de> + Into<Settings>>(
    bytes: &'de [u8],
) -> Settings {
    let res = serde_json::from_slice::<S>(bytes);

    match res {
        Ok(settings) => {
            debug!("got settings: {settings:?}");

            settings.into()
        }

        Err(err) => {
            debug!("could not read settings: {err}");

            Settings::default()
        }
    }
}

pub fn write_settings<W: Write>(writer: &mut W, settings: &Settings) {
    serde_json::to_writer_pretty(writer, settings).log_expect("could not convert settings to json");
}
