# Container image for the Solana/Jito e2e leg: Rust toolchain + Agave
# (solana-test-validator, cargo-build-sbf). The compose service mounts the
# repository and runs scripts/solana_e2e_local.sh — the exact same driver CI
# and local runs use, so there is exactly ONE definition of the e2e.
FROM rust:1.88-slim-bookworm

ARG AGAVE_VERSION=v4.1.1

RUN apt-get update && apt-get install -y --no-install-recommends \
        bzip2 \
        ca-certificates \
        clang \
        curl \
        git \
        libssl-dev \
        libudev-dev \
        pkg-config \
        procps \
    && rm -rf /var/lib/apt/lists/*

RUN sh -c "$(curl -sSfL https://release.anza.xyz/${AGAVE_VERSION}/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

WORKDIR /work
ENTRYPOINT ["bash", "scripts/solana_e2e_local.sh"]
