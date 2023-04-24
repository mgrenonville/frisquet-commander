use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::satellite::SatellitePayload;

pub mod satellite;
pub mod common;

pub mod chaudiere;

#[derive(Debug, PartialEq)]
pub enum FrisquetData {
    Satellite(SatellitePayload),
    Chaudiere(ChaudierePayload)
}

#[derive(Debug, PartialEq)]
pub struct FrisquetPayload {
    pub length: u8,
    pub to_addr: u8,
    pub from_addr: u8,
    pub request_id: u16,
    pub req_or_answer: u8,
    pub msg_type: u8,
    pub data: FrisquetData,
}

