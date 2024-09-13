'use client'

import { useEffect, useState } from 'react'
import { isEmpty } from 'lodash'
import {
  useAccount,
  useCosmWasmSigningClient,
  useDisconnect,
  useExecuteContract,
  useQuerySmart,
} from 'graz'

import { tw } from '@/lib/tw'
import { wasmEventHandler } from '@/lib/wasmEventHandler'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'
import { StyledText } from '@/components/StyledText'
import { Icon } from '@/components/Icon'
import { DepositModalWindow } from '@/components/DepositModalWindow'
import { TransferModalWindow } from '@/components/TransferModalWindow'
import { WithdrawModalWindow } from '@/components/WithdrawModalWindow'
import {
  clearMnemonic,
  decrypt,
  getEphemeralKeypair,
} from '@/lib/ephemeralKeypair'
import chain from '@/config/chain'
import { useGlobalState } from '@/state/useGlobalState'
import { showError, showSuccess } from '@/lib/notifications'

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
    onError: (error: any) => {
      setLoading(false)
      showError(error.message)
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
    decrypt(encryptedBalance!)
      .then((data) => setBalance(retrieveBalance(data)))
      .catch((error: any) => showError(error.message))
  }, [encryptedBalance])

  // Listen for the response event from the blockchain when requesting the current wallet new balance
  useEffect(() => {
    return wasmEventHandler(
      `execute._contract_address='${process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS}' AND wasm-store_balance.address='${walletAddress}'`,
      {
        next: (event: any) => {
          console.log(event)
          if (!isEmpty(event?.events['wasm-store_balance.encrypted_balance'])) {
            decrypt(
              event.events['wasm-store_balance.encrypted_balance'][0],
            ).then((data) => {
              setLoading(false)
              setBalance(retrieveBalance(data))
              showSuccess('Balance updated successfully')
            })
          }
        },
      },
    )
  }, [walletAddress])

  async function requestBalance() {
    setLoading(true)
    executeContract({
      signingClient: signingClient!,
      msg: contractMessageBuilders.requestBalance(
        (await getEphemeralKeypair()).pubkey,
      ),
    })
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
      <div
        className={tw`
          flex
          flex-col
          gap-2
          divide-y
          rounded-md
          bg-white
          p-5
          py-3
          shadow-md
          outline
          outline-1
          outline-black/5
        `}
      >
        <div className="flex w-full justify-between">
          <span className="font-bold">Balance:</span>
          {!loading ? (
            <span className="font-bold">{formatAmount(balance)}</span>
          ) : (
            <div className="animate-spin">
              <Icon name="spinner" />
            </div>
          )}
        </div>

        <StyledText
          className="justify-start font-bold"
          variant="button.primary"
          as="button"
          disabled={loading}
          onClick={requestBalance}
        >
          <Icon name="building-columns" />
          Get Balance
        </StyledText>

        <div className="my-1 w-full border-black/25"></div>

        <StyledText
          className="w-full justify-start bg-emerald-500"
          as="button"
          variant="button.primary"
          onClick={() => setIsDepositModalOpen(true)}
        >
          <Icon name="piggy-bank" />
          Deposit
        </StyledText>
        <StyledText
          as="button"
          variant="button.primary"
          className="w-full justify-start bg-violet-500"
          onClick={() => setIsTransferModalOpen(true)}
        >
          <Icon name="arrows-left-right" />
          Transfer
        </StyledText>
        <StyledText
          as="button"
          className="w-full justify-start bg-amber-500"
          variant="button.primary"
          onClick={() => setIsWithdrawModalOpen(true)}
        >
          <Icon name="money-bills-simple" />
          Withdraw
        </StyledText>
        <div className="my-1 w-full border-black/25"></div>
        <StyledText
          className="w-full justify-start"
          as="button"
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
