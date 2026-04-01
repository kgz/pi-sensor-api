use anyhow::{Context, Result};
use rppal::gpio::{Gpio, InputPin, Level};
use std::convert::TryFrom;
use std::thread;
use std::time::{Duration, Instant};

const READ_TIMEOUT_MS: u64 = 250;
const BIT_ONE_THRESHOLD_US: u64 = 50;

pub struct DhtBus {
    gpio: Gpio,
    gpio_pin: u8,
}

impl DhtBus {
    pub fn new(gpio_pin: u8) -> Result<Self> {
        let gpio = Gpio::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize GPIO (pin {}): {:?}", gpio_pin, e))?;
        Ok(Self { gpio, gpio_pin })
    }

    pub fn read_frame(&mut self) -> Result<[u8; 5]> {
        let mut data = [0u8; 5];

        let mut pin = self
            .gpio
            .get(self.gpio_pin)
            .with_context(|| format!("Failed to get GPIO pin {}", self.gpio_pin))?
            .into_output();

        pin.set_low();
        thread::sleep(Duration::from_millis(18));
        pin.set_high();
        thread::sleep(Duration::from_micros(30));
        drop(pin);

        let pin = self
            .gpio
            .get(self.gpio_pin)
            .with_context(|| format!("Failed to get GPIO pin {}", self.gpio_pin))?
            .into_input();

        if !wait_for_level(&pin, Level::Low, READ_TIMEOUT_MS)? {
            anyhow::bail!("Timeout waiting for sensor response (LOW)");
        }
        if !wait_for_level(&pin, Level::High, READ_TIMEOUT_MS)? {
            anyhow::bail!("Timeout waiting for sensor response (HIGH)");
        }
        if !wait_for_level(&pin, Level::Low, READ_TIMEOUT_MS)? {
            anyhow::bail!("Timeout waiting for sensor response (LOW 2)");
        }

        for byte in &mut data {
            for _bit in 0..8 {
                if !wait_for_level(&pin, Level::High, READ_TIMEOUT_MS)? {
                    anyhow::bail!("Timeout reading bit (HIGH)");
                }

                let high_duration_us = measure_high_duration_us(&pin)?;

                *byte <<= 1;
                if high_duration_us >= BIT_ONE_THRESHOLD_US {
                    *byte |= 1;
                }

                if !wait_for_level(&pin, Level::Low, READ_TIMEOUT_MS)? {
                    anyhow::bail!("Timeout reading bit (LOW)");
                }
            }
        }

        let checksum = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);
        if checksum != data[4] {
            anyhow::bail!("Checksum mismatch");
        }

        Ok(data)
    }
}

fn wait_for_level(pin: &InputPin, level: Level, timeout_ms: u64) -> Result<bool> {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if pin.read() == level {
            return Ok(true);
        }
    }
    Ok(false)
}

fn measure_high_duration_us(pin: &InputPin) -> Result<u64> {
    let start = Instant::now();
    let timeout = Duration::from_millis(READ_TIMEOUT_MS);

    while start.elapsed() < timeout {
        if pin.read() == Level::Low {
            let micros = start.elapsed().as_micros();
            return Ok(u64::try_from(micros).unwrap_or(u64::MAX));
        }
    }

    anyhow::bail!("Timeout measuring high duration");
}

