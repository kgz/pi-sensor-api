use super::{Reading, SensorModule};
use anyhow::Result;

pub struct Dht11;

impl SensorModule for Dht11 {
    fn name(&self) -> &'static str {
        "dht11"
    }

    fn start_low_ms(&self) -> u64 {
        18
    }

    fn bit_one_threshold_us(&self) -> u64 {
        40
    }

    fn decode(&self, data: [u8; 5]) -> Result<Reading> {
        let humidity_percent = f32::from(data[0]) + (f32::from(data[1]) / 10.0);
        let temperature_c = f32::from(data[2]) + (f32::from(data[3]) / 10.0);

        Ok(Reading {
            temperature_c,
            humidity_percent,
        })
    }
}

