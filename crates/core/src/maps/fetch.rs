use paths_types::{MapDimensions, MapRect};
use serde::Deserialize;

pub use self::error::{FetchError, FetchResult};

const GW2_MAPS_BASE_URL: &'static str = "https://api.guildwars2.com/v2/maps";

pub fn fetch_maps_index() -> FetchResult<Vec<u32>> {
    let res = minreq::get(GW2_MAPS_BASE_URL).send()?;

    if res.status_code != 200 {
        return Err(error::FetchError::NonOkStatus {
            status_code: res.status_code,
            reason_phrase: res.reason_phrase.clone(),
            body: res.as_str()?.to_owned(),
        });
    }

    Ok(res.json::<Vec<u32>>()?)
}

pub fn fetch_map_dimensions(map_ids: &[u32]) -> FetchResult<Vec<MapDimensions>> {
    let res = minreq::get(&format!(
        "{}?ids={}",
        GW2_MAPS_BASE_URL,
        map_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",")
    ))
    .send()?;

    if res.status_code != 200 {
        return Err(error::FetchError::NonOkStatus {
            status_code: res.status_code,
            reason_phrase: res.reason_phrase.clone(),
            body: res.as_str()?.to_owned(),
        });
    }

    Ok(res
        .json::<Vec<RawMap>>()
        .map(|maps| maps.iter().map(|map| map.to_dimensions()).collect())?)
}

#[derive(Deserialize)]
struct RawMap {
    id: u32,
    continent_rect: [[f32; 2]; 2],
    map_rect: [[f32; 2]; 2],
}

impl RawMap {
    fn to_dimensions(&self) -> MapDimensions {
        let continent_rect = MapRect {
            top_left: self.continent_rect[0],
            width: (self.continent_rect[1][0] - self.continent_rect[0][0]),
            height: (self.continent_rect[1][1] - self.continent_rect[0][1]),
        };

        let map_rect = MapRect {
            top_left: [self.map_rect[0][0], self.map_rect[1][1]],
            width: (self.map_rect[1][0] - self.map_rect[0][0]),
            height: (self.map_rect[1][1] - self.map_rect[0][1]),
        };

        MapDimensions {
            map_id: self.id,
            continent_rect,
            map_rect,
        }
    }
}

mod error {
    use std::fmt::Display;

    #[derive(Debug)]
    pub enum FetchError {
        MinReq(minreq::Error),
        NonOkStatus {
            status_code: i32,
            reason_phrase: String,
            body: String,
        },
    }

    pub type FetchResult<T> = Result<T, FetchError>;

    impl From<minreq::Error> for FetchError {
        fn from(value: minreq::Error) -> Self {
            Self::MinReq(value)
        }
    }

    impl Display for FetchError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                FetchError::MinReq(err) => write!(f, "{err}"),
                FetchError::NonOkStatus {
                    status_code,
                    reason_phrase,
                    body,
                } => write!(f, "non-200 response: {status_code} {reason_phrase}; {body}"),
            }
        }
    }
}
