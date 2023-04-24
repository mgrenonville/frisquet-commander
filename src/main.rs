mod messages;
mod frisquet;

use std::{
    env,
    process,
    time::Duration,
};


extern crate paho_mqtt as mqtt;

use serde_json::Result;


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
    let rx = cli.start_consuming();
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

    loop {
        for msg in rx.iter() {
            if let Some(msg) = msg {
                let data: messages::DataMessage = serde_json::from_str(msg.payload_str().as_ref()).unwrap();
                let parsed =  frisquet::parse_data_from_str(data.data.as_str()).unwrap();
                println!("received: {parsed:?}");
            }
        }
    }
}
