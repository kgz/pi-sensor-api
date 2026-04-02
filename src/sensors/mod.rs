use anyhow::Result;

mod dht_bus;
mod dht11;
mod dht22;

pub use dht_bus::DhtBus;

#[derive(Clone, Copy, Debug)]
pub struct Reading {
    pub temperature_c: f32,
    pub humidity_percent: f32,
}

pub trait SensorModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn start_low_ms(&self) -> u64;
    fn bit_one_threshold_us(&self) -> u64;
    fn decode(&self, data: [u8; 5]) -> Result<Reading>;
}

pub fn module_from_env() -> Box<dyn SensorModule> {
    let kind = std::env::var("SENSOR_TYPE").unwrap_or_else(|_| "dht11".to_string());
    match kind.trim().to_ascii_lowercase().as_str() {
        "dht22" | "am2302" => Box::new(dht22::Dht22),
        _ => Box::new(dht11::Dht11),
    }
}

