use super::{Reading, SensorModule};
use anyhow::Result;

pub struct Dht22;

impl SensorModule for Dht22 {
    fn name(&self) -> &'static str {
        "dht22"
    }

    fn decode(&self, data: [u8; 5]) -> Result<Reading> {
        let humidity_raw = (u16::from(data[0]) << 8) | u16::from(data[1]);
        let humidity_percent = f32::from(humidity_raw) / 10.0;

        let temp_high = data[2];
        let temp_low = data[3];
        let negative = (temp_high & 0x80) != 0;
        let temp_raw = (u16::from(temp_high & 0x7F) << 8) | u16::from(temp_low);
        let mut temperature_c = f32::from(temp_raw) / 10.0;
        if negative {
            temperature_c = -temperature_c;
        }

        Ok(Reading {
            temperature_c,
            humidity_percent,
        })
    }
}

