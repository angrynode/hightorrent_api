#! /usr/bin/env bash

export RUSTFLAGS="-Dwarnings"
export RUSTDOCFLAGS="--cfg docsrs -Dwarnings"

set -e

cargo +nightly --version
cargo +nightly fmt --check
cargo +nightly clippy --all-features
cargo +nightly test
cargo +nightly test --all-features
cargo +nightly doc --no-deps -Zrustdoc-map --all-features
cargo-rdme --check
