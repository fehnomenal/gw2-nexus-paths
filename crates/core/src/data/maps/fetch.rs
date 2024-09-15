use serde::Deserialize;

use super::dimensions::{MapDimensions, MapRect};

const GW2_MAPS_BASE_URL: &'static str = "https://api.guildwars2.com/v2/maps";

pub fn fetch_maps_index() -> Result<Vec<i32>, minreq::Error> {
    let res = minreq::get(GW2_MAPS_BASE_URL).send()?;

    res.json::<Vec<i32>>()
}

pub fn fetch_map_dimensions(map_id: i32) -> Result<MapDimensions, minreq::Error> {
    let res = minreq::get(&format!("{}/{}", GW2_MAPS_BASE_URL, map_id)).send()?;

    res.json::<RawMap>().map(|map| map.to_dimensions())
}

#[derive(Deserialize)]
struct RawMap {
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
            continent_rect,
            map_rect,
        }
    }
}
