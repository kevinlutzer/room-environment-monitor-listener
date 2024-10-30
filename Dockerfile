FROM rust:1.82.0-bullseye

# Setup workspace
WORKDIR /usr/src/room-environment-monitor-listener
COPY . .

# Setup build dependencies
RUN apt-get update -y
RUN apt-get install -y cmake build-essential pkg-config libssl-dev

# Build and install
RUN cargo install --path .

# Ports
EXPOSE 8080
EXPOSE 1883

# Env Config that isn't parameterized
ENV MQTT_PORT=1883
ENV RUST_LOG=info
ENV HTTP_HOST=0.0.0.0
ENV HTTP_PORT=8080

CMD ["room-environment-monitor-listener"]