# Transfer App

This is an example frontend that illustrates how to interact with a Transfer Quartz App.

This example offers:

- Deposit amounts into a balance
- Withdraw the whole deposit
- Transfer amounts between wallet addresses in a private-preserving way
- Query your encrypted balance to capture changes
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

The App requires some environment variables to fully work. Be sure to set up those accordingly to your local environment.

You should start from the template:

```bash
cp .env.example .env.local
```

Run the app:

```bash
npm run dev
```

And now everything is up & running 🎉
