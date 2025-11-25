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
COPY router/Cargo.toml ./router/Cargo.toml
COPY scripts/Cargo.toml ./scripts/Cargo.toml
COPY node/Cargo.toml ./node/Cargo.toml
COPY usecases/Cargo.toml ./usecases/Cargo.toml
COPY usecases/counter/node/Cargo.toml ./usecases/counter/node/Cargo.toml
COPY usecases/counter/router/Cargo.toml ./usecases/counter/router/Cargo.toml

# Create placeholder targets for workspace members so Cargo can resolve manifests
RUN mkdir -p core/src bindings/src router/src node/src scripts/src usecases/src usecases/counter/node/src usecases/counter/router/src \
    && echo 'pub fn _placeholder(){}' > core/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > bindings/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > router/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > node/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > usecases/src/lib.rs \
    && echo 'fn main(){}' > usecases/counter/node/src/app.rs \
    && echo 'fn main(){}' > usecases/counter/router/src/app.rs \
    && echo 'fn main(){}' > scripts/src/main.rs \
    && cargo fetch

# Copy sources
COPY core ./core
COPY bindings ./bindings
COPY node ./node
COPY router ./router
COPY usecases ./usecases

# Build only the node binary from the workspace
ARG BUILD_JOBS=2
RUN cargo build -p commonware-avs-node-app --release --jobs $BUILD_JOBS

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/commonware-avs-node-app /usr/local/bin/commonware-avs-node-app

ENTRYPOINT ["/usr/local/bin/commonware-avs-node-app"]
