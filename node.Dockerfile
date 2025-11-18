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

# Copy sources
COPY shared ./shared
COPY router ./router
COPY node ./node

# Build only the node binary from the workspace
RUN cargo build -p commonware-avs-node --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/commonware-avs-node /usr/local/bin/commonware-avs-node

ENTRYPOINT ["/usr/local/bin/commonware-avs-node"]