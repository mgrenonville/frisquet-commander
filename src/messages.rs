extern crate serde;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct DataMessage {
    pub data: String,
}

#[typetag::serde(tag = "type")]
trait CommandMessage {}

#[derive(Serialize, Deserialize)]
struct Listen {}

#[typetag::serde(name = "LISTEN")]
impl CommandMessage for Listen {}


#[derive(Serialize, Deserialize)]
struct SetNetworkId {
    networkId: String,
}

#[typetag::serde(name = "SET_NETWORK_ID")]
impl CommandMessage for SetNetworkId {}


#[derive(Serialize, Deserialize)]
struct SendData {
    data: String,
}

#[typetag::serde(name = "SEND")]
impl CommandMessage for SendData {}


