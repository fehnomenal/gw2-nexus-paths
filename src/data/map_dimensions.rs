use std::{collections::HashMap, sync::LazyLock};

use super::{CoordinatesRelativeToMapRectCenter, Point2, WorldCoordinates};

static MAP_DIMENSIONS: LazyLock<HashMap<u32, MapDimensions>> = LazyLock::new(|| {
    let raw = jzon::parse(include_str!("../../mapDetails.json")).unwrap();

    let mut dimensions = HashMap::with_capacity(raw.len());

    for (map_id_str, map_details) in raw.entries() {
        let continent_rect = (map_details["ContinentRectTopLeftX"].as_f32())
            .zip(map_details["ContinentRectTopLeftY"].as_f32())
            .zip(map_details["ContinentRectWidth"].as_f32())
            .zip(map_details["ContinentRectHeight"].as_f32())
            .map(|(((top_left_x, top_left_y), width), height)| Rect {
                top_left_x,
                top_left_y,
                width,
                height,
            });

        let map_rect = (map_details["MapRectTopLeftX"].as_f32())
            .zip(map_details["MapRectTopLeftY"].as_f32())
            .zip(map_details["MapRectWidth"].as_f32())
            .zip(map_details["MapRectHeight"].as_f32())
            .map(|(((top_left_x, top_left_y), width), height)| Rect {
                top_left_x,
                top_left_y,
                width,
                height,
            });

        if let Some((map_id, (continent_rect, map_rect))) = u32::from_str_radix(map_id_str, 10)
            .ok()
            .zip(continent_rect.zip(map_rect))
        {
            dimensions.insert(
                map_id,
                MapDimensions {
                    continent_rect,
                    map_rect,
                },
            );
        }
    }

    dimensions
});

pub fn get_map_dimensions(map_id: u32) -> Option<&'static MapDimensions> {
    MAP_DIMENSIONS.get(&map_id)
}

pub struct MapDimensions {
    continent_rect: Rect,
    map_rect: Rect,
}

impl MapDimensions {
    pub fn transform_map_coordinates_to_world_coordinates(
        &self,
        CoordinatesRelativeToMapRectCenter(point): &CoordinatesRelativeToMapRectCenter,
    ) -> WorldCoordinates {
        let x = self.continent_rect.top_left_x
            + (point.x * METER_CONVERSION - self.map_rect.top_left_x) / self.map_rect.width
                * self.continent_rect.width;

        let y = self.continent_rect.top_left_y
            - (point.y * METER_CONVERSION - self.map_rect.top_left_y) / self.map_rect.height
                * self.continent_rect.height;

        WorldCoordinates(Point2::new(x, y))
    }
}

pub struct Rect {
    top_left_x: f32,
    top_left_y: f32,
    width: f32,
    height: f32,
}

const METER_CONVERSION: f32 = 1.0 / 254.0 * 10000.0;
