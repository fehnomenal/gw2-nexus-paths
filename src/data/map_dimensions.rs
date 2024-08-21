use std::{collections::HashMap, sync::LazyLock};

use windows::Foundation::Numerics::Matrix3x2;

static MAP_DIMENSIONS: LazyLock<HashMap<u32, MapDimensions>> = LazyLock::new(|| {
    let raw =
        jzon::parse(include_str!("../../mapDetails.json")).expect("Could not parse map details");

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
            dimensions.insert(map_id, MapDimensions::new(continent_rect, map_rect));
        }
    }

    dimensions
});

pub fn get_map_dimensions(map_id: u32) -> Option<&'static MapDimensions> {
    MAP_DIMENSIONS.get(&map_id)
}

pub struct MapDimensions {
    pub map_to_world_transformation: Matrix3x2,
}

impl MapDimensions {
    fn new(continent_rect: Rect, map_rect: Rect) -> Self {
        let translate_map_rect_top_left =
            Matrix3x2::translation(-map_rect.top_left_x, -map_rect.top_left_y);

        let scale_map_rect = Matrix3x2 {
            M11: 1.0 / map_rect.width,
            M12: 0.0,
            M21: 0.0,
            M22: 1.0 / map_rect.height,
            M31: 0.0,
            M32: 0.0,
        };

        let scale_continent_rect = Matrix3x2 {
            M11: continent_rect.width,
            M12: 0.0,
            M21: 0.0,
            M22: continent_rect.height,
            M31: 0.0,
            M32: 0.0,
        };

        let translate_continent_rect_top_left =
            Matrix3x2::translation(continent_rect.top_left_x, continent_rect.top_left_y);

        Self {
            map_to_world_transformation: METER_TO_INCH_TRANSFORMATION
                * translate_map_rect_top_left
                * scale_map_rect
                * scale_continent_rect
                * INVERSE_Y
                * translate_continent_rect_top_left,
        }
    }
}

pub struct Rect {
    top_left_x: f32,
    top_left_y: f32,
    width: f32,
    height: f32,
}

const METER_TO_INCH: f32 = 1.0 / 254.0 * 10000.0;

const METER_TO_INCH_TRANSFORMATION: Matrix3x2 = Matrix3x2 {
    M11: METER_TO_INCH,
    M12: 0.0,
    M21: 0.0,
    M22: METER_TO_INCH,
    M31: 0.0,
    M32: 0.0,
};

const INVERSE_Y: Matrix3x2 = Matrix3x2 {
    M11: 1.0,
    M12: 0.0,
    M21: 0.0,
    M22: -1.0,
    M31: 0.0,
    M32: 0.0,
};
