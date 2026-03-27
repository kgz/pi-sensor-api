# Move Docker storage to new HD

Run these on the Pi after SSH: `ssh <pi-user>@<pi-host>`. Replace `/dev/sda1` with your new disk partition if different.

## 1. Find the new disk

```bash
lsblk
# Note the device for the new HD (e.g. /dev/sda1)
```

## 2. Format and mount (if not already done)

```bash
sudo mkfs.ext4 -L docker-data /dev/sda1
sudo mkdir -p /mnt/docker-data
sudo mount /dev/sda1 /mnt/docker-data
```

## 3. Add to fstab so it mounts on boot

```bash
sudo blkid /dev/sda1
# Copy the UUID from output, then:
echo 'UUID=YOUR_UUID_HERE /mnt/docker-data ext4 defaults,nofail 0 2' | sudo tee -a /etc/fstab
```

## 4. Stop Docker and move data

```bash
sudo systemctl stop docker
sudo systemctl stop docker.socket

sudo rsync -aP /var/lib/docker/ /mnt/docker-data/
```

## 5. Point Docker at new data root

```bash
sudo mkdir -p /etc/docker
echo '{"data-root": "/mnt/docker-data"}' | sudo tee /etc/docker/daemon.json
```

## 6. Start Docker and check

```bash
sudo systemctl start docker
docker info | grep "Docker Root Dir"
# Should show: Docker Root Dir: /mnt/docker-data
docker images
```

## 7. Remove old data from SD after confirming

```bash
sudo rm -rf /var/lib/docker
```

---

## Why the SD card still fills: containerd

Docker uses **containerd** to store image layers and container filesystems. Setting `data-root` only moves Docker’s own data; **containerd still uses `/var/lib/containerd` on the SD** by default, so pulls and new containers keep filling the card.

Move containerd to the HD as well:

```bash
sudo systemctl stop docker
sudo systemctl stop containerd

sudo mkdir -p /mnt/docker-data/containerd
sudo rsync -aP /var/lib/containerd/ /mnt/docker-data/containerd/

# Set containerd root (uncomment and set in config)
sudo sed -i 's|#root = \"/var/lib/containerd\"|root = \"/mnt/docker-data/containerd\"|' /etc/containerd/config.toml

sudo systemctl start containerd
sudo systemctl start docker
```

After confirming everything works, free SD space by clearing the old containerd data (containerd now uses the HD):

```bash
sudo rm -rf /var/lib/containerd/*
```

Leave the directory `/var/lib/containerd` in place; some setups expect it to exist.
