#!/bin/bash

set -xe

export RUNNER_ARCH=$(uname -m)
export RUNNER_OS=$(uname)
export RUSTC_VERSION=$(rustc --version)
export BINARY_SIZE=$(ls -la target/release/git-metrics | cut -d ' ' -f 5)

target/release/git-metrics add \
    binary-size \
    --tag "unit: byte" \
    --tag "platform.arch: $RUNNER_ARCH" \
    --tag "platform.os: $RUNNER_OS" \
    --tag "rust.version: $RUSTC_VERSION" \
    $BINARY_SIZE
