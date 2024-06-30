#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$HOME}

echo "--------------------------------------------------------"
echo "building enclave binary"

cd $ROOT/cycles-quartz/apps/transfers/enclave
CARGO_TARGET_DIR=./target cargo build --release

echo "--------------------------------------------------------"
echo "building cosmwasm contract binary"


cd $ROOT/cycles-quartz/apps/transfers/contracts
bash build.sh
