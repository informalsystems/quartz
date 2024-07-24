#!/bin/bash

set -eo pipefail

ROOT=${ROOT:-$HOME}


echo "--------------------------------------------------------"
echo "instantiate"
cd  $ROOT/Dev/cycles-quartz/relayer/
# export INSTANTIATE_MSG=$(./scripts/relayNeutron.sh Instantiate | jq '{msg: .} + {denom: "untrn"}' )

# echo $INSTANTIATE_MSG
# echo "--------------------------------------------------------"

# # Escape the JSON for passing as a shell argument
# ESCAPED_INSTANTIATE_MSG=$(echo "$INSTANTIATE_MSG" | jq -c -R '@json')
# echo "Escaped INSTANTIATE_MSG:"
# echo "$ESCAPED_INSTANTIATE_MSG"
# echo "--------------------------------------------------------"
INSTANTIATE_MSG=$(./scripts/relayNeutron.sh Instantiate | jq -c '.')
echo "Raw INSTANTIATE_MSG:"
echo "$INSTANTIATE_MSG" | jq '.'
echo "--------------------------------------------------------"

echo "deploy contract"
cd $HOME/cycles-quartz/apps/transfers/contracts/

bash deploy-contract-Neutrond.sh target/wasm32-unknown-unknown/release/transfers_contract.wasm  "$INSTANTIATE_MSG" | tee output
export CONTRACT=$(cat output | grep Address | awk '{print $NF}' | sed 's/\x1b\[[0-9;]*m//g')
echo $CONTRACT 