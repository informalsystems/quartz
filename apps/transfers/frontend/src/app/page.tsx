'use client'

import { useEffect, useState } from 'react'
import { isEmpty } from 'lodash'

import {
  DepositModalWindow,
  Icon,
  Notifications,
  StyledText,
  TransferModalWindow,
  WithdrawModalWindow,
} from '@/components'
import { tw } from '@/lib/tw'
import { wasmEventHandler } from '@/lib/wasmEventHandler'
import { FormActionResponse } from '@/lib/types'
import { chain } from '@/lib/chainConfig'
import { cosm } from '@/lib/cosm'
import { wallet } from '@/lib/wallet'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'

function formatAmount(value: number) {
  return value.toLocaleString('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
    maximumFractionDigits: chain.stakeCurrency?.coinDecimals,
  })
}

// Safe method to get the balance amount from the decrypted data
const retrieveBalance = (data: string) => {
  let balance = 0

  if (!isEmpty(data)) {
    const json = JSON.parse(data)

    balance = Number(json.balance ?? 0)
  }

  return balance
}

export default function Home() {
  const [requestBalanceResult, setRequestBalanceResult] =
    useState<FormActionResponse>()
  const [balance, setBalance] = useState(0)
  const [isLoading, setIsLoading] = useState(false)
  const [walletAddress, setWalletAddress] = useState(
    wallet.getAccount().address,
  )
  const [isDepositModalOpen, setIsDepositModalOpen] = useState(false)
  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false)
  const [isWithdrawModalOpen, setIsWithdrawModalOpen] = useState(false)

  // Listen for Keplr wallet switches so we can refresh the Keplr & CosmWasm info
  useEffect(() => {
    const params: [string, () => void] = [
      'keplr_keystorechange',
      () => {
        setIsLoading(true)

        wallet
          .refreshUser()
          .then(cosm.init)
          .finally(() => {
            setWalletAddress(wallet.getAccount().address)
            setIsLoading(false)
          })
      },
    ]

    window.addEventListener(...params)

    return () => window.removeEventListener(...params)
  }, [])

  // Set the current balance for the wallet. Whenever the wallet changes, we retrieve its balance
  useEffect(() => {
    cosm
      .queryTransferContract({
        messageBuilder: () => contractMessageBuilders.getBalance(walletAddress),
      })
      .then((data) => setBalance(retrieveBalance(wallet.decrypt(data))))
  }, [walletAddress])

  // Listen for the response event from the blockchain when requesting the current wallet new balance
  useEffect(() => {
    return wasmEventHandler(
      `execute._contract_address='${process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS}' AND wasm-store_balance.address='${walletAddress}'`,
      // `execute._contract_address='${process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS}' `,
      {
        next: (event) => {
          console.log(event)
          if (!isEmpty(event?.events['wasm-store_balance.encrypted_balance'])) {
            setBalance(
              retrieveBalance(
                wallet.decrypt(
                  event.events['wasm-store_balance.encrypted_balance'][0],
                ),
              ),
            )
          }
        },
      },
    )
  }, [walletAddress])

  // Request the current wallet new balance calling the transfer contract
  async function requestBalance(): Promise<void> {
    let result

    try {
      setIsLoading(true)
      const response = await cosm.executeTransferContract({
        messageBuilder: () =>
          contractMessageBuilders.requestBalance(wallet.getKeypair().pubkey),
      })
      console.log(response)
      setIsLoading(false)

      result = {
        success: true,
        messages: ['woo!'],
      }
    } catch (error) {
      console.error(error)
      setIsLoading(false)
      result = {
        success: false,
        messages: ['Something went wrong'],
      }
    } finally {
      setRequestBalanceResult(result)
    }
  }

  return (
    <main
      className={tw`
        flex
        h-screen
        flex-col
        items-center
        justify-center
        bg-[url(/images/moroccan-flower.png)]
        p-12
      `}
    >
      <Notifications formActionResponse={requestBalanceResult} />
      <div
        className={tw`
          flex
          flex-col
          gap-2
          divide-y
          rounded-md
          border
          border-black/20
          bg-white
          p-5
          py-3
          shadow-2xl
        `}
      >
        <div className="flex w-full justify-between">
          <span className="font-bold">Balance:</span>
          <span className="font-bold">{formatAmount(balance)}</span>
        </div>

        <StyledText
          className="bg-blue-500 font-bold"
          variant="button.primary"
          as="button"
          disabled={isLoading}
          onClick={requestBalance}
        >
          {!isLoading ? (
            <Icon name="building-columns" />
          ) : (
            <div className="animate-spin">
              <Icon name="spinner" />
            </div>
          )}
          Get Balance
        </StyledText>

        <div className="my-1 w-full border-black/20"></div>

        <StyledText
          className="w-full bg-emerald-500 font-bold"
          variant="button.primary"
          onClick={() => setIsDepositModalOpen(true)}
        >
          <Icon name="piggy-bank" />
          Deposit
        </StyledText>
        <StyledText
          variant="button.primary"
          className="w-full font-bold"
          onClick={() => setIsTransferModalOpen(true)}
        >
          <Icon name="arrows-left-right" />
          Transfer
        </StyledText>
        <StyledText
          className="w-full bg-amber-500 font-bold"
          variant="button.primary"
          onClick={() => setIsWithdrawModalOpen(true)}
        >
          <Icon name="money-bills-simple" />
          Withdraw
        </StyledText>
      </div>

      <DepositModalWindow
        isOpen={isDepositModalOpen}
        onClose={() => setIsDepositModalOpen(false)}
      />
      <TransferModalWindow
        isOpen={isTransferModalOpen}
        onClose={() => setIsTransferModalOpen(false)}
      />
      <WithdrawModalWindow
        isOpen={isWithdrawModalOpen}
        onClose={() => setIsWithdrawModalOpen(false)}
      />
    </main>
  )
}
