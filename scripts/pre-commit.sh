#! /usr/bin/env bash

export RUSTFLAGS="-Dwarnings"
export RUSTDOCFLAGS="--cfg docsrs -Dwarnings"

declare -A LOCALDEPS=(
  [hightorrent]="https://github.com/angrynode/hightorrent"
)

set -e
set -x

if [[ "$1" = "tag" ]]; then
  # When testing for a tag, we only want to use released versions on crates.io
  cp Cargo.toml Cargo.toml.bak
  sed -ri 's/, ?path ?= ?".*"//' Cargo.toml
else
  # If testing in CI, we want to download the latest main branch of LOCALDEPS
  for dep in ${!LOCALDEPS[@]}; do
    git clone --depth=1 "${LOCALDEPS[${dep}]}" ../$dep
  done
fi

cargo +nightly --version
cargo +nightly fmt --check
cargo +nightly clippy --all-features
cargo +nightly test
cargo +nightly test --all-features
cargo +nightly doc --no-deps -Zrustdoc-map --all-features
if command -v cargo-rdme 2>&1 >/dev/null; then
  cargo-rdme --check
else
  echo "Skip cargo-rdme (not installed)"
fi
