extern crate serialport;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::result::Result;
use std::time::Duration;

use hex;

use crate::rf::RFClient;

pub struct SerialClient {
    port: Box<dyn serialport::SerialPort>,
    buffer: Vec<u8>,
    data_packets: VecDeque<Vec<u8>>,
}

pub fn new(settings: &HashMap<String, String>) -> Result<SerialClient, String> {
    let port = serialport::new(
        settings.get("serial_port").unwrap(),
        settings.get("serial_speed").unwrap().parse().unwrap(),
    )
    .timeout(Duration::from_millis(5000))
    .open()
    .expect("Failed to open port");

    Ok(SerialClient {
        port: port,
        buffer: vec![],
        data_packets: VecDeque::new(),
    })
}

impl RFClient for SerialClient {
    fn set_network_id(&mut self, network_id: Vec<u8>) -> Result<(), String> {
        let cmd = format!("NID: {}", hex::encode(network_id));
        self.port
            .write_all(cmd.as_bytes())
            .map(|_x| ())
            .map_err(|e| e.to_string())?;
        self.port.flush().map_err(|e| e.to_string())
    }

    fn receive(&mut self) -> Result<Vec<u8>, String> {
        if let Some(data) = self.data_packets.pop_front() {
            return Ok(data);
        }

        let cmd = format!("LST:");
        self.port
            .write_all(cmd.as_bytes())
            .map(|_x| ())
            .map_err(|e| e.to_string())?;
        self.port.flush().map_err(|e| e.to_string())?;

        loop {
            let mut buf = [0; 512];
            let read = match self.port.read(&mut buf) {
                Ok(v) => Ok(v),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::TimedOut => Ok(0 as usize),
                    error => Err(error.to_string()),
                },
            }?;

            for n in 0..read {
                if buf[n] == 0xd {
                    // \r
                    continue;
                }
                if buf[n] == 0xA {
                    // \r
                    let data = hex::decode(&self.buffer)
                        .map_err(|e| {
                            // println!("{}: {}", e, String::from_utf8(self.buffer.clone()).unwrap())
                        })
                        .unwrap_or(vec![]);

                    if !data.is_empty() {
                        self.data_packets.push_back(data)
                    }
                    self.buffer.clear();
                } else {
                    self.buffer.push(buf[n]);
                }
            }

            if let Some(data) = self.data_packets.pop_front() {
                return Ok(data);
            }
        }
    }

    fn send(&mut self, payload: Vec<u8>) -> Result<(), String> {
        let cmd = format!("CMD: {}", hex::encode(payload));
        self.port
            .write_all(cmd.as_bytes())
            .map(|_x| ())
            .map_err(|e| e.to_string())?;
        self.port.flush().map_err(|e| e.to_string())
    }

    fn sleep(&mut self) -> Result<(), String> {
        let cmd = format!("SLP:");
        self.port
            .write_all(cmd.as_bytes())
            .map(|_x| ())
            .map_err(|e| e.to_string())?;
        self.port.flush().map_err(|e| e.to_string())
    }
}
