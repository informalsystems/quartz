#!/bin/bash

#set -eo pipefail

ROOT=${ROOT:-$HOME}
DIR_QUARTZ="$ROOT/cycles-quartz"
DIR_QUARTZ_APP="$DIR_QUARTZ/apps/transfers"
DIR_QUARTZ_ENCLAVE="$DIR_QUARTZ_APP/enclave"
DIR_QUARTZ_TM_PROVER="$DIR_QUARTZ/utils/tm-prover"

NODE_URL=${NODE_URL:-127.0.0.1:26657}
CMD="wasmd --node http://$NODE_URL"


echo "--------------------------------------------------------"
echo "GRAMINE_PORT is set to: $GRAMINE_PORT"
echo "set trusted hash"

cd "$DIR_QUARTZ_TM_PROVER"
cargo run -- --chain-id testing \
--primary "http://$NODE_URL" \
--witnesses "http://$NODE_URL" \
--trusted-height 500000 \
--trusted-hash "5237772462A41C0296ED688A0327B8A60DF310F08997AD760EB74A70D0176C27" \
--contract-address "wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s0phg4d" \
--storage-key "quartz_session" \
--trace-file light-client-proof.json &> $DIR_QUARTZ_APP/output

cd $DIR_QUARTZ_APP
cat output | grep found | head -1 | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g' > trusted.hash
export TRUSTED_HASH=$(cat trusted.hash)
echo "... $TRUSTED_HASH"
rm output

echo "--------------------------------------------------------"
echo "configure gramine"
cd "$DIR_QUARTZ_ENCLAVE"

echo "... gen priv key if it doesnt exist"
gramine-sgx-gen-private-key > /dev/null 2>&1 || :  # may fail

echo "... update manifest template with trusted hash $TRUSTED_HASH"
sed -i -r "s/(\"--trusted-hash\", \")[A-Z0-9]+(\"])/\1$TRUSTED_HASH\2/" quartz.manifest.template


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
-Dgramine_port="$GRAMINE_PORT" \
quartz.manifest.template quartz.manifest

echo "... sign manifest"
gramine-sgx-sign --manifest quartz.manifest --output quartz.manifest.sgx


echo "--------------------------------------------------------"
echo "... start gramine"
gramine-sgx ./quartz
