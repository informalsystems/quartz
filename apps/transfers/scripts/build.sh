#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$HOME}
FEATURES=

if [ -n "$MOCK_SGX" ]; then
    echo "MOCK_SGX is set. Adding mock-sgx feature."
    FEATURES="--features=mock-sgx"
fi

echo "--------------------------------------------------------"
echo "building enclave binary"

cd $ROOT/cycles-quartz/apps/transfers/enclave
CARGO_TARGET_DIR=./target cargo build --release $FEATURES

echo "--------------------------------------------------------"
echo "building cosmwasm contract binary"


cd $ROOT/cycles-quartz/apps/transfers/contracts
bash build.sh $FEATURES
