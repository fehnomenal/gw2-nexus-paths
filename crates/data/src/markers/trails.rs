use nom::{
    combinator::verify,
    multi::many1,
    number::complete::{le_f32, le_i32},
    sequence::{preceded, tuple},
    IResult, Parser,
};
use paths_types::{Point3, Trail};

fn parse_i32(input: &[u8]) -> IResult<&[u8], i32> {
    le_i32(input)
}

fn parse_version(input: &[u8]) -> IResult<&[u8], i32> {
    verify(parse_i32, |version| *version == 0)(input)
}

fn parse_header(input: &[u8]) -> IResult<&[u8], i32> {
    preceded(parse_version, parse_i32)(input)
}

fn parse_f32(input: &[u8]) -> IResult<&[u8], f32> {
    le_f32(input)
}

fn parse_point(input: &[u8]) -> IResult<&[u8], Point3> {
    tuple((parse_f32, parse_f32, parse_f32))
        .map(|(x, y, z)| Point3::new(x, z, y))
        .parse(input)
}

pub fn parse_trail(input: &[u8]) -> IResult<&[u8], Trail> {
    parse_header
        .and(many1(parse_point))
        .map(|(map_id, points)| Trail { map_id, points })
        .parse(input)
}
