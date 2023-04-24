use nom::bytes::complete::take_while_m_n;
use nom::combinator::map_res;
use nom::Err::Error;
use nom::error::{ErrorKind, ParseError};
use nom::IResult;
use nom::multi::{count, many0};
use nom::number::complete::{be_u16, be_u8};

use crate::frisquet::proto::{FrisquetData, FrisquetPayload, SatellitePayload};
use crate::frisquet::proto::chaudiere::{ChaudierePayload, parse_set_temperature_message_response};
use crate::frisquet::proto::satellite::parse_set_temperature_message;

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
    map_res(
        take_while_m_n(2, 2, is_hex_digit),
        from_hex,
    )(input)
}

pub fn unhexify(input: &str) -> IResult<&str, Vec<u8>> {
    let result = many0(hex_primary)(input)?;
    return Ok((result.0, result.1));
}

fn parse_chaudiere_payload(length: u8, input: &[u8]) -> IResult<&[u8], FrisquetData, CustomError<&[u8]>> {
    match length {
        49 => {
            let (input, x) = parse_set_temperature_message_response(input).expect("Valid ");
            Ok((input, FrisquetData::Chaudiere(x)))
        }
        _ =>
            Err(Error(CustomError::MyError))
    }
}

fn parse_satellite_payload(length: u8, input: &[u8]) -> IResult<&[u8], FrisquetData, CustomError<&[u8]>> {
    match length {
        17 => {
            let (input, static_part) = count(be_u8, 7)(input)?;
            let (input, message_part) = count(be_u8, 3)(input)?;
            Ok((input, FrisquetData::Satellite(SatellitePayload::SatelliteInitMessage {
                static_part: static_part.as_slice().try_into().unwrap(),
                message_part: message_part.as_slice().try_into().unwrap(),
            })))
        }

        23 => {
            let (input, x) = parse_set_temperature_message(input).expect("Valid setTemperatureMessage");
            Ok((input, FrisquetData::Satellite(x)))
        }

        _ => {
            Err(Error(CustomError::MyError))
        }
    }
}

pub fn parse_data(input: &[u8]) -> IResult<&[u8], FrisquetPayload, CustomError<&[u8]>> {
    let (input, length) = be_u8(input)?;
    let (input, to_addr) = be_u8(input)?;
    let (input, from_addr) = be_u8(input)?;
    let (input, request_id) = be_u16(input)?;
    let (input, req_or_answer) = be_u8(input)?;
    let (input, msg_type) = be_u8(input)?;


    let (input, payload) = match from_addr {
        0x08..=0x0a =>
            parse_satellite_payload(length, input)?,

        0x80 =>
            parse_chaudiere_payload(length, input)?,
        // 0x20 =>
        // 0x7e =>
        _ => {
            panic!("Not implemented")
        }
    };

    Ok((input, FrisquetPayload {
        length,
        to_addr,
        from_addr,
        request_id,
        req_or_answer,
        msg_type,
        data: payload,
    }))
}

