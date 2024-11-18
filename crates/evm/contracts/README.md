# Quartz Solidity
A Solidity project for the Quartz contract, utilizing Foundry for testing and deployment. This contract integrates with the DCAP attestation system on the Sepolia testnet, providing secure attestation and session management.

Quartz.sol provides the attestation message handling for a solidity app. You must inherit the
`Quartz` contract into your contracts app. This can be seen under `examples/transfers/transfers-evm/contracts`.

## Prerequisites
Foundry: Install Foundry by running:
```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

Environment Variables: Set up a .env file in the project root with your private key and Sepolia RPC URL (alchemy or any other provider):
```
SEPOLIA_PRIV_KEY=your_private_key_here
RPC_URL_SEPOLIA=https://eth-sepolia.alchemyapi.io/v2/YOUR_ALCHEMY_API_KEY
```

## Testing
Run tests on the Sepolia testnet using Foundry. This command forks the Sepolia network, allowing tests to run against real contract data.
This is so we can easily use the deployed attestation contract on sepolia, and not deploy everything ourselves. It does not require Sepolia ETH, as it forks the blockchain state at that block.

```bash
source .env
forge test --fork-url $RPC_URL_SEPOLIA
```

## Project Structure
- src/: Contains the Quartz contract source code.
- test/: Test files for Quartz contract functions and features.

## License
This project is licensed under the Apache License 2.0.
