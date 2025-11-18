FROM rust:1.83-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests and member manifests for dependency resolution
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY shared/Cargo.toml ./shared/Cargo.toml
COPY router/Cargo.toml ./router/Cargo.toml
COPY node/Cargo.toml ./node/Cargo.toml
COPY scripts/Cargo.toml ./scripts/Cargo.toml

# Create placeholder targets for workspace members so Cargo can resolve manifests
RUN mkdir -p node/src router/src shared/src scripts/src \
    && echo 'fn main(){}' > node/src/main.rs \
    && echo 'fn main(){}' > router/src/main.rs \
    && echo 'pub fn _placeholder(){}' > shared/src/lib.rs \
    && echo 'fn main(){}' > scripts/src/main.rs \
    && cargo fetch

# Copy sources needed to build the router package
COPY shared ./shared
COPY router ./router

# Build only the router binary from the workspace
RUN cargo build -p commonware-avs-router --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 -s /bin/bash appuser

# Copy the binary from builder
COPY --from=builder /app/target/release/commonware-avs-router /usr/local/bin/commonware-avs-router

# Copy configuration files
COPY config /app/config

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Set working directory
WORKDIR /app

# Expose port 3000 for the application
EXPOSE 3000

# Run the binary
ENTRYPOINT ["commonware-avs-router"]

