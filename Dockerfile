# Multi-stage Dockerfile for IP Geolocation Service
# Uses Chainguard's secure, minimal container images
# Reference: https://images.chainguard.dev/directory/image/rust/overview

# =============================================================================
# Stage 1: Download GeoLite2 database and flag icons
# =============================================================================
FROM cgr.dev/chainguard/wolfi-base AS assets-downloader

# MaxMind account ID and license key are required to download GeoLite2
# Get a free account at: https://www.maxmind.com/en/geolite2/signup
ARG MAXMIND_ACCOUNT_ID
ARG MAXMIND_LICENSE_KEY

RUN apk add --no-cache curl git

WORKDIR /assets

# Download GeoLite2-City database
# The database is approximately 70MB compressed
RUN if [ -n "$MAXMIND_LICENSE_KEY" ] && [ -n "$MAXMIND_ACCOUNT_ID" ]; then \
        echo "Downloading GeoLite2-City database..." && \
        curl -fSL "https://download.maxmind.com/geoip/databases/GeoLite2-City/download?suffix=tar.gz" \
            -u "${MAXMIND_ACCOUNT_ID}:${MAXMIND_LICENSE_KEY}" \
            -o GeoLite2-City.tar.gz && \
        tar -xzf GeoLite2-City.tar.gz && \
        mv GeoLite2-City_*/GeoLite2-City.mmdb . && \
        rm -rf GeoLite2-City.tar.gz GeoLite2-City_* && \
        echo "GeoLite2-City database downloaded successfully"; \
    else \
        echo "Warning: MAXMIND_ACCOUNT_ID and MAXMIND_LICENSE_KEY not provided." && \
        echo "You must mount a GeoLite2-City.mmdb file at /app/data/GeoLite2-City.mmdb" && \
        touch GeoLite2-City.mmdb; \
    fi

# Download flag icons from lipis/flag-icons (MIT license)
# Using 4x3 aspect ratio SVGs (~1.5MB total)
RUN git clone --depth 1 --filter=blob:none --sparse https://github.com/lipis/flag-icons.git /tmp/flag-icons && \
    cd /tmp/flag-icons && \
    git sparse-checkout set flags/4x3 && \
    mkdir -p /assets/flags && \
    cp flags/4x3/*.svg /assets/flags/ && \
    rm -rf /tmp/flag-icons

# =============================================================================
# Stage 2: Build Rust application using Chainguard Rust image
# =============================================================================
FROM cgr.dev/chainguard/rust:latest-dev AS builder

# Install protobuf compiler (required for prost-build)
# Using -dev variant which includes apk
RUN apk add --no-cache protobuf-dev

# Chainguard images run as nonroot user - use their home directory
WORKDIR /home/nonroot/app

# Copy manifests and build files first for better layer caching
COPY --chown=nonroot:nonroot Cargo.toml Cargo.lock* ./
COPY --chown=nonroot:nonroot build.rs ./
COPY --chown=nonroot:nonroot proto ./proto

# Create dummy source files to build dependencies
# Must match all [[bin]] and lib targets in Cargo.toml
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/mcp_server.rs && \
    echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code and other files
COPY --chown=nonroot:nonroot src ./src
COPY --chown=nonroot:nonroot llms.txt ./llms.txt

# Touch main.rs to ensure rebuild with actual code
RUN touch src/main.rs

# Build the application (cargo-auditable is used by default for security scanning)
RUN cargo build --release

# =============================================================================
# Stage 3: Runtime image using Chainguard glibc-dynamic
# =============================================================================
FROM cgr.dev/chainguard/glibc-dynamic:latest AS runtime

WORKDIR /app

# Copy binary from builder (chainguard images use nonroot user by default)
COPY --from=builder --chown=nonroot:nonroot /home/nonroot/app/target/release/ipgeolocation /app/ipgeolocation

# Create data directory and copy GeoIP database
COPY --from=assets-downloader --chown=nonroot:nonroot /assets/GeoLite2-City.mmdb /app/data/GeoLite2-City.mmdb

# Copy flag icons for static serving
COPY --from=assets-downloader --chown=nonroot:nonroot /assets/flags /app/static/flags

# Environment variables with defaults
ENV BIND_ADDRESS=0.0.0.0:3000
ENV GEOIP_DB_PATH=/app/data/GeoLite2-City.mmdb
ENV STATIC_DIR=/app/static
ENV CACHE_SIZE=10000
ENV CACHE_TTL_SECS=3600
ENV RUST_LOG=ipgeolocation=info

EXPOSE 3000

ENTRYPOINT ["/app/ipgeolocation"]
