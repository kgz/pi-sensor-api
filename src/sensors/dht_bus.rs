use anyhow::{Context, Result};
use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};
use nix::poll::{poll, PollFd, PollFlags};
use std::time::{Duration, Instant};

const READ_TIMEOUT_MS: u64 = 250;
const DEFAULT_GPIO_CHIP: &str = "/dev/gpiochip0";

pub struct DhtBus {
    gpio_pin: u8,
    gpio_chip: String,
}

impl DhtBus {
    pub fn new(gpio_pin: u8) -> Result<Self> {
        let gpio_chip = std::env::var("GPIO_CHIP").unwrap_or_else(|_| DEFAULT_GPIO_CHIP.to_string());
        Ok(Self { gpio_pin, gpio_chip })
    }

    pub fn read_frame(&mut self, start_low_ms: u64, bit_one_threshold_us: u64) -> Result<[u8; 5]> {
        let mut chip = Chip::new(&self.gpio_chip)
            .with_context(|| format!("Failed to open GPIO chip {}", self.gpio_chip))?;

        {
            let line = chip
                .get_line(u32::from(self.gpio_pin))
                .with_context(|| format!("Failed to get GPIO line {}", self.gpio_pin))?;

            let handle = line
                .request(LineRequestFlags::OUTPUT, 1, "pi-sensor-api")
                .context("Failed to request GPIO as output")?;

            handle.set_value(0).context("Failed to drive GPIO low")?;
            std::thread::sleep(Duration::from_millis(start_low_ms));
        }
        // Release the line and let the pull-up resistor bring it HIGH.
        std::thread::sleep(Duration::from_micros(30));

        let line = chip
            .get_line(u32::from(self.gpio_pin))
            .with_context(|| format!("Failed to get GPIO line {}", self.gpio_pin))?;

        let mut events = line
            .events(
                LineRequestFlags::INPUT,
                EventRequestFlags::BOTH_EDGES,
                "pi-sensor-api",
            )
            .context("Failed to request GPIO events")?;

        let mut highs_us: Vec<u64> = Vec::with_capacity(41);

        // We measure HIGH pulse width per bit: rising edge -> falling edge.
        // First we need to sync past the sensor's response pulses.
        let deadline = Instant::now() + Duration::from_millis(READ_TIMEOUT_MS);
        let mut last_rise_ns: Option<u64> = None;

        while highs_us.len() < 41 {
            let now = Instant::now();
            if now >= deadline {
                anyhow::bail!("Timeout waiting for sensor edge events");
            }

            let remaining = deadline.saturating_duration_since(now);
            let timeout_ms_i32 = i32::try_from(remaining.as_millis()).unwrap_or(i32::MAX);
            let mut fds = [PollFd::new(events.file(), PollFlags::POLLIN)];
            let ready = poll(&mut fds, timeout_ms_i32).context("poll() failed")?;
            if ready == 0 {
                continue;
            }

            let evt = events.get_event().context("Failed to read GPIO event")?;

            match evt.event_type() {
                EventType::RisingEdge => {
                    last_rise_ns = Some(evt.timestamp());
                }
                EventType::FallingEdge => {
                    if let Some(rise_ns) = last_rise_ns {
                        let fall_ns = evt.timestamp();
                        if fall_ns > rise_ns {
                            let high_us = (fall_ns - rise_ns) / 1_000;
                            if high_us >= 10 && high_us <= 120 {
                                highs_us.push(high_us);
                            }
                        }
                    }
                    last_rise_ns = None;
                }
            }
        }

        // First HIGH pulse is the sensor's 80us response HIGH; the remaining 40 are data bits.
        let data_highs = highs_us
            .get(1..)
            .ok_or_else(|| anyhow::anyhow!("Missing data pulses"))?;
        if data_highs.len() != 40 {
            anyhow::bail!("Expected 40 data pulses, got {}", data_highs.len());
        }

        let mut data = [0u8; 5];
        for i in 0..40 {
            let byte_idx = i / 8;
            data[byte_idx] <<= 1;
            data[byte_idx] |= if data_highs[i] >= bit_one_threshold_us { 1 } else { 0 };
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
