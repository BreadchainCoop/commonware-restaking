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
COPY usecase-counter/Cargo.toml ./usecase-counter/Cargo.toml
COPY usecase-counter/node/Cargo.toml ./usecase-counter/node/Cargo.toml
COPY usecase-counter/router/Cargo.toml ./usecase-counter/router/Cargo.toml

# Create placeholder targets for workspace members so Cargo can resolve manifests
RUN mkdir -p core/src bindings/src router/src node/src scripts/src usecase-counter/src usecase-counter/node/src usecase-counter/router/src \
    && echo 'pub fn _placeholder(){}' > core/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > bindings/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > router/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > node/src/lib.rs \
    && echo 'pub fn _placeholder(){}' > usecase-counter/lib.rs \
    && echo 'fn main(){}' > usecase-counter/node/src/app.rs \
    && echo 'fn main(){}' > usecase-counter/router/src/app.rs \
    && echo 'fn main(){}' > scripts/src/main.rs \
    && cargo fetch

# Copy sources
COPY core ./core
COPY bindings ./bindings
COPY node ./node
COPY router ./router
COPY usecase-counter ./usecase-counter

# Build only the node binary from the workspace
RUN cargo build -p commonware-avs-node-app --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/commonware-avs-node-app /usr/local/bin/commonware-avs-node-app

ENTRYPOINT ["/usr/local/bin/commonware-avs-node-app"]