use crate::frisquet::proto::{FrisquetPayload};
use crate::frisquet::proto::common::{parse_data, unhexify};

mod proto;



pub fn parse_data_from_str(input: &str) -> Result<FrisquetPayload, &'static str> {
    let (_, x) = unhexify(input).unwrap();
    let (_, r) = parse_data(x.as_slice()).expect("Issue with parsing");
    Ok(r)
}

