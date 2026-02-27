# PII Redacta - MVP Release
# Build stage
FROM rust:1.75-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build release binary
RUN cargo build --release -p pii_redacta_api

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r pii-redacta && useradd -r -g pii-redacta pii-redacta

# Copy binary from builder
COPY --from=builder /app/target/release/pii-redacta-api /usr/local/bin/

# Change ownership
RUN chown -R pii-redacta:pii-redacta /app

# Switch to non-root user
USER pii-redacta

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["pii-redacta-api"]
