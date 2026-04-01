SHELL := /bin/sh

# For Raspberry Pi: build .deb inside Docker (matches arch). Example: make deb-docker DEB_ARCH=arm64
DEB_ARCH ?= arm64
DEB_IMAGE ?= rust:bookworm

SERVICE_NAME := pi-sensor.service
SERVICE_TEMPLATE := pi-sensor.service.template
SERVICE_PATH := /etc/systemd/system/$(SERVICE_NAME)
ENV_TARGET := /etc/pi-sensor-api.env
BIN_TARGET := /usr/local/bin/pi-sensor-api
WORKDIR := /opt/pi-sensor-api
INSTALL_USER ?= $(shell id -un)
INSTALL_GROUP ?= $(shell id -gn)

.PHONY: help env build install install-bin install-env install-service enable-service restart status logs docker-up docker-down deb deb-install deb-asset deb-docker deb-asset-docker release-upload release-upload-docker

help:
	@printf "%s\n" \
	"make env           # create .env from .env.skeleton if missing" \
	"make build         # build release binary" \
	"make install       # build + install binary + install systemd service" \
	"make deb           # build .deb on this machine (requires cargo-deb)" \
	"make deb-docker    # build .deb in Docker for DEB_ARCH (default arm64; good from Windows)" \
	"make deb-install   # build and install .deb with apt" \
	"make deb-asset     # create predictable release asset filename" \
	"make deb-asset-docker  # deb-docker + copy to target/release-assets/pi-sensor-api_linux_\$\$DEB_ARCH.deb" \
	"make release-upload VERSION=vX.Y.Z REPO=owner/repo  # upload .deb (native arch)" \
	"make release-upload-docker VERSION=vX.Y.Z REPO=owner/repo DEB_ARCH=arm64  # upload Pi .deb from any host" \
	"make restart       # restart systemd service" \
	"make status        # service status" \
	"make logs          # follow service logs" \
	"make docker-up     # docker compose up -d" \
	"make docker-down   # docker compose down"

env:
	@if [ ! -f ".env" ]; then cp .env.skeleton .env; fi

build:
	cargo build --release

install: env build install-bin install-env install-service enable-service

install-bin:
	sudo install -m 0755 target/release/pi-sensor-api $(BIN_TARGET)

install-env:
	sudo mkdir -p $(WORKDIR)
	sudo install -m 0644 .env $(ENV_TARGET)

install-service:
	sed \
		-e "s|{{USER}}|$(INSTALL_USER)|g" \
		-e "s|{{GROUP}}|$(INSTALL_GROUP)|g" \
		-e "s|{{WORKDIR}}|$(WORKDIR)|g" \
		-e "s|{{ENV_FILE}}|$(ENV_TARGET)|g" \
		$(SERVICE_TEMPLATE) > /tmp/$(SERVICE_NAME)
	sudo install -m 0644 /tmp/$(SERVICE_NAME) $(SERVICE_PATH)
	rm -f /tmp/$(SERVICE_NAME)

enable-service:
	sudo systemctl daemon-reload
	sudo systemctl enable --now $(SERVICE_NAME)

restart:
	sudo systemctl restart $(SERVICE_NAME)

status:
	sudo systemctl status $(SERVICE_NAME)

logs:
	sudo journalctl -u $(SERVICE_NAME) -f

docker-up: env
	docker compose up -d --build

docker-down:
	docker compose down

deb:
	cargo deb

deb-install: deb
	sudo apt install -y ./target/debian/pi-sensor-api_*.deb

deb-asset: deb
	mkdir -p target/release-assets
	cp target/debian/pi-sensor-api_*.deb target/release-assets/pi-sensor-api_linux_$$(dpkg --print-architecture).deb

deb-docker:
	docker run --rm --platform linux/$(DEB_ARCH) \
		-v "$(CURDIR):/work" -w /work \
		$(DEB_IMAGE) \
		bash -lc 'set -eu; \
		CARGO_BIN="$$(command -v cargo 2>/dev/null || true)"; \
		if [ -z "$$CARGO_BIN" ] && [ -x /usr/local/cargo/bin/cargo ]; then CARGO_BIN=/usr/local/cargo/bin/cargo; fi; \
		if [ -z "$$CARGO_BIN" ]; then echo "cargo not found in container PATH"; exit 127; fi; \
		apt-get update; \
		apt-get install -y --no-install-recommends pkg-config ca-certificates; \
		"$$CARGO_BIN" install cargo-deb; \
		"$$CARGO_BIN" deb'

deb-asset-docker: deb-docker
	mkdir -p target/release-assets
	cp target/debian/pi-sensor-api_*_$(DEB_ARCH).deb target/release-assets/pi-sensor-api_linux_$(DEB_ARCH).deb

release-upload: deb-asset
	@if [ -z "$(VERSION)" ] || [ -z "$(REPO)" ]; then echo "Usage: make release-upload VERSION=vX.Y.Z REPO=owner/repo"; exit 1; fi
	@command -v gh >/dev/null 2>&1 || { echo "Install GitHub CLI: sudo apt install gh   or   https://cli.github.com"; exit 127; }
	@test -x "$$(command -v gh)" || { echo "gh is not executable: $$(command -v gh) — try: chmod +x $$(command -v gh)"; exit 126; }
	gh release view "$(VERSION)" --repo "$(REPO)" >/dev/null 2>&1 || gh release create "$(VERSION)" --repo "$(REPO)" --title "$(VERSION)" --notes "Release $(VERSION)"
	gh release upload "$(VERSION)" target/release-assets/pi-sensor-api_linux_$$(dpkg --print-architecture).deb --repo "$(REPO)" --clobber

release-upload-docker: deb-asset-docker
	@if [ -z "$(VERSION)" ] || [ -z "$(REPO)" ]; then echo "Usage: make release-upload-docker VERSION=vX.Y.Z REPO=owner/repo DEB_ARCH=arm64"; exit 1; fi
	@command -v gh >/dev/null 2>&1 || { echo "Install GitHub CLI: sudo apt install gh   or   https://cli.github.com"; exit 127; }
	@test -x "$$(command -v gh)" || { echo "gh is not executable: $$(command -v gh) — try: chmod +x $$(command -v gh)"; exit 126; }
	gh release view "$(VERSION)" --repo "$(REPO)" >/dev/null 2>&1 || gh release create "$(VERSION)" --repo "$(REPO)" --title "$(VERSION)" --notes "Release $(VERSION)"
	gh release upload "$(VERSION)" target/release-assets/pi-sensor-api_linux_$(DEB_ARCH).deb --repo "$(REPO)" --clobber
