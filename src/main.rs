use std::collections::HashMap;
use std::fmt::Debug;
use std::thread::sleep;
use std::time;

use deku::prelude::*;
use hex;

use config::Config;
use frisquet::proto::FrisquetMetadata;

use crate::frisquet::proto::chaudiere::ChaudierePayload;
use crate::frisquet::proto::satellite::SatellitePayload;
use crate::frisquet::proto::sonde::SondePayload;
use crate::frisquet::proto::FrisquetData;
use crate::rf::RFClient;

pub mod rf;

pub mod frisquet;
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

    let mut cli = rf_client(&settings).unwrap();

    // start_association(&cli, &rx);
    // sendTemperatureExt(&cli, &rx, true);
    let network_id = settings.get("network_id").unwrap().to_string();

    cli.set_network_id(hex::decode(network_id).expect("network_id should be hex"))
        .unwrap();
    sleep(time::Duration::from_millis(1000));

    loop {
        let msg = cli.receive().unwrap();
        let (metadata, x) =
            frisquet::parse_data_from_str(hex::encode(msg.clone()).as_str()).unwrap();
        // publish(&cli, &Listen {});
        //
        // println!("{x:?}");
        println!("Received: {metadata:?} data: {x:?}");
        // if (metadata.length == 8 && metadata.to_addr == 32) {
        //     println!("Send announce message");
        //     sendData(&cli, 32, 128, metadata.request_id, metadata.req_or_answer + 0x80, metadata.msg_type, SondePayload::SondeAssociationAnnounceMessage { data: vec![] });
        // }
    }
}

fn rf_client(settings: &HashMap<String, String>) -> Result<Box<dyn RFClient>, String> {
    if settings.get("mqtt_client").is_some() {
        Ok(Box::new(rf::mqtt::new(&settings)?))
    } else if settings.get("serial_port").is_some() {
        Ok(Box::new(rf::serial::new(&settings)?))
    } else {
        Err("no client configured".to_string())
    }
}

fn send_data<T>(
    client: &mut dyn RFClient,
    from: u8,
    to: u8,
    request_id: u16,
    req_or_answer: u8,
    msg_type: u8,
    message: T,
) where
    T: for<'a> DekuEnumExt<'a, u8> + DekuWrite<u8> + Debug,
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
    let mut payload = metadata.to_bytes().unwrap();
    payload.append(&mut out);
    let data = hex::encode(payload.clone());
    println!("send: {metadata:?}, {message:?}, payload: {data:?}");

    client.send(payload).unwrap();
}

fn send_temperature_ext(client: &mut dyn RFClient, plug: bool) {
    let network_id: Vec<u8> = vec![5, 218, 46, 226];
    client.sleep().unwrap();
    client.set_network_id(network_id).unwrap();
    sleep(time::Duration::from_millis(1000));

    if plug {
        send_data(
            client,
            32,
            128,
            33536,
            1,
            67,
            SondePayload::SondeInitMessage { data: vec![0, 0] },
        );
        let msg = client.receive().unwrap();
        let (metadata, x) = frisquet::parse_data_from_str(hex::encode(msg).as_str()).unwrap();
        println!("Received: {metadata:?} data: {x:?}");
    }
    sleep(time::Duration::from_millis(3000));

    send_data(
        client,
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

    let msg = client.receive().unwrap();
    if let (metadata, FrisquetData::Chaudiere(data)) =
        frisquet::parse_data_from_str(hex::encode(msg).as_str()).unwrap()
    {
        println!("Received: {metadata:?} data: {data:?}")
    }
    client.sleep().unwrap();
}

fn start_association(client: &mut dyn RFClient) {
    let network_id: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff];
    client.set_network_id(network_id).unwrap();
    println!("Setting network_id to ffffffff");

    println!("Waiting broadcast networkId message");
    let msg = client.receive().unwrap();
    if let (metadata, FrisquetData::Chaudiere(data)) =
        frisquet::parse_data_from_str(hex::encode(msg).as_str()).unwrap()
    {
        client.sleep().unwrap();
        println!("Received a message {data:?}");

        if let ChaudierePayload::ChaudiereAssociationBroadcast {
            unknown: _,
            network_id,
        } = data
        {
            println!("This is a ChaudiereAssociationBroadcast message, will announce");
            send_data(
                client,
                32,
                128,
                metadata.request_id,
                metadata.req_or_answer + 0x80,
                metadata.msg_type,
                SondePayload::SondeAssociationAnnounceMessage { data: vec![] },
            );
            let network_id_str = hex::encode(network_id).to_string();
            println!("Switch to networkId {network_id_str}");

            client.set_network_id(Vec::from(network_id)).unwrap();
            sleep(time::Duration::from_millis(200));
            send_data(
                client,
                32,
                128,
                33536,
                1,
                67,
                SondePayload::SondeInitMessage { data: vec![0, 0] },
            );
            loop {
                sleep(time::Duration::from_millis(1000));
                send_data(
                    client,
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

                let msg = client.receive().unwrap();

                if let (metadata, FrisquetData::Chaudiere(data)) =
                    frisquet::parse_data_from_str(hex::encode(msg).as_str()).unwrap()
                {
                    println!("Received: {metadata:?} data: {data:?}");
                    return;
                }
            }
        }
    }
}
