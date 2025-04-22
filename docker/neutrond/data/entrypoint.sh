#!/bin/sh
set -e

# Clean previous keys & reset node
rm -rf /root/.neutrond/keyring-test &> /dev/null
neutrond tendermint unsafe-reset-all

# Init configuration files
neutrond init test --chain-id testing --overwrite

# Modify default configurations
sed -i 's/keyring-backend = "os"/keyring-backend = "test"/g' /root/.neutrond/config/client.toml
sed -i 's/cors_allowed_origins = \[\]/cors_allowed_origins = \[\"*\"\]/g' /root/.neutrond/config/config.toml
sed -i 's/minimum-gas-prices = ""/minimum-gas-prices = "0.0025untrn"/g' /root/.neutrond/config/app.toml
sed -i 's/enabled-unsafe-cors = false/enabled-unsafe-cors = true/g' /root/.neutrond/config/app.toml
sed -i 's/enable = false/enable = true/g' /root/.neutrond/config/app.toml
sed -i 's/swagger = false/swagger = true/g' /root/.neutrond/config/app.toml

# Changing pruining and empty blocks so our node doesn't grow and cause us to redeploy too often"
sed -i 's/pruning = "default"/pruning = "everything"/g' /root/.neutrond/config/app.toml
sed -i 's/create_empty_blocks = true/create_empty_blocks = false/g' /root/.neutrond/config/config.toml

# feemarket parameters in genesis.json
sed -i 's/"min_base_gas_price": "[^"]*"/"min_base_gas_price": "0.0025"/g' /root/.neutrond/config/genesis.json
sed -i 's/"fee_denom": "[^"]*"/"fee_denom": "untrn"/g' /root/.neutrond/config/genesis.json
sed -i 's/"base_gas_price": "[^"]*"/"base_gas_price": "0.0025"/g' /root/.neutrond/config/genesis.json

# Import all test accounts
for filename in /root/accounts/*.txt; do
  tail -n 1 "$filename" | neutrond keys add "$(basename "$filename" .txt)" --no-backup --recover --keyring-backend=test

  neutrond add-genesis-account "$(neutrond keys show $(basename "$filename" .txt) -a)" 12000000000000untrn
done

# Enable as a single node consumer
neutrond add-consumer-section

# Start node
neutrond start --trace --rpc.laddr="tcp://0.0.0.0:26657" --api.address="tcp://0.0.0.0:1317" --grpc.address="0.0.0.0:9090"
