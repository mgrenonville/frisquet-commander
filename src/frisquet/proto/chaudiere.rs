use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "length: u8", id = "length")]
pub enum ChaudierePayload {
    #[deku(id = "11")]
    ChaudiereAssociationBroadcast { unknown: u8, network_id: [u8; 4] },
    #[deku(id = "15")]
    ChaudiereSondeResponseMessage {
        unknown_start: u8,
        year: u8,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        #[deku(count = "(length - 6) as usize - deku::byte_offset")]
        data: Vec<u8>,
    },
    #[deku(id = "49")]
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
        static_part_2: [u8; 20],
    },
    #[deku(id_pat = "55")]
    ChaudiereToSatelliteUnknownMessageResponse {
        #[deku(count = "length - 6")]
        data: Vec<u8>,
    },

    #[deku(id_pat = "_")]
    ChaudiereUnknownMessage {
        #[deku(count = "length - 6")]
        data: Vec<u8>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::frisquet::proto::common::unhexify;
    use crate::frisquet::proto::FrisquetMetadata;

    #[test]
    fn test() {
        let payload = unhexify("310880194881172A050A0000230423171012000000C000BE002500C600C604F6000000000000000004F60000000000000000");

        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();
        let (_rest, message) =
            ChaudierePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();

        assert_eq!(
            metadata,
            FrisquetMetadata {
                length: 49,
                to_addr: 8,
                from_addr: 128,
                request_id: 6472,
                req_or_answer: 129,
                msg_type: 23,
            }
        );
        assert_eq!(
            message,
            ChaudierePayload::ChaudiereSetTemperatureMessageResponse {
                unknown_start: [42, 5],
                temperature_exterieure: 10,
                unknown: 0,
                year: 0x23,
                month: 0x4,
                day: 0x23,
                hour: 23,
                minute: 16,
                second: 18,
                unknown_1: [0, 0, 0],
                temperature: 192,
                consigne: 190,
                unknown_2: [37, 0],
                signature: [198, 0, 198],
                static_part_2: [4, 246, 0, 0, 0, 0, 0, 0, 0, 0, 4, 246, 0, 0, 0, 0, 0, 0, 0, 0],
            }
        );

        let mut res = metadata.to_bytes().unwrap();
        let mut out = deku::bitvec::BitVec::with_capacity(message.deku_id().unwrap() as usize);
        message.write(&mut out, message.deku_id().unwrap()).unwrap();
        let mut out = out.into_vec();
        res.append(&mut out);
        assert_eq!(res, payload);
        assert_eq!(res.len() - 1, metadata.length as usize)
    }

    #[test]
    fn test_broadcast() {
        let payload = hex::decode("0b0080d3c802410405d7199e").unwrap();

        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();
        let (_rest, message) =
            ChaudierePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();
        assert_eq!(
            metadata,
            FrisquetMetadata {
                length: 11,
                to_addr: 0,
                from_addr: 128,
                request_id: 54216,
                req_or_answer: 2,
                msg_type: 65
            }
        );
        assert_eq!(
            message,
            ChaudierePayload::ChaudiereAssociationBroadcast {
                unknown: 4,
                network_id: [5, 215, 25, 158]
            }
        );
        println!("{metadata:?}");
        println!("{message:?}");
    }
    #[test]
    fn test_response() {
        let payload = hex::decode("0f2080ba408117082304051131172803").unwrap();

        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();
        let (_rest, message) =
            ChaudierePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();
        assert_eq!(
            metadata,
            FrisquetMetadata {
                length: 15,
                to_addr: 32,
                from_addr: 128,
                request_id: 47680,
                req_or_answer: 129,
                msg_type: 23
            }
        );
        assert_eq!(
            message,
            ChaudierePayload::ChaudiereSondeResponseMessage {
                unknown_start: 8,
                year: 35,
                month: 4,
                day: 5,
                hour: 17,
                minute: 49,
                second: 23,
                data: [40, 3].to_vec()
            }
        );
        println!("{metadata:?}");
        println!("{message:?}");
    }
}
