FROM --platform=$BUILDPLATFORM rust:1-bookworm AS vendor

ENV USER=root

WORKDIR /code
RUN cargo init
COPY Cargo.lock /code/Cargo.lock
COPY Cargo.toml /code/Cargo.toml

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor >> /code/.cargo/config.toml

FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

ENV USER=root

WORKDIR /code

COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor
COPY src /code/src

RUN cargo build --release \
    --package git-metrics \
    --no-default-features \
    --features impl-command \
    --offline

FROM alpine

RUN apk add --no-cache git
RUN git config --global --add safe.directory /github/workspace

COPY --from=builder /code/target/release/git-metrics /usr/bin/git-metrics

ENTRYPOINT ["/usr/bin/git-metrics"]
CMD ["--help"]
