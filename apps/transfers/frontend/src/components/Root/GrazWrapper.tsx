import { GrazProvider, WalletType } from 'graz'
import { ChainInfo } from '@keplr-wallet/types'

export default function GrazWrapper({
  chains,
  children,
}: {
  chains: ChainInfo[]
  children: React.ReactNode
}) {
  return (
    <GrazProvider
      grazOptions={{
        chains,
        defaultWallet: WalletType.KEPLR,
        chainsConfig: chains.reduce(
          (acc, curr) => ({
            ...acc,
            [curr.chainId]: {
              gas: { denom: curr.feeCurrencies[0].coinDenom, price: '0' },
            },
          }),
          {},
        ),
      }}
    >
      {children}
    </GrazProvider>
  )
}
