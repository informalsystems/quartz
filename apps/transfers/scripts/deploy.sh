#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$HOME}


echo "--------------------------------------------------------"
echo "instantiate"
cd  $ROOT/cycles-quartz/relayer/
export INSTANTIATE_MSG=$(./scripts/relay.sh Instantiate | jq '{quartz: .} + {denom: "ucosm"}' )
echo $INSTANTIATE_MSG
echo "--------------------------------------------------------"

echo "deploy contract"
cd $ROOT/cycles-quartz/apps/transfers/contracts/

bash deploy-contract.sh target/wasm32-unknown-unknown/release/transfers_contract.wasm  2>&1 | tee output
export CONTRACT=$(cat output | grep Address | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')
echo $CONTRACT 




