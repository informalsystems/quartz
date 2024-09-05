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

Run all E2E tests:

```bash
npm run test
```

If want to run the tests with the Playwright dedicated interface, run:

```bash
npm run test:ui
```
