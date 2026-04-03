FROM alpine:latest
WORKDIR /app

# Install runtime dependencies (ca-certs for Telegram API)
RUN apk add --no-cache ca-certificates

# Arguments from GitHub Actions
ARG TARGETPLATFORM
ARG BIN_PATH_AMD64
ARG BIN_PATH_ARM64
ARG BIN_PATH_ARMV7

# Copy the pre-built binaries
COPY ${BIN_PATH_AMD64} /app/travel-rs-amd64
COPY ${BIN_PATH_ARM64} /app/travel-rs-arm64
COPY ${BIN_PATH_ARMV7} /app/travel-rs-armv7

# Select the correct binary based on the platform being built
RUN case "${TARGETPLATFORM}" in \
        "linux/amd64")  mv /app/travel-rs-amd64 /app/travel-rs ;; \
        "linux/arm64")  mv /app/travel-rs-arm64 /app/travel-rs ;; \
        "linux/arm/v7") mv /app/travel-rs-armv7 /app/travel-rs ;; \
    esac && \
    chmod +x /app/travel-rs && \
    rm -f /app/travel-rs-*

RUN mkdir -p /app/config /app/locales /app/logs

# Declare volumes for external data
VOLUME ["/app/config", "/app/locales", "/app/logs"]

# Set environment variables
ENV RUST_LOG=info

# Define the default command to run the application
CMD ["./travel-rs"]
