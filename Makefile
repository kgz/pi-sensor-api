SHELL := /bin/sh

SERVICE_NAME := pi-sensor.service
SERVICE_TEMPLATE := pi-sensor.service.template
SERVICE_PATH := /etc/systemd/system/$(SERVICE_NAME)
ENV_TARGET := /etc/pi-sensor-api.env
BIN_TARGET := /usr/local/bin/pi-sensor-api
WORKDIR := /opt/pi-sensor-api
INSTALL_USER ?= $(shell id -un)
INSTALL_GROUP ?= $(shell id -gn)

.PHONY: help env build install install-bin install-env install-service enable-service restart status logs docker-up docker-down deb deb-install

help:
	@printf "%s\n" \
	"make env           # create .env from .env.skeleton if missing" \
	"make build         # build release binary" \
	"make install       # build + install binary + install systemd service" \
	"make deb           # build .deb package (requires cargo-deb)" \
	"make deb-install   # build and install .deb with apt" \
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
