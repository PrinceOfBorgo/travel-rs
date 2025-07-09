# Stage 1: Prepare dependencies using cargo-chef
FROM rust:slim AS chef
WORKDIR /app
# Install cargo-chef here. This will be compiled once per architecture.
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build the application with cache mounts
FROM rust:slim AS builder
WORKDIR /app

# Install build dependencies and tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy cargo-chef binary from the 'chef' stage.
# This avoids recompiling it in the 'builder' stage for each platform.
# cargo install places binaries in /usr/local/cargo/bin by default.
COPY --from=chef /usr/local/cargo/bin/cargo-chef /usr/local/bin/cargo-chef

# Copy only manifests and recipe for dependency resolution
COPY --from=chef /app/recipe.json recipe.json
COPY Cargo.toml Cargo.lock ./
# Ensure all parts of travel-rs-derive are copied for cargo chef cook and subsequent cargo build
COPY travel-rs-derive/Cargo.toml travel-rs-derive/Cargo.toml
COPY travel-rs-derive/src/ travel-rs-derive/src/

# Build dependencies, leveraging BuildKit's cache mounts for speed.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Copy the rest of the source code
COPY . .

# Build the final application, leveraging cached dependencies and compiler cache.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    mv /app/target/release/travel-rs /app/travel-rs

# Stage 3: Create the final, minimal runtime image
FROM debian:bookworm-slim
WORKDIR /app

# Install only necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled executable from the builder stage from its new direct location
COPY --from=builder /app/travel-rs /app/travel-rs

# Create directories for persistent volumes
RUN mkdir -p /app/config /app/locales /app/logs

# Declare volumes for external data
VOLUME ["/app/config", "/app/locales", "/app/logs"]

# Set environment variables
ENV RUST_LOG=info

# Define the default command to run the application
CMD ["./travel-rs"]
