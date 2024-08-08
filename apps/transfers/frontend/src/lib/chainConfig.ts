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
  chainName: 'Neutron Local Chain',
  stakeCurrency: {
    coinDenom: 'NEUTRON',
    coinMinimalDenom: 'untrn',
    coinDecimals: 6,
    coinGeckoId: 'neutron',
  },
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: 'neutron',
    bech32PrefixAccPub: 'neutron' + 'pub',
    bech32PrefixValAddr: 'neutron' + 'valoper',
    bech32PrefixValPub: 'neutron' + 'valoperpub',
    bech32PrefixConsAddr: 'neutron' + 'valcons',
    bech32PrefixConsPub: 'neutron' + 'valconspub',
  },
  currencies: [
    {
      coinDenom: 'NTRN',
      coinMinimalDenom: 'untrn',
      coinDecimals: 6,
      coinGeckoId: 'neutron',
    },
  ],
  feeCurrencies: [
    {
      coinDenom: 'NTRN',
      coinMinimalDenom: 'untrn',
      coinDecimals: 6,
      coinGeckoId: 'neutron',
      gasPriceStep: { low: 0.001, average: 0.0025, high: 0.004 },
    },
  ],
}
