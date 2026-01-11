# Multi-stage Dockerfile for IP Geolocation Service
# Uses Chainguard's secure, minimal container images

# =============================================================================
# Stage 1: Download GeoLite2 database and flag icons
# =============================================================================
FROM cgr.dev/chainguard/wolfi-base:latest AS assets

ARG MAXMIND_ACCOUNT_ID
ARG MAXMIND_LICENSE_KEY

RUN apk add --no-cache curl git

WORKDIR /assets

# Download GeoLite2-City database if credentials provided.
# If not provided, a placeholder is created and you MUST mount a real database
# at runtime: docker run -v /path/to/GeoLite2-City.mmdb:/app/data/GeoLite2-City.mmdb
RUN if [ -n "$MAXMIND_LICENSE_KEY" ] && [ -n "$MAXMIND_ACCOUNT_ID" ]; then \
        curl -fsSL "https://download.maxmind.com/geoip/databases/GeoLite2-City/download?suffix=tar.gz" \
            -u "${MAXMIND_ACCOUNT_ID}:${MAXMIND_LICENSE_KEY}" \
            -o GeoLite2-City.tar.gz && \
        tar -xzf GeoLite2-City.tar.gz --strip-components=1 --wildcards '*/GeoLite2-City.mmdb' && \
        rm -f GeoLite2-City.tar.gz; \
    else \
        echo "WARNING: No MaxMind credentials provided." && \
        echo "You MUST mount a GeoLite2-City.mmdb file at /app/data/GeoLite2-City.mmdb" && \
        touch GeoLite2-City.mmdb; \
    fi

# Download flag icons from lipis/flag-icons (MIT license)
RUN git clone --depth 1 --filter=blob:none --sparse \
        https://github.com/lipis/flag-icons.git /tmp/flags && \
    cd /tmp/flags && \
    git sparse-checkout set flags/4x3 && \
    mkdir -p /assets/flags && \
    mv flags/4x3/*.svg /assets/flags/ && \
    rm -rf /tmp/flags

# =============================================================================
# Stage 2: Build Rust application
# =============================================================================
FROM cgr.dev/chainguard/rust:latest-dev AS builder

# Install protobuf compiler for prost-build (requires root)
USER root
RUN apk add --no-cache protobuf-dev

# Create build directory with correct ownership
RUN mkdir -p /build && chown nonroot:nonroot /build
USER nonroot

WORKDIR /build

# Copy everything needed for the build
# Note: .dockerignore excludes target/, .git/, etc.
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY proto ./proto
COPY src ./src
COPY llms.txt ./

# Build release binaries (limit jobs to reduce memory usage in constrained environments)
RUN cargo build --release --locked -j 2

# =============================================================================
# Stage 3: Runtime image
# =============================================================================
FROM cgr.dev/chainguard/glibc-dynamic:latest

WORKDIR /app

# Copy binaries
COPY --from=builder /build/target/release/ipgeolocation /app/
COPY --from=builder /build/target/release/mcp_server /app/

# Copy assets
COPY --from=assets /assets/GeoLite2-City.mmdb /app/data/
COPY --from=assets /assets/flags /app/static/flags/

# Note: llms.txt is embedded in the binary via include_str! at compile time

# Configuration via environment variables
ENV BIND_ADDRESS=0.0.0.0:3000 \
    GEOIP_DB_PATH=/app/data/GeoLite2-City.mmdb \
    STATIC_DIR=/app/static \
    CACHE_SIZE=10000 \
    CACHE_TTL_SECS=3600 \
    RUST_LOG=ipgeolocation=info

EXPOSE 3000

# Note: For health checks, configure your orchestrator to probe GET /health
# Example for Kubernetes:
#   livenessProbe:
#     httpGet:
#       path: /health
#       port: 3000

ENTRYPOINT ["/app/ipgeolocation"]
