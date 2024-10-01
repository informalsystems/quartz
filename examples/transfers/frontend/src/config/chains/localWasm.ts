import { ChainInfo } from '@keplr-wallet/types'

// Wasm local chain definition
export const localWasm: ChainInfo = {
  chainId: 'testing',
  chainName: 'Local Wasm Testchain',
  rpc: 'http://localhost:26657',
  rest: 'http://localhost:1317',
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
