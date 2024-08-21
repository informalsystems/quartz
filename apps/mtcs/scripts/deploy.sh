#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$(git rev-parse --show-toplevel)}


echo "--------------------------------------------------------"
echo "instantiate"
cd  $ROOT/relayer/
export INSTANTIATE_MSG=$(./scripts/relay.sh Instantiate | jq '{quartz: .}' )
echo "--------------------------------------------------------"
echo "deploy contract"
cd $ROOT/apps/mtcs/contracts/cw-tee-mtcs

bash deploy-contract.sh target/wasm32-unknown-unknown/release/cw_tee_mtcs.wasm  |& tee output
export CONTRACT=$(cat output | grep Address | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')
echo $CONTRACT 




