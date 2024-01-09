use deku::prelude::*;

use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::common::unhexify;
use crate::frisquet::proto::satellite::SatellitePayload;
use crate::frisquet::proto::sonde::SondePayload;
use crate::frisquet::proto::{FrisquetData, FrisquetMetadata};

pub mod proto;

pub fn parse_data_from_str(
    input: &str,
) -> Result<(FrisquetMetadata, FrisquetData), deku::DekuError> {
    let payload = unhexify(input);
    let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0))?;
    let rest = deku::bitvec::BitSlice::from_slice(rest.0);
    match metadata.from_addr {
        // Satellite
        0x08..=0x0a => {
            let (_, payload) = SatellitePayload::read(rest, metadata.length)?;
            Ok((metadata, FrisquetData::Satellite(payload)))
        }
        // Sonde
        32 => {
            let (_, payload) = SondePayload::read(rest, metadata.length)?;
            Ok((metadata, FrisquetData::Sonde(payload)))
        }
        // Chaudiere
        0x80 => {
            let (_, payload) = ChaudierePayload::read(rest, metadata.length)?;
            Ok((metadata, FrisquetData::Chaudiere(payload)))
        }
        _ => panic!("Unknown"),
    }
}
