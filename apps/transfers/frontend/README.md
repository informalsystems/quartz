# Transfer App

This is an example frontend that illustrates how to interact with a Quartz app.

This example offers the ability to:

- Deposit amounts into a balance
- Withdraw the whole deposit
- Transfer amounts between wallets in a private-preserving way
- Query your encrypted balance
- Switch between Keplr wallets

## Requirements

In order to get started, you will need:

- [Node.js](https://nodejs.org/) LTS (v20.x)
- `npm`
- A [Keplr](https://www.keplr.app/) Wallet

## Development

Install dependencies:

```bash
npm ci
```

The app requires some environment variables to fully work. Be sure to set up those accordingly to your local environment.

You should start from the template:

```bash
cp .env.example .env.local
```

Required environment variables:

```
# Choose target chain configuration
NEXT_PUBLIC_TARGET_CHAIN=<localWasm | localNeutron | doWasm>
# Enclave public key to encrypt transfers
NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY=<public_key>
# Target transfers contract
NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS=<contract_address>
```

Run the app:

```bash
npm run dev
```

App will be running on http://localhost:3000/ and now everything is up & running 🎉

## E2E Testing

For tests to work, you need to set up the following required environment variables:

```
# Frontend base url
TEST_BASE_URL=<url>
# Keplr browser extension version
TEST_KEPLR_EXTENSION_VERSION=<version>
# Main wallet mnemonic
TEST_WALLET_MNEMONIC=<mnemonic>
# Secondary wallet mnemonic
TEST_SECONDARY_WALLET_MNEMONIC=<mnemonic>
# Secondary wallet address
TEST_SECONDARY_WALLET_ADDRESS=<wallet_address>
# Keplr wallet password. It can be whatever
TEST_WALLET_PASSWORD=<password>
```

Run all E2E tests:

```bash
npm run test
```

If want to run the tests with the Playwright dedicated interface, run:

```bash
npm run test:ui
```
