# Use this with
#
#  docker build -t ic-solana .
#  or use ./scripts/docker-build
#
# The docker image. To update, run `docker pull ubuntu` locally, and update the
# sha256:... accordingly.

FROM --platform=linux/amd64 ubuntu@sha256:bbf3d1baa208b7649d1d0264ef7d522e1dc0deeeaaf6085bf8e4618867f03494 as deps

ENV TZ=UTC

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    apt -yq update && \
    apt -yqq install --no-install-recommends curl ca-certificates \
        build-essential pkg-config libssl-dev llvm-dev liblmdb-dev clang cmake jq

# Install Rust and Cargo in /opt
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/cargo \
    PATH=/cargo/bin:$PATH

RUN mkdir -p ./scripts
COPY ./scripts/bootstrap ./scripts/bootstrap
COPY ./rust-toolchain.toml ./rust-toolchain.toml

RUN ./scripts/bootstrap
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN wasm-pack --version

# Pre-build all cargo dependencies. Because cargo doesn't have a build option
# to build only the dependecies, we pretend that our project is a simple, empty
# `lib.rs`. When we COPY the actual files we make sure to `touch` lib.rs so
# that cargo knows to rebuild it with the new content.
COPY Cargo.lock .
COPY Cargo.toml .
COPY src/ic-solana/Cargo.toml src/ic-solana/Cargo.toml
COPY src/ic-solana-rpc/Cargo.toml src/ic-solana-rpc/Cargo.toml
COPY src/ic-solana-wallet/Cargo.toml src/ic-solana-wallet/Cargo.toml
COPY src/e2e/Cargo.toml src/e2e/Cargo.toml

ENV CARGO_TARGET_DIR=/cargo_target
COPY ./scripts/build ./scripts/build
RUN mkdir -p src/ic-solana/src \
    && touch src/ic-solana/src/lib.rs \
    && mkdir -p src/ic-solana-rpc/src \
    && touch src/ic-solana-rpc/src/lib.rs \
    && mkdir -p src/ic-solana-wallet/src \
    && touch src/ic-solana-wallet/src/lib.rs \
    && mkdir -p src/e2e/src \
    && touch src/e2e/src/lib.rs \
    && ./scripts/build --only-dependencies --all \
    && rm -rf src

FROM deps as build_ic_solana_rpc

COPY . .
RUN touch src/*/src/lib.rs
RUN ./scripts/build --rpc
RUN sha256sum /ic-solana-rpc.wasm.gz

FROM deps as build_ic_solana_wallet

COPY . .
RUN touch src/*/src/lib.rs
RUN ./scripts/build --wallet
RUN sha256sum /ic-solana-wallet.wasm.gz

FROM scratch AS scratch_ic_solana_rpc
COPY --from=build_ic_solana_rpc /ic-solana-rpc.wasm.gz /

FROM scratch AS scratch_ic_solana_wallet
COPY --from=build_ic_solana_wallet /ic-solana-wallet.wasm.gz /
