# Deploying Updated Sensor API to Raspberry Pi

## Quick setup on Pi (systemd, root for GPIO)

On the Pi host, the API runs as a **system** service (root) so it can access `/dev/gpiomem*`:

- Binary: `/usr/local/bin/pi-sensor-api`
- Service: `/etc/systemd/system/pi-sensor-api.service`
- `sudo systemctl status pi-sensor-api` — `sudo systemctl restart pi-sensor-api`

Docker is an option but needs more disk and had restart-loop issues on this Pi; the system service is the working setup.

---

The rate limit has been removed from the sensor API. To deploy an updated version:

## 1. Build for Raspberry Pi (aarch64)

Using Docker for cross-compilation:

```bash
cd pi-sensor-api
docker run --rm -v $(pwd):/work -w /work rust:latest cargo build --release --target aarch64-unknown-linux-gnu
```

Or if you have cross-compilation set up locally:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

## 2. Copy to Raspberry Pi

```bash
scp target/aarch64-unknown-linux-gnu/release/pi-sensor-api <pi-user>@<pi-host>:/tmp/pi-sensor-api
```

## 3. Restart the Service on Pi

SSH into the Pi and restart the service:

```bash
ssh <pi-user>@<pi-host>
sudo systemctl restart pi-sensor.service
# Or if running manually:
# pkill pi-sensor-api
# /usr/local/bin/pi-sensor-api
```

## 4. Verify

Check the service is running:

```bash
curl http://<pi-host>:8777/sensor
```

You should be able to call it multiple times quickly without getting 429 errors.
