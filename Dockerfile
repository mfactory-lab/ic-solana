# =======================
# Base Stage
# =======================

FROM --platform=linux/amd64 ubuntu@sha256:bbf3d1baa208b7649d1d0264ef7d522e1dc0deeeaaf6085bf8e4618867f03494 AS base

ENV TZ=UTC \
    DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone \
    && apt-get update -qq \
    && apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
        build-essential \
        pkg-config \
        libssl-dev \
        llvm-dev \
        liblmdb-dev \
        clang \
        cmake \
        jq \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# =======================
# Builder Stage
# =======================

FROM base AS builder

ARG BUILD_MODE=production
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/cargo \
    PATH=/cargo/bin:$PATH \
    BUILD_MODE=${BUILD_MODE}

SHELL ["bash", "-c"]
WORKDIR /code

RUN mkdir -p ./scripts
COPY ./scripts/bootstrap ./scripts/bootstrap
COPY ./rust-toolchain.toml ./rust-toolchain.toml

RUN ./scripts/bootstrap

COPY . .

# =======================
# Build RPC Stage
# =======================

FROM builder AS build_rpc
LABEL io.icp.artifactType="canister" \
      io.icp.artifactName="rpc"
RUN mkdir -p ./artifacts \
    && ./scripts/build --rpc \
    &&  cp ./ic-solana-rpc.wasm.gz ./artifacts/ic-solana-rpc.wasm.gz \
    && ./scripts/did ic-solana-rpc \
    && cp ./src/ic-solana-rpc/ic-solana-rpc.did ./artifacts/ic-solana-rpc.did \
    && sha256sum ./ic-solana-rpc.wasm.gz | awk '{ print $1 }' > ./artifacts/ic-solana-rpc.wasm.gz.sha256

# =======================
# Build Wallet Stage
# =======================

FROM builder AS build_wallet
LABEL io.icp.artifactType="canister" \
      io.icp.artifactName="wallet"
RUN mkdir -p ./artifacts \
    && ./scripts/build --wallet \
    &&  cp ./ic-solana-wallet.wasm.gz ./artifacts/ic-solana-wallet.wasm.gz \
    && ./scripts/did ic-solana-wallet \
    && cp ./src/ic-solana-wallet/ic-solana-wallet.did ./artifacts/ic-solana-wallet.did \
    && sha256sum ./ic-solana-wallet.wasm.gz | awk '{ print $1 }' > ./artifacts/ic-solana-wallet.wasm.gz.sha256
