#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$HOME}


echo "--------------------------------------------------------"
echo "instantiate"
cd  $HOME/cycles-quartz/relayer/
export INSTANTIATE_MSG=$(./scripts/relay.sh Instantiate | jq '{quartz: .} + {denom: "untrn"}' )
echo "--------------------------------------------------------"

echo "deploy contract"
cd $HOME/cycles-quartz/apps/transfers/contracts/

bash deploy-contract-Neutrond.sh target/wasm32-unknown-unknown/release/transfers_contract.wasm  |& tee output
export CONTRACT=$(cat output | grep Address | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')
echo $CONTRACT 




