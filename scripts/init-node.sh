#!/bin/bash

# Initialize a wasmd node that can host the MVP CosmWasm smart contract.
# Also creates a validator account and adds default genesis accounts with sufficient tokens for testing (stake and fees)

set -euo pipefail

ADMIN=${ADMIN:-$(wasmd keys show -a admin)}
ALICE=${ALICE:-$(wasmd keys show -a alice)}
BOB=${BOB:-$(wasmd keys show -a bob)}
CHARLIE=${CHARLIE:-$(wasmd keys show -a charlie)}

echo "Remove old docker volume (if it exists)..."
docker volume rm -f wasmd_data


echo ""
echo "Setup wasmd (with validator and default genesis accounts)..."
docker run --rm -it \
  --mount type=volume,source=wasmd_data,target=/root \
  --name wasmd \
  cosmwasm/wasmd:v0.44.0  \
  /bin/sh -c "sed -i  's/1000000000/12000000000000/g' /opt/setup_wasmd.sh;
  /opt/setup_wasmd.sh "$ADMIN" "$ALICE" "$BOB" "$CHARLIE";" \


