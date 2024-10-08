#!/bin/sh
set e

# Clean previous keys & reset node
rm -rf /root/.neutrond/keyring-test &> /dev/null
neutrond tendermint unsafe-reset-all

# Init configuration files
neutrond init test --chain-id testing --overwrite

# Modify default configurations
sed -i 's/keyring-backend = "os"/keyring-backend = "test"/g' /root/.neutrond/config/client.toml
sed -i 's/minimum-gas-prices = ""/minimum-gas-prices = "0.25stake,0.0001untrn"/g' /root/.neutrond/config/app.toml
sed -i 's/enabled-unsafe-cors = false/enabled-unsafe-cors = true/g' /root/.neutrond/config/app.toml
sed -i 's/enable = false/enable = true/g' /root/.neutrond/config/app.toml
sed -i 's/swagger = false/swagger = true/g' /root/.neutrond/config/app.toml
# sed -i -e 's/oracle_address = "localhost:8080"/oracle_address = "localhost:8080"/g' /root/.neutrond/config/app.toml

# Import all test accounts
for filename in /root/accounts/*.txt; do
  tail -n 1 "$filename" | neutrond keys add "$(basename "$filename" .txt)" --no-backup --recover --keyring-backend=test

  neutrond add-genesis-account "$(neutrond keys show $(basename "$filename" .txt) -a)" 12000000000000stake,12000000000000untrn
done

# Enable as a single node consumer
neutrond add-consumer-section

# Start node
neutrond start --trace
