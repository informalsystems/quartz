#!/bin/bash

# Build and optimize the contract to output a WASM binary that can be deployed to a CosmWasm chain.

set -euo pipefail

if ! [ -f "Cargo.toml" ]; then
  echo "❌ Error: Cannot find 'Cargo.toml' in current directory. Make sure this command is run from the contract's source directory"
  exit 1
fi

echo "👷 Building and optimizing the contract..."
echo "==========================================="

RUSTFLAGS='-C link-arg=-s' cargo wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.15.0

echo ""
echo "🎉 Contract build and optimization completed successfully!"
