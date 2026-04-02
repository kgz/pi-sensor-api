use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

mod sensors;

const DEFAULT_GPIO_PIN: u8 = 4;
const DEFAULT_PORT: u16 = 8777;
const MAX_RETRIES: u32 = 3;

#[derive(Clone)]
struct AppState {
    gpio_pin: u8,
    temp_offset_c: f32,
    humidity_offset: f32,
    sensor: std::sync::Arc<dyn sensors::SensorModule>,
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

async fn read_sensor_with_retry(state: &AppState) -> Result<sensors::Reading> {
    let mut bus = sensors::DhtBus::new(state.gpio_pin).with_context(|| {
        format!(
            "Failed to initialize sensor bus (type={}, GPIO={})",
            state.sensor.name(),
            state.gpio_pin
        )
    })?;

    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        match bus
            .read_frame(state.sensor.start_low_ms(), state.sensor.bit_one_threshold_us())
            .and_then(|frame| state.sensor.decode(frame))
        {
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
        Ok(reading) => {
            let temperature = reading.temperature_c + state.temp_offset_c;
            let humidity = reading.humidity_percent + state.humidity_offset;
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

    let temp_offset_c = std::env::var("TEMP_OFFSET_C")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let humidity_offset = std::env::var("HUMIDITY_OFFSET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let sensor: std::sync::Arc<dyn sensors::SensorModule> = sensors::module_from_env().into();
    let sensor_name = sensor.name();

    let state = AppState {
        gpio_pin,
        temp_offset_c,
        humidity_offset,
        sensor,
    };

    let app = Router::new()
        .route("/sensor", get(sensor_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .with_context(|| format!("Failed to bind to port {}", port))?;

    println!("Sensor API listening on port {}", port);
    println!("GPIO pin: {}", gpio_pin);
    println!("Sensor type: {}", sensor_name);
    println!("Endpoint: http://0.0.0.0:{}/sensor", port);

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
