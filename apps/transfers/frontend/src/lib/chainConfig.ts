import { ChainInfo } from '@keplr-wallet/types'
import invariant from 'tiny-invariant'

invariant(
  process.env.NEXT_PUBLIC_CHAIN_ID,
  'NEXT_PUBLIC_CHAIN_ID must be defined',
)
invariant(
  process.env.NEXT_PUBLIC_CHAIN_RPC_URL,
  'NEXT_PUBLIC_CHAIN_RPC_URL must be defined',
)
invariant(
  process.env.NEXT_PUBLIC_CHAIN_REST_URL,
  'NEXT_PUBLIC_CHAIN_REST_URL must be defined',
)

// Testchain definition
export const chain: ChainInfo = {
  rpc: process.env.NEXT_PUBLIC_CHAIN_RPC_URL,
  rest: process.env.NEXT_PUBLIC_CHAIN_REST_URL,
  chainId: process.env.NEXT_PUBLIC_CHAIN_ID,
  chainName: 'My Testing Chain',
  stakeCurrency: {
    coinDenom: 'COSM',
    coinMinimalDenom: 'ucosm',
    coinDecimals: 6,
    coinGeckoId: 'regen',
  },
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: 'wasm',
    bech32PrefixAccPub: 'wasm' + 'pub',
    bech32PrefixValAddr: 'wasm' + 'valoper',
    bech32PrefixValPub: 'wasm' + 'valoperpub',
    bech32PrefixConsAddr: 'wasm' + 'valcons',
    bech32PrefixConsPub: 'wasm' + 'valconspub',
  },
  currencies: [
    {
      coinDenom: 'COSM',
      coinMinimalDenom: 'ucosm',
      coinDecimals: 6,
      coinGeckoId: 'regen',
    },
  ],
  feeCurrencies: [
    {
      coinDenom: 'COSM',
      coinMinimalDenom: 'ucosm',
      coinDecimals: 6,
      coinGeckoId: 'regen',
      gasPriceStep: { low: 0.01, average: 0.025, high: 0.04 },
    },
  ],
}
