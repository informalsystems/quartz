#!/bin/bash

set -e

# Initialize neutrond
neutrond init testnet --chain-id test-1

# Create accounts
echo "y" | neutrond keys add val1 --keyring-backend test
echo "y" | neutrond keys add val2 --keyring-backend test
echo "y" | neutrond keys add user1 --keyring-backend test
echo "y" | neutrond keys add user2 --keyring-backend test

# Add genesis accounts
neutrond add-genesis-account $(neutrond keys show val1 -a --keyring-backend test) 1000000000untrn,1000000000stake
neutrond add-genesis-account $(neutrond keys show val2 -a --keyring-backend test) 1000000000untrn,1000000000stake
neutrond add-genesis-account $(neutrond keys show user1 -a --keyring-backend test) 1000000000untrn,1000000000stake
neutrond add-genesis-account $(neutrond keys show user2 -a --keyring-backend test) 1000000000untrn,1000000000stake

# Create validator transactions
neutrond gentx val1 500000000untrn --chain-id test-1 --keyring-backend test
neutrond gentx val2 500000000untrn --chain-id test-1 --keyring-backend test

# Collect genesis transactions
neutrond collect-gentxs

# Validate genesis file
neutrond validate-genesis

# Start the node
neutrond start --minimum-gas-prices 0.0053untrn
# #!/bin/bash

# set -e

# # Initialize neutrond
# neutrond init testnet --chain-id test-1

# # Create accounts
# neutrond keys add val1 --keyring-backend test
# neutrond keys add val2 --keyring-backend test
# neutrond keys add user1 --keyring-backend test
# neutrond keys add user2 --keyring-backend test

# # Add genesis accounts
# neutrond add-genesis-account $(neutrond keys show val1 -a --keyring-backend test) 1000000000untrn
# neutrond add-genesis-account $(neutrond keys show val2 -a --keyring-backend test) 1000000000untrn
# neutrond add-genesis-account $(neutrond keys show user1 -a --keyring-backend test) 1000000000untrn
# neutrond add-genesis-account $(neutrond keys show user2 -a --keyring-backend test) 1000000000untrn

# # Create validator transactions
# neutrond gentx val1 500000000untrn --chain-id test-1 --keyring-backend test
# neutrond gentx val2 500000000untrn --chain-id test-1 --keyring-backend test

# # Collect genesis transactions
# neutrond collect-gentxs

# # Validate genesis file
# neutrond validate-genesis