# docker buildx build --platform linux/arm64 --target binary --output type=local,dest=$(pwd)/target/docker .

FROM --platform=$BUILDPLATFORM rust:1-bookworm AS vendor

ENV USER=root

WORKDIR /code
RUN cargo init --bin --name git-metrics /code
COPY Cargo.lock Cargo.toml /code/
COPY .cargo /code/.cargo

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor >> /code/.cargo/config.toml

FROM --platform=amd64 rust:1-bookworm AS base-builder

RUN cargo install cargo-deb

FROM --platform=amd64 base-builder AS amd64-builder

RUN apt-get update \
    && apt-get install -y git \
    && rm -rf /var/lib/apt/lists

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY src /code/src
COPY LICENSE /code/LICENSE
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor

RUN --mount=type=cache,target=/code/target/x86_64-unknown-linux-gnu/release/deps,sharing=locked \
    --mount=type=cache,target=/code/target/x86_64-unknown-linux-gnu/release/build,sharing=locked \
    --mount=type=cache,target=/code/target/x86_64-unknown-linux-gnu/release/incremental,sharing=locked \
    cargo build --release --offline --target x86_64-unknown-linux-gnu

RUN strip /code/target/x86_64-unknown-linux-gnu/release/git-metrics
RUN cargo deb --no-build --target x86_64-unknown-linux-gnu

FROM --platform=amd64 base-builder AS arm64-builder

RUN rustup target add aarch64-unknown-linux-gnu

RUN dpkg --add-architecture arm64
RUN apt-get update \
    && apt-get install -y libssl-dev:arm64 gcc-aarch64-linux-gnu g++-aarch64-linux-gnu binutils-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY src /code/src
COPY LICENSE /code/LICENSE
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor

ENV OPENSSL_INCLUDE_DIR=/usr/include/aarch64-linux-gnu/openssl
ENV OPENSSL_LIB_DIR=/usr/lib/aarch64-linux-gnu
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

RUN --mount=type=cache,target=/code/target/aarch64-unknown-linux-gnu/release/deps,sharing=locked \
    --mount=type=cache,target=/code/target/aarch64-unknown-linux-gnu/release/build,sharing=locked \
    --mount=type=cache,target=/code/target/aarch64-unknown-linux-gnu/release/incremental,sharing=locked \
    cargo build --release --offline --target aarch64-unknown-linux-gnu

RUN /usr/aarch64-linux-gnu/bin/strip /code/target/aarch64-unknown-linux-gnu/release/git-metrics
RUN cargo deb --no-build --target aarch64-unknown-linux-gnu

FROM scratch AS binary

COPY --from=amd64-builder /code/target/x86_64-unknown-linux-gnu/release/git-metrics /git-metrics_linux-x86_64
COPY --from=amd64-builder /code/target/x86_64-unknown-linux-gnu/debian/*.deb /
COPY --from=arm64-builder /code/target/aarch64-unknown-linux-gnu/release/git-metrics /git-metrics_linux-aarch64
COPY --from=arm64-builder /code/target/aarch64-unknown-linux-gnu/debian/*.deb /
