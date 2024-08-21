#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$(git rev-parse --show-toplevel)}

FEATURES=

if [ -n "$MOCK_SGX" ]; then
    echo "MOCK_SGX is set. Adding mock-sgx feature."
    FEATURES="--features=mock-sgx"
fi

echo "--------------------------------------------------------"
echo "building enclave binary"

cd $ROOT/apps/mtcs/enclave/
CARGO_TARGET_DIR=./target cargo build --release $FEATURES

echo "--------------------------------------------------------"
echo "building cosmwasm contract binary"


cd $ROOT/apps/mtcs/contracts/cw-tee-mtcs/

RUSTFLAGS='-C link-arg=-s' cargo wasm $FEATURES
