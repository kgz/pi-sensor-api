# Raspberry Pi DHT11 Sensor API

Rust HTTP API for reading temperature and humidity from a DHT11 sensor on a Raspberry Pi.

## Clone and install

```bash
git clone <your-repo-url>
cd pi-sensor-api
make install
```

`make install` does all of the following:

- creates `.env` from `.env.skeleton` if `.env` is missing
- builds `target/release/pi-sensor-api`
- installs binary to `/usr/local/bin/pi-sensor-api`
- installs env file to `/etc/pi-sensor-api.env`
- renders and installs systemd unit from `pi-sensor.service.template`
- runs `systemctl daemon-reload` and enables/starts `pi-sensor.service`

## Configuration

The app loads `.env` automatically when running from the repo, and reads:

- `GPIO_PIN` (default `4`)
- `PORT` (default `8777`)

Initial config:

```bash
cp .env.skeleton .env
```

Then edit `.env` values for your hardware.

## Local run (without systemd)

```bash
make env
cargo run --release
```

Test endpoint:

```bash
curl http://localhost:8777/sensor
```

## Docker Compose

Bring up the service:

```bash
make docker-up
```

Stop it:

```bash
make docker-down
```

## Service commands

```bash
make status
make logs
make restart
```

## Pin wiring

See `PIN_CONNECTIONS.md`.
