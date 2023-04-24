use nom::bits::{bits, streaming::{bool, take}};
use nom::combinator::rest;
use nom::error::{dbg_dmp, Error};
use nom::IResult;
use nom::multi::count;
use nom::number::complete::{be_u16, be_u8, le_i16, le_u16, le_u8};
use nom::sequence::tuple;

#[derive(Debug, PartialEq)]
pub enum ChaudierePayload {
    ChaudiereSetTemperatureMessageResponse {
        unknown_start: [u8; 2],
        temperature_exterieure: i16,
        unknown: u8,
        year: u8,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        unknown_1: [u8; 3],
        temperature: i16,
        consigne: i16,
        unknown_2: [u8; 2],
        signature: [u8; 3],
        static_part_2: [u8; 19],

    },
}


pub fn parse_set_temperature_message_response(input: &[u8]) -> IResult<&[u8], ChaudierePayload> {
    let (input, unknown_start) = count(be_u8, 2)(input)?;
    let (input, temperature_exterieure) = le_i16(input)?;
    let (input, unknown) = le_u8(input)?;
    let (input, year) = le_u8(input)?;
    let (input, month) = le_u8(input)?;
    let (input, day) = le_u8(input)?;
    let (input, hour) = le_u8(input)?;
    let (input, minute) = le_u8(input)?;
    let (input, second) = le_u8(input)?;
    let (input, unknown_1) = count(be_u8, 3)(input)?;
    let (input, temperature) = le_i16(input)?;
    let (input, consigne) = le_i16(input)?;
    let (input, unknown_2) = count(be_u8, 2)(input)?;
    let (input, signature) = count(be_u8, 3)(input)?;
    let (input, static_part_2) = count(be_u8, 19)(input)?;
    let (input, remaining) = rest(input)?;
    Ok((input, ChaudierePayload::ChaudiereSetTemperatureMessageResponse {
        unknown_start: unknown_start.as_slice().try_into().unwrap(),
        temperature_exterieure: temperature_exterieure,
        unknown: unknown,
        year: format!("{year:X}").parse::<u8>().unwrap(),
        month: format!("{month:X}").parse::<u8>().unwrap(),
        day: format!("{day:X}").parse::<u8>().unwrap(),
        hour: hour,
        minute: minute,
        second: second,
        unknown_1: unknown_1.as_slice().try_into().unwrap(),
        temperature: temperature,
        consigne: consigne,
        unknown_2: unknown_2.as_slice().try_into().unwrap(),
        signature: signature.as_slice().try_into().unwrap(),
        static_part_2: static_part_2.as_slice().try_into().unwrap(),

    }))
}


use crate::frisquet::proto::common::{parse_data, unhexify};
use crate::frisquet::proto::FrisquetData;

#[test]
fn test() {
    let (_, payload) = unhexify("310880194881172A050A0000230423171012000000C000BE002500C600C604F6000000000000000004F60000000000000000").unwrap();
    let (_, payload) = dbg_dmp(parse_data, "data")(&payload.as_slice()).unwrap();
    assert_eq!(payload.length, 49);
    assert_eq!(payload.from_addr, 128);
    assert_eq!(payload.to_addr, 8);
    assert_eq!(payload.request_id, 6472);
    assert_eq!(payload.req_or_answer, 129);
    assert_eq!(payload.msg_type, 23);
    // if let FrisquetData::Chaudiere(x) = payload.data {
    //     // let c = x as ChaudierePayload::ChaudiereSetTemperatureMessageResponse;
    //
    //
    //
    //
    // };


    println!("Parsed input: {payload:?}");
}
