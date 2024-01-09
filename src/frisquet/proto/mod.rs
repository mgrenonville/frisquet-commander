use deku::prelude::*;

use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::satellite::SatellitePayload;
use crate::frisquet::proto::sonde::SondePayload;

pub mod common;
pub mod satellite;

pub mod chaudiere;
pub mod sonde;

#[derive(Debug, PartialEq)]
pub enum FrisquetData {
    Satellite(SatellitePayload),
    Chaudiere(ChaudierePayload),
    Sonde(SondePayload),
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct FrisquetMetadata {
    pub length: u8,
    pub to_addr: u8,
    pub from_addr: u8,
    pub request_id: u16,
    pub req_or_answer: u8,
    pub msg_type: u8,
}
