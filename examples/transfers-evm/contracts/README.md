# Transfers Solidity
A Solidity project for the Transfers app.

## Prerequisites
Foundry: Install Foundry by running:
```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

Environment Variables: Set up a .env, as per `.env.example`.

## Testing
Run tests on the Sepolia testnet using Foundry. This command forks the Sepolia network, allowing tests to run against real contract data.
This is so we can easily use the deployed attestation contract on sepolia, and not deploy everything ourselves. It does not require Sepolia ETH, as it forks the blockchain state at that block.

```bash
source .env
forge test --fork-url $RPC_URL_SEPOLIA
```

## Project Structure
- src/: Contains the Transfer contract source code.
- script/: Deployment scripts using Foundry’s forge script.
- test/: Test files for Transfers contract functions and features.

## License
This project is licensed under the Apache License 2.0.

## Temporary transfers documentation
Just keeping this here so I can make notes, and add this to the overall docs when I am done

```bash
# get sepolia forked
source .env
anvil --fork-url $RPC_URL_SEPOLIA

# deploy CYC test token
# must updated CYC_ADDRESS and CYC_MINT_ADDRESS (i.e. your test account) in .env after deploying
forge script script/DeployMockERC20.s.sol:DeployMockERC20 --fork-url $SEPOLIA_FORK_URL --private-key $SEPOLIA_PRIV_KEY --broadcast

# deploy transfers app
# must update TRANSFERS_ADDRESS in .env after deploying
forge script script/DeployTransfers.s.sol:DeployTransfers --fork-url $SEPOLIA_FORK_URL --private-key $SEPOLIA_PRIV_KEY --broadcast

# deploy transfers app with mock sgx
# must update TRANSFERS_ADDRESS in .env after deploying
forge script script/DeployTransfersMockSGX.s.sol:DeployTransfersMockSGX --fork-url $SEPOLIA_FORK_URL --private-key $SEPOLIA_PRIV_KEY --broadcast

# call into a contract with forge
cast call 0xD747b295f6F6BC85081fEb484623FE8faAa60aE1 "getAllRequests()" --rpc-url $SEPOLIA_FORK_URL
```