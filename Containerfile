# PII Redacta - Containerfile
# Multi-stage build for production-ready Rust binary

# Stage 1: Build
FROM docker.io/rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    postgresql-dev

WORKDIR /build

# Copy workspace manifests first (for better caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/pii_redacta_core/Cargo.toml crates/pii_redacta_core/
COPY crates/pii_redacta_api/Cargo.toml crates/pii_redacta_api/

# Create dummy main.rs to cache dependencies
RUN mkdir -p crates/pii_redacta_core/src crates/pii_redacta_api/src && \
    echo "fn main() {}" > crates/pii_redacta_core/src/lib.rs && \
    echo "fn main() {}" > crates/pii_redacta_api/src/main.rs && \
    cargo build --release && \
    rm -rf crates/pii_redacta_core/src crates/pii_redacta_api/src

# Copy actual source code
COPY crates/ ./crates/

# Touch to force rebuild
RUN touch crates/pii_redacta_core/src/lib.rs crates/pii_redacta_api/src/main.rs

# Build release binary (static linking)
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release && \
    strip target/release/pii-redacta-api

# Stage 2: Runtime
FROM docker.io/alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    postgresql-client \
    && rm -rf /var/cache/apk/*

# Create non-root user
RUN addgroup -g 1000 -S pii-redacta && \
    adduser -u 1000 -S pii-redacta -G pii-redacta

# Copy binary from builder
COPY --from=builder /build/target/release/pii-redacta-api /usr/local/bin/

# Set permissions
RUN chmod +x /usr/local/bin/pii-redacta-api && \
    chown -R pii-redacta:pii-redacta /usr/local/bin/pii-redacta-api

# Use non-root user
USER pii-redacta

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080

# Run the binary
CMD ["pii-redacta-api"]
