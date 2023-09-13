#!/usr/bin/env bash

set -xe

echo "Building:"
cargo build --release --target wasm32-unknown-unknown --bin dndhelper --features "bevy"

echo "Generating WASM JS:"
wasm-bindgen --out-name dndhelper_wasm --out-dir ./public/wasm --target web target/wasm32-unknown-unknown/release/dndhelper.wasm

echo "Generating nations assets"
cargo run --release --bin generate_assets_file

echo "Copying assets:"
cp -r ./assets ./public/assets

echo "done building wasm"