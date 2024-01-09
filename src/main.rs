extern crate paho_mqtt as mqtt;

use std::collections::HashMap;
use std::fmt::Debug;
use std::thread::sleep;
use std::{env, process, time, time::Duration};

use binascii::{bin2hex, hex2bin};
use deku::prelude::*;
use hex;
use mqtt::{Client, Message, Receiver};
use serde_json::Result;

use config::Config;
use frisquet::proto::FrisquetMetadata;

// use serde_json::Value::String;
use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::satellite::SatellitePayload;
use crate::frisquet::proto::sonde::SondePayload;
use crate::frisquet::proto::FrisquetData;
use crate::messages::{CommandMessage, Listen, SendData, SetNetworkId, Sleep};

pub mod frisquet;
mod messages;

fn main() {
    println!("Hello, world!");
    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("config"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| settings.get("broker").unwrap().to_string());

    // Define the set of options for the create.
    // Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(settings.get("mqtt_client").unwrap().to_string())
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

    if let Err(e) = cli.subscribe(settings.get("mqtt_frisquet_topic").unwrap(), 0) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }

    // startAssociation(&cli, &rx);
    // sendTemperatureExt(&cli, &rx, true);
    let network_id = settings.get("network_id").unwrap().to_string();

    publish(&cli, &SetNetworkId { network_id });
    sleep(time::Duration::from_millis(1000));
    publish(&cli, &Listen {});

    loop {
        println!("loop");
        let (metadata, x) = awaitMessage(&rx);
        // publish(&cli, &Listen {});
        //
        println!("Received: {metadata:?} data: {x:?}");
        // if (metadata.length == 8 && metadata.to_addr == 32) {
        //     println!("Send announce message");
        //     sendData(&cli, 32, 128, metadata.request_id, metadata.req_or_answer + 0x80, metadata.msg_type, SondePayload::SondeAssociationAnnounceMessage { data: vec![] });
        // }
    }
}

fn publish(cli: &Client, value: &dyn CommandMessage) {
    let json = serde_json::to_vec(value).unwrap();

    if let Err(e) = cli.publish(Message::new("frisquet/command", json, 0)) {
        println!("Error subscribes topics: {:?}", e);
        process::exit(1);
    }
}

fn sendData<T>(
    cli: &Client,
    from: u8,
    to: u8,
    request_id: u16,
    req_or_answer: u8,
    msg_type: u8,
    message: T,
) where
    T: for<'a> DekuEnumExt<'a, (u8)> + DekuWrite<(u8)> + Debug,
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
                let data: messages::DataMessage =
                    serde_json::from_str(msg.payload_str().as_ref()).unwrap();
                let parsed = frisquet::parse_data_from_str(data.data.as_str()).unwrap();
                // println!("{parsed:?}");
                return parsed;
            }
        }
    }
}

fn sendTemperatureExt(cli: &Client, rx: &Receiver<Option<Message>>, plug: bool) {
    let network_id = [5, 218, 46, 226];
    publish(&cli, &Sleep {});
    publish(
        &cli,
        &SetNetworkId {
            network_id: hex::encode(network_id),
        },
    );
    sleep(time::Duration::from_millis(1000));
    publish(&cli, &Listen {});
    sleep(time::Duration::from_millis(1000));
    if (plug) {
        sendData(
            cli,
            32,
            128,
            33536,
            1,
            67,
            SondePayload::SondeInitMessage { data: vec![0, 0] },
        );
        let received = awaitMessage(rx);
        println!("Received: {received:?}");
    }
    sleep(time::Duration::from_millis(3000));

    sendData(
        cli,
        32,
        128,
        6648,
        1,
        23,
        SondePayload::SondeTemperatureMessage {
            data: [156, 84, 0, 4, 160, 41, 0, 1, 2],
            temperature: 190,
        },
    );
    if let (metadata, FrisquetData::Chaudiere(data)) = awaitMessage(rx) {
        println!("Received: {metadata:?} data: {data:?}")
    }
    publish(&cli, &Sleep {});
}

fn startAssociation(cli: &Client, rx: &Receiver<Option<Message>>) {
    publish(
        &cli,
        &SetNetworkId {
            network_id: "ffffffff".to_string(),
        },
    );
    println!("Setting network_id to ffffffff");
    publish(&cli, &Listen {});
    println!("Waiting broadcast networkId message");

    if let (metadata, FrisquetData::Chaudiere(data)) = awaitMessage(rx) {
        publish(&cli, &Sleep {});
        println!("Received a message {data:?}");

        if let ChaudierePayload::ChaudiereAssociationBroadcast {
            unknown,
            network_id,
        } = data
        {
            println!("This is a ChaudiereAssociationBroadcast message, will announce");
            sendData(
                cli,
                32,
                128,
                metadata.request_id,
                metadata.req_or_answer + 0x80,
                metadata.msg_type,
                SondePayload::SondeAssociationAnnounceMessage { data: vec![] },
            );
            let networkIdStr = hex::encode(network_id).to_string();
            println!("Switch to networkId {networkIdStr}");

            publish(
                &cli,
                &SetNetworkId {
                    network_id: networkIdStr,
                },
            );
            sleep(time::Duration::from_millis(200));
            println!("Start listening before sending message");
            publish(&cli, &Listen {});
            sleep(time::Duration::from_millis(200));
            sendData(
                cli,
                32,
                128,
                33536,
                1,
                67,
                SondePayload::SondeInitMessage { data: vec![0, 0] },
            );
            loop {
                sleep(time::Duration::from_millis(1000));
                sendData(
                    cli,   // le client MQTT
                    32,    // from
                    128,   // to
                    34692, // request_id présent dans la trame de la chaudière
                    1,     // +0x80 en cas de réponse
                    23,    // le type du message en provenance de la chaudière
                    SondePayload::SondeTemperatureMessage {
                        data: [156, 84, 0, 4, 160, 41, 0, 1, 2], // des données qui semblent fixes
                        temperature: 180,                        // la température exterieure, * 10
                    },
                );
                if let (metadata, FrisquetData::Chaudiere(data)) = awaitMessage(rx) {
                    println!("Received: {metadata:?} data: {data:?}");
                    return;
                }
            }
        }
    }
}
