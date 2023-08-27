echo "Installing Rustup..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Adding binaries to path
source "$HOME/.cargo/env"

rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

echo "Building:"
cargo build --release --target wasm32-unknown-unknown

echo "Generating WASM JS:"
wasm-bindgen --out-name dndhelper_wasm --out-dir ./public/wasm --target web target/wasm32-unknown-unknown/release/dndhelper.wasm

echo "done"
