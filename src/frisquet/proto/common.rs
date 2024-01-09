use nom::bytes::complete::take_while_m_n;
use nom::combinator::map_res;
use nom::error::{ErrorKind, ParseError};
use nom::multi::{count, many0};
use nom::number::complete::{be_u16, be_u8};
use nom::Err::Error;
use nom::IResult;

use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::{FrisquetData, FrisquetMetadata, SatellitePayload};
use hexlit::hex;

#[derive(Debug, PartialEq)]
pub enum CustomError<I> {
    MyError,
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for CustomError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        CustomError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

pub fn unhexify(input: &str) -> Vec<u8> {
    let result = many0(hex_primary)(input).unwrap();
    return result.1;
    // hex!(input).to_vec()
}
