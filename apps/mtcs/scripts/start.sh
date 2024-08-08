#!/bin/bash

#set -eo pipefail

DIR_QUARTZ=${ROOT:-$(git rev-parse --show-toplevel)}
DIR_QUARTZ_APP="$DIR_QUARTZ/apps/mtcs"
DIR_QUARTZ_ENCLAVE="$DIR_QUARTZ_APP/enclave"
DIR_QUARTZ_TM_PROVER="$DIR_QUARTZ/utils/tm-prover"

NODE_URL=${NODE_URL:-143.244.186.205:26657}
CMD="wasmd --node http://$NODE_URL"

echo "--------------------------------------------------------"
echo "set trusted hash"

cd "$DIR_QUARTZ_TM_PROVER"
# cargo run -- --chain-id testing \
# --primary "http://$NODE_URL" \
# --witnesses "http://$NODE_URL" \
# --trusted-height 1 \
# --trusted-hash "5237772462A41C0296ED688A0327B8A60DF310F08997AD760EB74A70D0176C27" \
# --contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
# --storage-key "quartz_session" \
# --trace-file light-client-proof.json &> $DIR_QUARTZ_APP/output

# # Debug output of cargo run
# echo "Cargo run output:"
# cat $DIR_QUARTZ_APP/output

# cd $DIR_QUARTZ_APP
# # Debug hash extraction
# echo "Attempting to extract trusted hash from output..."
# cat output | grep found | head -1 | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g' > trusted.hash

# # Check if the hash was extracted correctly
# if [[ ! -s trusted.hash ]]; then
#     echo "Failed to extract trusted hash from output"
#     exit 1
# fi

# export TRUSTED_HASH=$(cat trusted.hash)
# echo "Extracted TRUSTED_HASH: $TRUSTED_HASH"
# rm output
CHAIN_STATUS=$($CMD status)
TRUSTED_HASH=$(echo "$CHAIN_STATUS" | jq -r .SyncInfo.latest_block_hash)
TRUSTED_HEIGHT=$(echo "$CHAIN_STATUS" | jq -r .SyncInfo.latest_block_height)
echo "... $TRUSTED_HASH"


cd "$DIR_QUARTZ_APP"
echo "$TRUSTED_HASH" > trusted.hash
echo "$TRUSTED_HEIGHT" > trusted.height

if [ -n "$MOCK_SGX" ]; then
    echo "MOCK_SGX is set. Running enclave without gramine..."
    cd $DIR_QUARTZ

    RUST_BACKTRACE=full ./target/release/mtcs-enclave --chain-id "testing" --trusted-height "$TRUSTED_HEIGHT" --trusted-hash "$TRUSTED_HASH"
    exit
fi

echo "--------------------------------------------------------"
echo "configure gramine"
cd "$DIR_QUARTZ_ENCLAVE"

echo "... gen priv key if it doesnt exist"
gramine-sgx-gen-private-key > /dev/null 2>&1 || :  # may fail

# echo "... update manifest template with trusted hash $TRUSTED_HASH"
# sed -i -r "s/(\"--trusted-hash\", \")[A-Z0-9]+(\"])/\1$TRUSTED_HASH\2/" quartz.manifest.template

echo "... create manifest"
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

if [ $? -ne 0 ]; then
    echo "gramine-manifest failed"
    exit 1
fi

echo "... sign manifest"
gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx

if [ $? -ne 0 ]; then
    echo "gramine-sgx-sign failed"
    exit 1
fi

echo "--------------------------------------------------------"
echo "... start gramine"
gramine-sgx ./quartz

if [ $? -ne 0 ]; then
    echo "gramine-sgx failed to start"
    exit 1
fi