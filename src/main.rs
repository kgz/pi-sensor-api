use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use rppal::gpio::{Gpio, InputPin, Level};
use serde::Serialize;
use std::thread;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const DEFAULT_GPIO_PIN: u8 = 4;
const DEFAULT_PORT: u16 = 8777;
const READ_TIMEOUT_MS: u64 = 250;
const MAX_RETRIES: u32 = 3;

#[derive(Clone)]
struct AppState {
    gpio_pin: u8,
}

#[derive(Serialize)]
struct SensorResponse {
    temperature: f32,
    humidity: f32,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

struct DHT11Reader {
    gpio: Gpio,
    gpio_pin: u8,
}

impl DHT11Reader {
    fn new(gpio_pin: u8) -> Result<Self> {
        let gpio = Gpio::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize GPIO (pin {}): {:?}", gpio_pin, e))?;
        Ok(DHT11Reader { gpio, gpio_pin })
    }

    fn read_sensor(&mut self) -> Result<(f32, f32)> {
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

        if !Self::wait_for_level(&pin, Level::Low, READ_TIMEOUT_MS)? {
            anyhow::bail!("Timeout waiting for DHT11 response");
        }

        if !Self::wait_for_level(&pin, Level::High, READ_TIMEOUT_MS)? {
            anyhow::bail!("Timeout waiting for DHT11 response");
        }

        if !Self::wait_for_level(&pin, Level::Low, READ_TIMEOUT_MS)? {
            anyhow::bail!("Timeout waiting for DHT11 response");
        }

        for byte in &mut data {
            for _bit in 0..8 {
                if !Self::wait_for_level(&pin, Level::High, READ_TIMEOUT_MS)? {
                    anyhow::bail!("Timeout reading bit");
                }

                let high_duration = Self::measure_high_duration(&pin)?;

                *byte <<= 1;
                if high_duration > 30 {
                    *byte |= 1;
                }

                if !Self::wait_for_level(&pin, Level::Low, READ_TIMEOUT_MS)? {
                    anyhow::bail!("Timeout reading bit");
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

        let humidity = data[0] as f32 + (data[1] as f32 / 10.0);
        let temperature = data[2] as f32 + (data[3] as f32 / 10.0);

        Ok((temperature, humidity))
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

    fn measure_high_duration(pin: &InputPin) -> Result<u64> {
        let start = Instant::now();
        let timeout = Duration::from_millis(READ_TIMEOUT_MS);

        while start.elapsed() < timeout {
            if pin.read() == Level::Low {
                return Ok(start.elapsed().as_micros() as u64);
            }
        }
        anyhow::bail!("Timeout measuring high duration");
    }
}

async fn read_sensor_with_retry(state: &AppState) -> Result<(f32, f32)> {
    let mut reader = DHT11Reader::new(state.gpio_pin)
        .map_err(|e| anyhow::anyhow!("Failed to initialize DHT11 reader on GPIO {}: {}", state.gpio_pin, e))?;

    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        match reader.read_sensor() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < MAX_RETRIES {
                    sleep(Duration::from_millis(100 * attempt as u64)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
}

async fn sensor_handler(State(state): State<AppState>) -> Result<Json<SensorResponse>, (StatusCode, Json<ErrorResponse>)> {
    match read_sensor_with_retry(&state).await {
        Ok((temperature, humidity)) => {
            Ok(Json(SensorResponse {
                temperature,
                humidity,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to read sensor: {}", e),
            }),
        )),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let gpio_pin = std::env::var("GPIO_PIN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_GPIO_PIN);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let state = AppState {
        gpio_pin,
    };

    let app = Router::new()
        .route("/sensor", get(sensor_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .with_context(|| format!("Failed to bind to port {}", port))?;

    println!("Sensor API listening on port {}", port);
    println!("GPIO pin: {}", gpio_pin);
    println!("Endpoint: http://0.0.0.0:{}/sensor", port);

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
