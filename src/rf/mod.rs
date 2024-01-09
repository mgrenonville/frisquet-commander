pub mod mqtt;
pub mod serial;

pub trait RFClient {
    fn set_network_id(&mut self, network_id: Vec<u8>) -> Result<(), String>;
    fn receive(&mut self) -> Result<Vec<u8>, String>;
    fn send(&mut self, payload: Vec<u8>) -> Result<(), String>;
    fn sleep(&mut self) -> Result<(), String>;
}
