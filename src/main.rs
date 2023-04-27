mod messages;
pub mod frisquet;

use deku::prelude::*;
use binascii::{bin2hex, hex2bin};
use hex;

use std::{env, process, time, time::Duration};
use std::fmt::Debug;
use std::thread::sleep;
use mqtt::{Client, Message, Receiver};

use frisquet::proto::FrisquetMetadata;

extern crate paho_mqtt as mqtt;

use serde_json::Result;
use serde_json::Value::String;
use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::FrisquetData;
use crate::frisquet::proto::satellite::SatellitePayload;
use crate::frisquet::proto::sonde::SondePayload;
use crate::messages::{CommandMessage, Listen, SendData, SetNetworkId, Sleep};


const DFLT_BROKER: &str = "tcp://192.168.254.17:1883";
const DFLT_CLIENT: &str = "rust_publish";
const DFLT_TOPICS: &str = "frisquet/receive";


fn main() {
    println!("Hello, world!");
    let host = env::args().nth(1).unwrap_or_else(||
        DFLT_BROKER.to_string()
    );

// Define the set of options for the create.
// Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(DFLT_CLIENT.to_string())
        .finalize();

// Create a client.
    let cli = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });
    let rx: Receiver<Option<Message>> = cli.start_consuming();
// Define the set of options for the connection.
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

// Connect and wait for it to complete or fail.
    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    if let Err(e) = cli.subscribe(DFLT_TOPICS, 0) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }

    // startAssociation(&cli, &rx);
    sendTemperatureExt(&cli, &rx);
}


fn publish(cli: &Client, value: &dyn CommandMessage) {
    let json = serde_json::to_vec(
        value
    ).unwrap();

    if let Err(e) = cli.publish(Message::new("frisquet/command",
                                             json,
                                             0)) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }
}

fn sendData<T>(cli: &Client, from: u8, to: u8, request_id: u16, req_or_answer: u8, msg_type: u8, message: T)
    where
        T: for<'a> DekuEnumExt<'a, (u8)> + DekuWrite<(u8)> + Debug

{
    let length = message.deku_id().unwrap();
    let mut out = deku::bitvec::BitVec::with_capacity(length as usize);
    message.write(&mut out, length).unwrap();
    let mut out = out.into_vec();

    let metadata = FrisquetMetadata {
        length,
        to_addr: to,
        from_addr: from,
        request_id: request_id,
        req_or_answer: req_or_answer,
        msg_type: msg_type,
    };
    let mut res = metadata.to_bytes().unwrap();
    res.append(&mut out);
    let payload = hex::encode(res);
    println!("send: {metadata:?}, {message:?}, payload: {payload:?}");


    publish(cli, &SendData { payload: payload })
}


fn awaitMessage(rx: &Receiver<Option<Message>>) -> (FrisquetMetadata, FrisquetData) {
    loop {
        for msg in rx.iter() {
            if let Some(msg) = msg {
                let data: messages::DataMessage = serde_json::from_str(msg.payload_str().as_ref()).unwrap();
                let parsed = frisquet::parse_data_from_str(data.data.as_str()).unwrap();
                println!("{parsed:?}");
                return parsed;
            }
        }
    }
}

fn sendTemperatureExt(cli: &Client, rx: &Receiver<Option<Message>>) {
    let network_id = [5, 218, 46, 226];
    publish(&cli, &Sleep {});
    publish(&cli, &SetNetworkId { network_id: hex::encode(network_id) });
    sleep(time::Duration::from_millis(10000));
    publish(&cli, &Listen {});
    sleep(time::Duration::from_millis(10000));
    sendData(cli, 32, 128, 3612, 1, 23, SondePayload::SondeTemperatureMessage { data: [156, 84, 0, 4, 160, 41, 0, 1,2], temperature: 120 });
    if let (metadata, FrisquetData::Chaudiere(data)) = awaitMessage(rx) {
        println!("Received: {metadata:?} data: {data:?}")

    }
    publish(&cli, &Sleep {});

}

fn startAssociation(cli: &Client, rx: &Receiver<Option<Message>>) {
    publish(&cli, &SetNetworkId { network_id: "ffffffff".to_string() });
    publish(&cli, &Listen {});
    if let (metadata, FrisquetData::Chaudiere(data)) = awaitMessage(rx) {
        publish(&cli, &Sleep {});
        if let ChaudierePayload::ChaudiereAssociationBroadcast { unknown, network_id } = data {
            sendData(cli, 32, 128, metadata.request_id, metadata.req_or_answer + 0x80, metadata.msg_type, SatellitePayload::SatelliteUnknownMessage { data: vec![01, 27, 00, 02] });
            publish(&cli, &SetNetworkId { network_id: hex::encode(network_id).to_string() });


            sendData(
                cli, // le client MQTT
                32, // from
                128, // to
                metadata.request_id, // request_id présent dans la trame de la chaudière
                metadata.req_or_answer + 0x80, // +0x80 en cas de réponse
                metadata.msg_type, // le type du message en provenance de la chaudière
                SondePayload::SondeTemperatureMessage {
                    data: [156, 84, 0, 4, 160, 41, 0, 1,2], // des données qui semblent fixes
                    temperature: 130 // la température exterieure, * 10
                }
            );
            publish(&cli, &Listen {});
            if let (metadata, FrisquetData::Chaudiere(data)) = awaitMessage(rx) {
                println!("Received: {metadata:?} data: {data:?}")

            }


        }


    }
}

