# Multi-stage build for AI Manager
FROM rust:1.83-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml ./
COPY crates/ ./crates/

# Build dependencies (this is the caching Docker layer!)
RUN cargo build --release --workspace

# Copy source code
COPY . .

# Build application
RUN cargo build --release -p ai-manager-core

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r app && useradd -r -g app app

# Create app directory and set permissions
WORKDIR /app
RUN chown app:app /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/ai-manager-core /app/ai-manager-core
COPY --from=builder /app/crates/shared/config/default.toml /app/config/default.toml

# Create necessary directories
RUN mkdir -p /app/{data,logs,credentials} && \
    chown -R app:app /app

# Switch to app user
USER app

# Expose port (if needed for future web interface)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ./ai-manager-core --health-check || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV AI_MANAGER_CONFIG_PATH=/app/config/default.toml
ENV AI_MANAGER_DATA_PATH=/app/data
ENV AI_MANAGER_LOG_PATH=/app/logs

# Run the application
CMD ["./ai-manager-core"]
