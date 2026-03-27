# pi-sensor-api Service Setup (Auto-Start)

The **pi-sensor-api is not currently configured as a systemd service** on the sensor host (e.g. `mat@192.168.0.39`). It must be started manually. This guide explains how to set it up to start automatically on boot.

## Prerequisites

- The `pi-sensor-api` binary is on the host (e.g. at `/home/mat/pi-sensor-api`)
- You have `sudo` on the host to install and enable a systemd service

## 1. Ensure `pi-sensor.service` Matches Your Deployment

The `pi-sensor.service` file in this directory is set up for:

- **User/Group:** `mat`
- **Binary:** `/home/mat/pi-sensor-api`
- **WorkingDirectory:** `/home/mat`
- **Environment:** `GPIO_PIN=4`, `PORT=8777`

If your binary or user is different, edit `pi-sensor.service` and change:

- `ExecStart` – full path to the `pi-sensor-api` binary  
- `WorkingDirectory` – directory to run from (e.g. home dir)  
- `User` and `Group` – user that should run the service  

## 2. Install and Enable the Service

On the sensor host:

```bash
# Copy the service file (adjust path if your repo is elsewhere)
sudo cp pi-sensor.service /etc/systemd/system/

# Reload systemd so it sees the new unit
sudo systemctl daemon-reload

# Enable auto-start on boot
sudo systemctl enable pi-sensor.service

# Start the service now (no need to reboot)
sudo systemctl start pi-sensor.service
```

## 3. Verify

```bash
# Check status
sudo systemctl status pi-sensor.service

# Test the API (replace host if different)
curl http://192.168.0.39:8777/sensor
```

## Useful Commands

| Action        | Command                                  |
|---------------|-------------------------------------------|
| Start         | `sudo systemctl start pi-sensor.service`  |
| Stop          | `sudo systemctl stop pi-sensor.service`   |
| Restart       | `sudo systemctl restart pi-sensor.service`|
| View logs     | `sudo journalctl -u pi-sensor.service -f` |
| Disable       | `sudo systemctl disable pi-sensor.service`|

## After Updating the Binary

If you deploy a new `pi-sensor-api` binary (e.g. via `scp` as in `DEPLOY.md`), restart the service:

```bash
sudo systemctl restart pi-sensor.service
```

## Notes

- The service is configured with `Restart=on-failure` and `RestartSec=5`, so it will be restarted if it exits with an error.
- `WantedBy=multi-user.target` means it starts when the system reaches the normal multi-user (non-graphical) runlevel, which includes after reboot.
