#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$(pwd)}
DIR_QUARTZ="$ROOT/cycles-quartz"
DIR_QUARTZ_CORE="$DIR_QUARTZ/core/quartz"
DIR_TM_PROVER="$DIR_QUARTZ/utils/tm-prover"

NODE_URL=${NODE_URL:-127.0.0.1:26657}
CMD="wasmd --node http://$NODE_URL"


echo "--------------------------------------------------------"
echo "set trusted hash"

cd "$DIR_TM_PROVER"
CHAIN_STATUS=$($CMD status)
TRUSTED_HASH=$(echo "$CHAIN_STATUS" | jq -r .SyncInfo.latest_block_hash)
TRUSTED_HEIGHT=$(echo "$CHAIN_STATUS" | jq -r .SyncInfo.latest_block_height)
echo "$TRUSTED_HASH" > trusted.hash
echo "$TRUSTED_HEIGHT" > trusted.height

echo "--------------------------------------------------------"
echo "building enclave"
cd "$DIR_QUARTZ_CORE"
gramine-sgx-gen-private-key > /dev/null 2>&1 || :  # may fail
CARGO_TARGET_DIR=./target cargo build --release

gramine-manifest  \
-Dlog_level="error"  \
-Dhome="$HOME"  \
-Darch_libdir="/lib/$(gcc -dumpmachine)"  \
-Dra_type="epid" \
-Dra_client_spid="51CAF5A48B450D624AEFE3286D314894" \
-Dra_client_linkable=1 \
-Dquartz_dir="$(pwd)"  \
-Dtrusted_height="$TRUSTED_HEIGHT"  \
-Dtrusted_hash="$TRUSTED_HASH"  \
quartz.manifest.template quartz.manifest

gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx
gramine-sgx ./quartz
