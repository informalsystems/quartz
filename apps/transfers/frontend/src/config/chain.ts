import { ChainInfo } from '@keplr-wallet/types'

const baseTestConfig = {
  chainId: 'testing',
  bech32Config: {
    bech32PrefixAccAddr: 'wasm',
    bech32PrefixAccPub: 'wasmpub',
    bech32PrefixValAddr: 'wasmvaloper',
    bech32PrefixValPub: 'wasmvaloperpub',
    bech32PrefixConsAddr: 'wasmvalcons',
    bech32PrefixConsPub: 'wasmvalconspub',
  },
  currencies: [
    {
      coinDenom: 'COSM',
      coinMinimalDenom: 'ucosm',
      coinDecimals: 6,
    },
    {
      coinDenom: 'ATOM',
      coinMinimalDenom: 'uatom',
      coinDecimals: 6,
    },
  ],
  feeCurrencies: [
    {
      coinDenom: 'COSM',
      coinMinimalDenom: 'ucosm',
      coinDecimals: 6,
    },
  ],
  stakeCurrency: {
    coinDenom: 'ATOM',
    coinMinimalDenom: 'uatom',
    coinDecimals: 6,
  },
  bip44: { coinType: 118 },
}

const supportedChains: Record<string, ChainInfo> = {
  local: {
    ...baseTestConfig,
    chainName: 'Local',
    rpc: 'http://localhost:26657',
    rest: 'http://localhost:1317',
  },
  do: {
    ...baseTestConfig,
    chainName: 'DO Testnet',
    rpc: 'http://143.244.186.205:26657',
    rest: 'http://143.244.186.205:1317',
  },
}

const chain = supportedChains[process.env.NEXT_PUBLIC_TARGET_CHAIN!]

export default chain
