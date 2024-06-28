# docker buildx build --platform linux/arm64 --target binary --output type=local,dest=$(pwd)/target/docker .

FROM --platform=$BUILDPLATFORM rust:1-bookworm AS vendor

ENV USER=root

WORKDIR /code
RUN cargo init --bin --name git-metrics /code
COPY Cargo.lock Cargo.toml /code/

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor >> /code/.cargo/config.toml

FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y git \
    && rm -rf /var/lib/apt/lists

RUN cargo install cargo-deb

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY src /code/src
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor

RUN cargo build --release --offline \
    && strip /code/target/release/git-metrics

COPY LICENSE /code/LICENSE
COPY readme.md /code/readme.md

RUN cargo deb --no-build --package git-metrics

FROM scratch AS binary

COPY --from=builder /code/target/release/git-metrics /git-metrics
COPY --from=builder /code/target/debian/git-metrics_*.deb /
