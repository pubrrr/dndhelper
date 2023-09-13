#!/usr/bin/env bash

set -xe

echo "Installing Rustup..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Adding binaries to path
source "$HOME/.cargo/env"

rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

source ./build-wasm.sh

echo "done"
