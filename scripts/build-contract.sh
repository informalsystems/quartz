#!/bin/bash

# Build and optimize the contract to output a WASM binary that can be deployed to a CosmWasm chain.

set -euo pipefail

usage() {
    echo "Usage: $0 CARGO_PKG_DIR"
    echo "Example: $0 apps/mtcs/contracts/cw-tee-mtcs/"
    exit 1
}

if [ $# -ne 1 ]; then
    echo "❌ Error: Missing CARGO_PKG_DIR parameter. Please check if all parameters were specified."
    usage
fi

if ! [ -f "$1/Cargo.toml" ]; then
  echo "❌ Error: Cannot find 'Cargo.toml' in current directory. Make sure the contract's source directory is $1"
    usage
fi

ROOT=${ROOT:-$(pwd)}
CARGO_PKG_DIR="$1"

cd "$CARGO_PKG_DIR"

echo "👷 Building and optimizing the contract..."
echo "==========================================="

RUSTFLAGS='-C link-arg=-s' cargo wasm

docker run --rm -v "$ROOT":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target="/code/target" \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.15.0 "$CARGO_PKG_DIR"

echo ""
echo "🎉 Contract build and optimization completed successfully!"
