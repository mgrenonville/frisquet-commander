use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "length: u8", id = "length")]
pub enum SondePayload {
    #[deku(id = "17")]
    SondeTemperatureMessage {
        data: [u8; 9],
        #[deku(endian = "big")]
        temperature: i16,
    },

    #[deku(id = "6")]
    SondeAssociationAnnounceMessage {
        #[deku(count = "0")]
        data: Vec<u8>,
    },

    #[deku(id = "8")]
    SondeInitMessage {
        #[deku(count = "2")]
        data: Vec<u8>,
    },
    #[deku(id_pat = "_")]
    SondeUnknownMessage {
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
        let payload = unhexify("118020ba4001179c540004a029000102005c");
        // let (_, payload) = dbg_dmp(parse_data, "data")(&payload.as_slice()).unwrap();

        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();
        let (rest, message) =
            SondePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();

        assert_eq!(
            metadata,
            FrisquetMetadata {
                length: 17,
                to_addr: 128,
                from_addr: 32,
                request_id: 47680,
                req_or_answer: 1,
                msg_type: 23
            }
        );
        assert_eq!(
            message,
            SondePayload::SondeTemperatureMessage {
                data: [156, 84, 0, 4, 160, 41, 0, 1, 2],
                temperature: 92
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
    fn test_announce_response() {
        let payload = hex::decode("06802020948241").unwrap();

        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();
        let (rest, message) =
            SondePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();
        assert_eq!(
            metadata,
            FrisquetMetadata {
                length: 6,
                to_addr: 128,
                from_addr: 32,
                request_id: 8340,
                req_or_answer: 130,
                msg_type: 65
            }
        );
        assert_eq!(
            message,
            SondePayload::SondeAssociationAnnounceMessage { data: vec![] }
        );
        println!("{metadata:?}");
        println!("{message:?}");
    }

    #[test]
    fn test_init() {
        let payload = hex::decode("088020830001430000").unwrap();

        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();
        let (rest, message) =
            SondePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();
        assert_eq!(
            metadata,
            FrisquetMetadata {
                length: 8,
                to_addr: 128,
                from_addr: 32,
                request_id: 33536,
                req_or_answer: 1,
                msg_type: 67
            }
        );
        assert_eq!(message, SondePayload::SondeInitMessage { data: vec![0, 0] });
        println!("{metadata:?}");
        println!("{message:?}");
        let (rest, metadata) = FrisquetMetadata::from_bytes((payload.as_ref(), 0)).unwrap();
        let (_rest, message) =
            SondePayload::read(deku::bitvec::BitSlice::from_slice(rest.0), metadata.length)
                .unwrap();
        assert_eq!(
            metadata,
            FrisquetMetadata {
                length: 8,
                to_addr: 128,
                from_addr: 32,
                request_id: 33536,
                req_or_answer: 1,
                msg_type: 67
            }
        );
        assert_eq!(message, SondePayload::SondeInitMessage { data: vec![0, 0] });
        println!("{metadata:?}");
        println!("{message:?}");
    }
}
