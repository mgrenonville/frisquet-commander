use nom::bits::{bits, streaming::{bool, take}};
use nom::error::{dbg_dmp, Error};
use nom::IResult;
use nom::multi::count;
use nom::number::complete::{be_u16, be_u8, le_i16, le_u16};
use nom::sequence::tuple;

use crate::frisquet::proto::common::{parse_data, unhexify};

#[derive(Debug, PartialEq)]
pub enum SatellitePayload {
    SatelliteInitMessage {
        static_part: [u8; 7],
        message_part: [u8; 3],
    },

    SatelliteSetTemperatureMessage {
        static_part: [u8; 3],
        unknown1: u8,
        static_part_end: [u8; 3],
        unknown2: u8,
        message_static_part: [u8; 2],
        temperature: i16,
        consigne: i16,
        unknown_mode1: u8,
        hors_gel: bool,
        unknown_mode2: u8,
        derogation: bool,
        soleil: bool,
        signature: [u8; 2],
    },
}


pub fn parse_set_temperature_message(input: &[u8]) -> IResult<&[u8], SatellitePayload> {
    let (input, static_part) = count(be_u8, 3)(input)?;
    let (input, unknown1) = be_u8(input)?;
    let (input, static_part_end) = count(be_u8, 3)(input)?;
    let (input, unknown2) = be_u8(input)?;
    let (input, message_static_part) = count(be_u8, 2)(input)?;
    let (input, temperature) = le_i16(input)?;
    let (input, consigne) = le_i16(input)?;
    let (input, (unknown_mode1, hors_gel, unknown_mode2, derogation, soleil)) = bits::<_, _, Error<(&[u8], usize)>, _, _>(tuple((take(3usize), bool, take(2usize), bool, bool)))(input)?;
    let (input, signature) = count(be_u8, 2)(input)?;


    Ok((input, SatellitePayload::SatelliteSetTemperatureMessage {
        static_part: static_part.as_slice().try_into().unwrap(),
        unknown1: unknown1,
        static_part_end: static_part_end.as_slice().try_into().unwrap(),
        unknown2: unknown2,
        message_static_part: message_static_part.as_slice().try_into().unwrap(),
        temperature: temperature,
        consigne: consigne,
        unknown_mode1: unknown_mode1,
        hors_gel: hors_gel,
        unknown_mode2: unknown_mode2,
        derogation: derogation,
        soleil: soleil,
        signature: signature.as_slice().try_into().unwrap(),
    }))
}


#[test]
fn test() {
    let (_, payload) = unhexify("17800819E40117A0290015A02F00040800B200AA002400C6").unwrap();
    let (_, payload) = dbg_dmp(parse_data, "data")(&payload.as_slice()).unwrap();
    assert_eq!(payload.length, 23);
    assert_eq!(payload.from_addr, 8);
    assert_eq!(payload.to_addr, 128);
    assert_eq!(payload.request_id, 6628);
    assert_eq!(payload.req_or_answer, 1);
    assert_eq!(payload.msg_type, 23);
    println!("Parsed input: {payload:?}");
}