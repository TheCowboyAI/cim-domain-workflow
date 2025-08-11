# CIM Domain Workflow Docker Image
# Multi-stage build for optimized production image

# Build stage
FROM rust:1.75 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash cim-user

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/cim-domain-workflow /app/

# Copy configuration files
COPY deployment/configs/default-config.toml /app/config/
COPY deployment/configs/production-config.toml /app/config/

# Set ownership
RUN chown -R cim-user:cim-user /app

# Switch to non-root user
USER cim-user

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Build arguments for metadata
ARG VERSION=unknown
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown

# Add metadata labels
LABEL org.opencontainers.image.title="CIM Domain Workflow" \
      org.opencontainers.image.description="Enterprise workflow orchestration with algebraic foundations" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.vendor="The CIM Consortium" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"

# Set environment variables
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config/production-config.toml

# Run the application
ENTRYPOINT ["/app/cim-domain-workflow"]