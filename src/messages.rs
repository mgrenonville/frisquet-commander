extern crate serde;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct DataMessage {
    pub data: String,
}

#[typetag::serde(tag = "type")]
pub trait CommandMessage {}

#[derive(Serialize, Deserialize)]
pub struct Listen {}

#[typetag::serde(name = "LISTEN")]
impl CommandMessage for Listen {}



#[derive(Serialize, Deserialize)]
pub struct Sleep {}

#[typetag::serde(name = "SLEEP")]
impl CommandMessage for Sleep {}


#[derive(Serialize, Deserialize)]
pub struct SetNetworkId {
    pub network_id: String,
}

#[typetag::serde(name = "SET_NETWORK_ID")]
impl CommandMessage for SetNetworkId {}


#[derive(Serialize, Deserialize)]
pub struct SendData {
    pub payload: String,
}

#[typetag::serde(name = "SEND")]
impl CommandMessage for SendData {}


