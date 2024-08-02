#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$HOME}

echo "--------------------------------------------------------"
echo "building enclave binary"

cd $ROOT/cycles-protocol/quartz-app/enclave/
CARGO_TARGET_DIR=./target cargo build --release

echo "--------------------------------------------------------"
echo "building cosmwasm contract binary"


cd $ROOT/cycles-protocol/quartz-app/contracts/cw-tee-mtcs/
bash build.sh
