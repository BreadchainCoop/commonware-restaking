FROM rust:1.83-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests and member manifests for dependency resolution
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY core/Cargo.toml ./core/Cargo.toml
COPY bindings/Cargo.toml ./bindings/Cargo.toml
COPY eigenlayer/Cargo.toml ./eigenlayer/Cargo.toml
COPY node/Cargo.toml ./node/Cargo.toml
COPY scripts/Cargo.toml ./scripts/Cargo.toml
COPY router/Cargo.toml ./router/Cargo.toml
COPY examples/counter/bindings/Cargo.toml ./examples/counter/bindings/Cargo.toml
COPY examples/counter/common/Cargo.toml ./examples/counter/common/Cargo.toml
COPY examples/counter/router/Cargo.toml ./examples/counter/router/Cargo.toml
COPY examples/counter/node/Cargo.toml ./examples/counter/node/Cargo.toml

# Create placeholder targets for workspace members so Cargo can resolve manifests
RUN mkdir -p core/src bindings/src eigenlayer/src router/src node/src scripts/src \
    examples/counter/bindings/src examples/counter/common/src \
    examples/counter/router/src examples/counter/node/src \
    && echo 'pub fn _placeholder(){}' > core/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > bindings/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > eigenlayer/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > router/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > node/src/lib.rs \
    && echo 'fn main(){}' > scripts/src/main.rs \
    && echo 'pub fn _placeholder(){}' > examples/counter/bindings/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > examples/counter/common/src/lib.rs \
    && echo 'fn main(){}' > examples/counter/router/src/app.rs \
    && echo 'fn main(){}' > examples/counter/node/src/app.rs \
    && cargo fetch

# Copy sources needed to build the router package
COPY core ./core
COPY bindings ./bindings
COPY eigenlayer ./eigenlayer
COPY router ./router
COPY node ./node
COPY examples ./examples

# Build only the router binary from the workspace
ARG BUILD_JOBS=2
RUN cargo build -p counter-router --release --jobs $BUILD_JOBS

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
COPY --from=builder /app/target/release/counter-router /usr/local/bin/counter-router

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
ENTRYPOINT ["counter-router"]
