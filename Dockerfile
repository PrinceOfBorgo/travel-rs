FROM rust:slim AS builder

WORKDIR /usr/src/app

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

# Build the application
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

# Install SSL runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy only the executable
COPY --from=builder /usr/src/app/target/release/travel-rs /app/travel-rs

# Create directories for volumes
RUN mkdir -p /app/config
RUN mkdir -p /app/locales
RUN mkdir -p /app/logs

VOLUME ["/app/config", "/app/locales", "/app/logs"]

ENV RUST_LOG=info

CMD ["./travel-rs"]
