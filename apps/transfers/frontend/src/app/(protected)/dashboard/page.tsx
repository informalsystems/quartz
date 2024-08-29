'use client'

import { useEffect, useState } from 'react'
import { isEmpty } from 'lodash'

import { tw } from '@/lib/tw'
import { wasmEventHandler } from '@/lib/wasmEventHandler'
import { FormActionResponse } from '@/lib/types'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'
import { Notifications } from '@/components/Notifications'
import { StyledText } from '@/components/StyledText'
import { Icon } from '@/components/Icon'
import { DepositModalWindow } from '@/components/DepositModalWindow'
import { TransferModalWindow } from '@/components/TransferModalWindow'
import { WithdrawModalWindow } from '@/components/WithdrawModalWindow'
import {
  useAccount,
  useCosmWasmSigningClient,
  useDisconnect,
  useExecuteContract,
  useQuerySmart,
} from 'graz'
import {
  clearMnemonic,
  decrypt,
  getEphemeralKeypair,
} from '@/lib/ephemeralKeypair'
import chain from '@/config/chain'
import { useGlobalState } from '@/state/useGlobalState'

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

export default function Dashboard() {
  const [requestBalanceResult, setRequestBalanceResult] =
    useState<FormActionResponse>()
  const [balance, setBalance] = useState(0)
  const [loading, setLoading] = useState(false)
  const { data } = useAccount()
  const [isDepositModalOpen, setIsDepositModalOpen] = useState(false)
  const [isTransferModalOpen, setIsTransferModalOpen] = useState(false)
  const [isWithdrawModalOpen, setIsWithdrawModalOpen] = useState(false)
  const { disconnect } = useDisconnect({
    onSuccess: clearMnemonic,
  })
  const { data: signingClient } = useCosmWasmSigningClient()
  const { executeContract } = useExecuteContract({
    contractAddress: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS!,
    onSuccess: (data) => {
      console.log(data)
      setLoading(false)
    },
    onLoading: () => setLoading(true),
    onError: (err: any) => {
      setLoading(false)
    },
  })
  const walletAddress = data?.bech32Address ?? ''

  const { data: encryptedBalance } = useQuerySmart<string, any>(
    data && {
      address: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS,
      queryMsg: contractMessageBuilders.getBalance(walletAddress),
    },
  )
  // Set the current balance for the wallet. Whenever the wallet changes, we retrieve its balance
  useEffect(() => {
    decrypt(encryptedBalance!).then((data) => setBalance(retrieveBalance(data)))
  }, [encryptedBalance])

  // Listen for the response event from the blockchain when requesting the current wallet new balance
  useEffect(() => {
    return wasmEventHandler(
      `execute._contract_address='${process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS}' AND wasm-store_balance.address='${walletAddress}'`,
      {
        next: (event) => {
          console.log(event)
          if (!isEmpty(event?.events['wasm-store_balance.encrypted_balance'])) {
            decrypt(
              event.events['wasm-store_balance.encrypted_balance'][0],
            ).then((data) => setBalance(retrieveBalance(data)))
          }
        },
      },
    )
  }, [walletAddress])

  async function requestBalance(): Promise<void> {
    let result

    try {
      setLoading(true)
      executeContract({
        signingClient,
        msg: contractMessageBuilders.requestBalance(
          (await getEphemeralKeypair()).pubkey,
        ),
      })
      setLoading(false)

      result = {
        success: true,
        messages: ['woo!'],
      }
    } catch (error) {
      console.error(error)
      setLoading(false)
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
          className="font-bold"
          variant="button.primary"
          as="button"
          disabled={loading}
          onClick={requestBalance}
        >
          {!loading ? (
            <Icon name="building-columns" />
          ) : (
            <div className="animate-spin">
              <Icon name="spinner" />
            </div>
          )}
          Get Balance
        </StyledText>

        <div className="my-1 w-full border-black/25"></div>

        <StyledText
          className="w-full bg-emerald-500"
          variant="button.primary"
          onClick={() => setIsDepositModalOpen(true)}
        >
          <Icon name="piggy-bank" />
          Deposit
        </StyledText>
        <StyledText
          variant="button.primary"
          className="w-full bg-violet-500"
          onClick={() => setIsTransferModalOpen(true)}
        >
          <Icon name="arrows-left-right" />
          Transfer
        </StyledText>
        <StyledText
          className="w-full bg-amber-500"
          variant="button.primary"
          onClick={() => setIsWithdrawModalOpen(true)}
        >
          <Icon name="money-bills-simple" />
          Withdraw
        </StyledText>
        <div className="my-1 w-full border-black/25"></div>
        <StyledText
          className="w-full"
          variant="button.secondary"
          onClick={() => {
            const res = confirm(
              'Disconnecting your account will remove your mnemonic and private key and you will lose access to your data unless you have them backed up. Are you sure you want to continue?',
            )

            if (!res) {
              return
            }

            useGlobalState.getState().setLoading(true)
            disconnect()
          }}
        >
          <Icon name="door-open" />
          Disconnect
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
