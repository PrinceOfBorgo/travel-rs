# Stage 1: Prepare dependencies using cargo-chef
FROM rust:slim AS chef
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build the application with sccache and cache mounts
FROM rust:slim AS builder
WORKDIR /app

# Copy the generated recipe for dependency building
COPY --from=chef /app/recipe.json recipe.json

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install sccache for compiler caching
RUN cargo install sccache

# Set up sccache as the Rust compiler wrapper
ENV RUSTC_WRAPPER="sccache"

# Build dependencies, leveraging BuildKit's cache mounts for speed
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --manifest-path recipe.json

# Copy the rest of the application source code
COPY . .

# Build the final application, leveraging cached dependencies and compiler cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release

# Stage 3: Create the final, minimal runtime image
FROM debian:bookworm-slim
WORKDIR /app

# Install only necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled executable from the builder stage
COPY --from=builder /app/target/release/travel-rs /app/travel-rs

# Create directories for persistent volumes
RUN mkdir -p /app/config
RUN mkdir -p /app/locales
RUN mkdir -p /app/logs

# Declare volumes for external data
VOLUME ["/app/config", "/app/locales", "/app/logs"]

# Set environment variables
ENV RUST_LOG=info

# Define the default command to run the application
CMD ["./travel-rs"]
