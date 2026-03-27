# syntax=docker/dockerfile:1
# Build stage
FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build the real binary
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/pi-sensor-api /usr/local/bin/pi-sensor-api
EXPOSE 8777
ENTRYPOINT ["/usr/local/bin/pi-sensor-api"]
