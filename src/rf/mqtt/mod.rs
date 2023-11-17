extern crate paho_mqtt as mqtt;

use std::collections::HashMap;
use std::result::Result;
use std::{env, process, time::Duration};

use hex;
use mqtt::{Message, Receiver};
// use serde_json::Result;

use crate::rf::mqtt::messages::{CommandMessage, Listen, SendData, SetNetworkId, Sleep};
use crate::rf::RFClient;
pub mod messages;

pub struct MqttClient {
    client: mqtt::Client,
    rx: Receiver<Option<Message>>,
}

pub fn new(settings: &HashMap<String, String>) -> Result<MqttClient, String> {
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
    let client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });
    let rx: Receiver<Option<Message>> = client.start_consuming();
    // Define the set of options for the connection.
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = client.connect(conn_opts) {
        return Err(format!("Unable to connect:\n\t{:?}", e));
    }

    if let Err(e) = client.subscribe(settings.get("mqtt_frisquet_topic").unwrap(), 0) {
        return Err(format!("Error subscribes topics: {:?}", e));
    }

    Ok(MqttClient {
        client: client,
        rx: rx,
    })
}

impl MqttClient {
    fn publish(&self, value: &dyn CommandMessage) -> Result<(), String> {
        let json = serde_json::to_vec(value).unwrap();

        if let Err(e) = self
            .client
            .publish(Message::new("frisquet/command", json, 0))
        {
            return Err(format!("Error subscribes topics: {:?}", e));
        }
        Ok(())
    }

    fn await_message(&self) -> String {
        loop {
            for msg in self.rx.iter() {
                if let Some(msg) = msg {
                    let data: messages::DataMessage =
                        serde_json::from_str(msg.payload_str().as_ref()).unwrap();
                    return data.data;
                }
            }
        }
    }
}

impl RFClient for MqttClient {
    fn set_network_id(&mut self, network_id: Vec<u8>) -> Result<(), String> {
        self.publish(&SetNetworkId {
            network_id: hex::encode(network_id),
        })
    }

    fn receive(&mut self) -> Result<Vec<u8>, String> {
        self.publish(&Listen {})?;
        hex::decode(self.await_message()).map_err(|e| e.to_string())
    }

    fn send(&mut self, payload: Vec<u8>) -> Result<(), String> {
        self.publish(&SendData {
            payload: hex::encode(payload),
        })
    }

    fn sleep(&mut self) -> Result<(), String> {
        self.publish(&Sleep {})
    }
}
