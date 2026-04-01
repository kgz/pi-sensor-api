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

## Build an APT package (.deb)

### From Windows (or any host): Docker → arm64 `.deb` for the Pi

Requires Docker (Docker Desktop on Windows). Builds inside `linux/arm64` so the package matches a 64-bit Pi OS.

```bash
make deb-docker
# optional: make deb-docker DEB_ARCH=amd64
```

Install cargo-deb on the host only if you build without Docker:

```bash
cargo install cargo-deb
make deb
```

Install locally with apt:

```bash
make deb-install
```

Package install behavior:

- installs binary to `/usr/bin/pi-sensor-api`
- installs systemd unit to `/lib/systemd/system/pi-sensor.service`
- creates `/etc/pi-sensor-api.env` from skeleton on first install
- reloads systemd and enables/starts `pi-sensor.service`

## GitHub Release asset install

Build and upload a `.deb` as a GitHub Release asset.

From Windows (or any machine with Docker), upload an **arm64** asset for Raspberry Pi:

```bash
make release-upload-docker VERSION=v0.1.0 REPO=<owner>/<repo> DEB_ARCH=arm64
```

On a Linux machine that matches the Pi’s architecture, you can use the native builder instead:

```bash
make release-upload VERSION=v0.1.0 REPO=<owner>/<repo>
```

This uploads:

- `pi-sensor-api_linux_arm64.deb` when using `release-upload-docker` with `DEB_ARCH=arm64`
- `pi-sensor-api_linux_amd64.deb` when using `deb-docker DEB_ARCH=amd64` + `release-upload-docker DEB_ARCH=amd64`, or a native amd64 `release-upload`

Install from release asset URL:

```bash
wget https://github.com/<owner>/<repo>/releases/download/v0.1.0/pi-sensor-api_linux_arm64.deb
sudo apt install ./pi-sensor-api_linux_arm64.deb
```

## Configuration

The app loads `.env` automatically when running from the repo, and reads:

- `GPIO_PIN` (default `4`)
- `PORT` (default `8777`)
- `SENSOR_TYPE` (`dht11` or `dht22`, default `dht11`)
- `TEMP_OFFSET_C` (default `0`)
- `HUMIDITY_OFFSET` (default `0`)

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
