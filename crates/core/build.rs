use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use windows::Foundation::Numerics::Matrix3x2;

include!("src/maps/shared_types.rs");

fn main() {
    generate_static_map_dimensions();
}

fn generate_static_map_dimensions() {
    let mut builder = phf_codegen::Map::<u32>::new();

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("map-dimensions.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let all_dimensions: std::collections::HashMap<u32, MapDimensions> =
        serde_json::from_slice(include_bytes!("./map-dimensions.json"))
            .expect("could not parse dimensions");

    for (map_id, dimensions) in all_dimensions.iter() {
        let Matrix3x2 {
            M11,
            M12,
            M21,
            M22,
            M31,
            M32,
        } = calculate_map_to_world_transformation_matrix(dimensions);

        builder.entry(
            *map_id,
            &format!(
                "Matrix3x2 {{ M11: {M11}_f32, M12: {M12}_f32, M21: {M21}_f32, M22: {M22}_f32, M31: {M31}_f32, M32: {M32}_f32 }}",
            ),
        );
    }

    writeln!(&mut file, "use windows::Foundation::Numerics::Matrix3x2;").unwrap();
    writeln!(
        &mut file,
        "pub static MAP_TO_WORLD_TRANSFORMATION_MATRICES: phf::Map<u32, Matrix3x2> = {};",
        builder.build()
    )
    .unwrap();
}

fn calculate_map_to_world_transformation_matrix(
    dimensions: &MapDimensions,
) -> Matrix3x2 {
    const METER_TO_INCH_TRANSFORMATION: Matrix3x2 = {
        const METER_TO_INCH: f32 = 1.0 / 254.0 * 10000.0;

        Matrix3x2::new(METER_TO_INCH, 0.0, 0.0, METER_TO_INCH, 0.0, 0.0)
    };

    const INVERSE_Y: Matrix3x2 = Matrix3x2::new(1.0, 0.0, 0.0, -1.0, 0.0, 0.0);

    let translate_map_rect_top_left = Matrix3x2::translation(
        -dimensions.map_rect.top_left[0],
        -dimensions.map_rect.top_left[1],
    );

    let scale_map_rect = Matrix3x2::new(
        1.0 / dimensions.map_rect.width,
        0.0,
        0.0,
        1.0 / dimensions.map_rect.height,
        0.0,
        0.0,
    );

    let scale_continent_rect = Matrix3x2::new(
        dimensions.continent_rect.width,
        0.0,
        0.0,
        dimensions.continent_rect.height,
        0.0,
        0.0,
    );

    let translate_continent_rect_top_left = Matrix3x2::translation(
        dimensions.continent_rect.top_left[0],
        dimensions.continent_rect.top_left[1],
    );

    METER_TO_INCH_TRANSFORMATION
        * translate_map_rect_top_left
        * scale_map_rect
        * scale_continent_rect
        * INVERSE_Y
        * translate_continent_rect_top_left
}

// This needs to be here because the windows crate is only available for
// windows targets but code generation does not run as a windows target?
mod windows {
    #[allow(non_snake_case)]
    pub mod Foundation {
        pub mod Numerics {
            // Copied from the windows crate.
            pub struct Matrix3x2 {
                pub M11: f32,
                pub M12: f32,
                pub M21: f32,
                pub M22: f32,
                pub M31: f32,
                pub M32: f32,
            }

            impl Matrix3x2 {
                pub const fn new(
                    M11: f32,
                    M12: f32,
                    M21: f32,
                    M22: f32,
                    M31: f32,
                    M32: f32,
                ) -> Self {
                    Self {
                        M11,
                        M12,
                        M21,
                        M22,
                        M31,
                        M32,
                    }
                }
            }

            // Copied from the windows crate.
            impl Matrix3x2 {
                pub const fn translation(x: f32, y: f32) -> Self {
                    Self {
                        M11: 1.0,
                        M12: 0.0,
                        M21: 0.0,
                        M22: 1.0,
                        M31: x,
                        M32: y,
                    }
                }

                fn impl_mul(&self, rhs: &Self) -> Self {
                    Self {
                        M11: self.M11 * rhs.M11 + self.M12 * rhs.M21,
                        M12: self.M11 * rhs.M12 + self.M12 * rhs.M22,
                        M21: self.M21 * rhs.M11 + self.M22 * rhs.M21,
                        M22: self.M21 * rhs.M12 + self.M22 * rhs.M22,
                        M31: self.M31 * rhs.M11 + self.M32 * rhs.M21 + rhs.M31,
                        M32: self.M31 * rhs.M12 + self.M32 * rhs.M22 + rhs.M32,
                    }
                }
            }

            // Copied from the windows crate.
            impl core::ops::Mul<Matrix3x2> for Matrix3x2 {
                type Output = Matrix3x2;
                fn mul(self, rhs: Matrix3x2) -> Matrix3x2 {
                    self.impl_mul(&rhs)
                }
            }
        }
    }
}
