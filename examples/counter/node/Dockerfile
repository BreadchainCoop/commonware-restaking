# Build stage
FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests first for better caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY core/Cargo.toml ./core/Cargo.toml
COPY bindings/Cargo.toml ./bindings/Cargo.toml
COPY eigenlayer/Cargo.toml ./eigenlayer/Cargo.toml
COPY router/Cargo.toml ./router/Cargo.toml
COPY scripts/Cargo.toml ./scripts/Cargo.toml
COPY node/Cargo.toml ./node/Cargo.toml
COPY examples/counter/bindings/Cargo.toml ./examples/counter/bindings/Cargo.toml
COPY examples/counter/common/Cargo.toml ./examples/counter/common/Cargo.toml
COPY examples/counter/node/Cargo.toml ./examples/counter/node/Cargo.toml
COPY examples/counter/router/Cargo.toml ./examples/counter/router/Cargo.toml

# Create placeholder targets for workspace members so Cargo can resolve manifests
RUN mkdir -p core/src bindings/src eigenlayer/src router/src node/src scripts/src \
    examples/counter/bindings/src examples/counter/common/src \
    examples/counter/node/src examples/counter/router/src \
    && echo 'pub fn _placeholder(){}' > core/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > bindings/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > eigenlayer/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > router/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > node/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > examples/counter/bindings/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > examples/counter/common/src/lib.rs \
    && echo 'fn main(){}' > examples/counter/node/src/app.rs \
    && echo 'fn main(){}' > examples/counter/router/src/app.rs \
    && echo 'fn main(){}' > scripts/src/main.rs \
    && cargo fetch

# Copy sources
COPY core ./core
COPY bindings ./bindings
COPY eigenlayer ./eigenlayer
COPY node ./node
COPY router ./router
COPY examples ./examples

# Build only the node binary from the workspace
ARG BUILD_JOBS=2
RUN cargo build -p counter-node --release --jobs $BUILD_JOBS

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/counter-node /usr/local/bin/counter-node

ENTRYPOINT ["/usr/local/bin/counter-node"]
