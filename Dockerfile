# Stage 1: Prepare dependencies using cargo-chef
FROM rust:slim AS chef
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build the application with sccache and cache mounts
FROM rust:slim AS builder
WORKDIR /app

# Install build dependencies and tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install sccache \
    && cargo install cargo-chef

# Set up sccache as the Rust compiler wrapper
# ENV RUSTC_WRAPPER="sccache" # Keeping this commented out as we're overriding it below

# Copy only manifests and recipe for dependency resolution
COPY --from=chef /app/recipe.json recipe.json
COPY Cargo.toml Cargo.lock ./
# Ensure all parts of travel-rs-derive are copied for cargo chef cook and subsequent cargo build
COPY travel-rs-derive/Cargo.toml travel-rs-derive/Cargo.toml
COPY travel-rs-derive/src/ travel-rs-derive/src/

# Build dependencies, leveraging BuildKit's cache mounts for speed.
# Explicitly unset RUSTC_WRAPPER for this command to bypass sccache.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    RUSTC_WRAPPER="" cargo chef cook --release --recipe-path recipe.json

# Copy the rest of the source code
COPY . .

# Build the final application, leveraging cached dependencies and compiler cache.
# Explicitly unset RUSTC_WRAPPER for this command to bypass sccache.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    RUSTC_WRAPPER="" cargo build --release && \
    mv /app/target/release/travel-rs /app/travel-rs

# The debugging commands are left for now but will likely show the binary at /app/travel-rs
RUN echo "--- Debugging: Contents of /app/target/release/ (might be empty due to cache mount isolation) ---" && ls -la /app/target/release/ || echo "Directory /app/target/release not found or empty for this platform."
RUN echo "--- Debugging: Contents of /app/target/ (might be empty due to cache mount isolation) ---" && ls -la /app/target/ || echo "Directory /app/target not found or empty for this platform."
RUN echo "--- Debugging: Contents of /app/ (should now contain travel-rs) ---" && ls -la /app/ || echo "Directory /app not found or empty for this platform."
RUN echo "--- Debugging: Searching for 'travel-rs' binary in /app/ ---" && find /app -name "travel-rs*" -print || echo "No 'travel-rs' binary found in /app for this platform."


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
