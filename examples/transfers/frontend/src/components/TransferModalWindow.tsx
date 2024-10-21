'use client'

import { PublicKey, encrypt } from 'eciesjs'
import { ChangeEvent, useState } from 'react'
import { useAccount, useCosmWasmSigningClient, useExecuteContract } from 'graz'
import { isEmpty } from 'lodash'

import { LoadingSpinner } from '@/components/LoadingSpinner'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { StyledBox } from '@/components/StyledBox'
import { StyledText } from '@/components/StyledText'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'
import chain from '@/config/chain'
import { tw } from '@/lib/tw'
import { showError, showSuccess } from '@/lib/notifications'
import { isValidAddress } from '@/lib/isValidAddress'
import { Icon } from './Icon'

// Encrypt the transfer data using the enclave public key
function encryptMsg(data: {
  sender: string
  receiver: string
  amount: string
}): string {
  // Create the public key from the hex
  const pubkey = PublicKey.fromHex(process.env.NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY)
  // Convert the data into a JSON string
  const serializedState = JSON.stringify(data)
  // Encrypt the data
  const encryptedState = encrypt(
    pubkey.toHex(),
    Buffer.from(serializedState, 'utf-8'),
  )

  return encryptedState.toString('hex')
}

export function TransferModalWindow(props: ModalWindowProps) {
  const [amount, setAmount] = useState(0)
  const [receiver, setRecipient] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const { data: wallet } = useAccount()
  const { data: signingClient } = useCosmWasmSigningClient()
  const { executeContract } = useExecuteContract({
    contractAddress: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS!,
    onSuccess: (data) => {
      console.log(data)
      setLoading(false)
      showSuccess('Transfer transaction sent successfully')
      setTimeout(() => props.onClose(), 2000) // Close after 2 seconds
    },
    onError: (error: any) => {
      setLoading(false)
      showError(error.message)
    },
  })

  // Transfer an amount between wallets calling the Transfer contract with an encrypted request
  function handleTransfer() {
    setError('')

    if (!isValidAddress(receiver)) {
      setError('Invalid recipient address format.')

      return
    }

    if (amount <= 0) {
      setError('Amount should be greater than zero.')

      return
    }

    setLoading(true)

    const encryptedMsg = encryptMsg({
      sender: wallet?.bech32Address!,
      receiver: String(receiver),
      amount: String(amount),
    })

    executeContract({
      signingClient: signingClient!,
      msg: contractMessageBuilders.transfer(encryptedMsg),
      funds: [
        {
          denom: chain.currencies[0].coinMinimalDenom,
          amount: String(amount),
        },
      ],
    })
  }

  return (
    <ModalWindow
      disableClosing={loading}
      {...props}
    >
      <LoadingSpinner isLoading={loading} />

      <ModalWindow.Title className="bg-violet-500">
        <Icon name="arrows-left-right" /> Transfer
      </ModalWindow.Title>

      <ModalWindow.Body className="space-y-3">
        {!isEmpty(error) && (
          <div className="font-bold text-red-500">{error}</div>
        )}

        <StyledBox
          as="input"
          className={tw`
              focus:!border-violet-500
              focus:!outline-violet-500
              focus:!ring-violet-500
            `}
          placeholder="recipient address"
          type="text"
          variant="input"
          name="receiver"
          value={receiver}
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            setRecipient(event.target.value)
          }
        />

        <StyledBox
          as="input"
          className={tw`
              focus:!border-violet-500
              focus:!outline-violet-500
              focus:!ring-violet-500
            `}
          min={0}
          name="amount"
          placeholder="0.00"
          type="number"
          value={amount || ''}
          variant="input"
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            setAmount(Number(event.target.value))
          }
        />
      </ModalWindow.Body>

      <ModalWindow.Buttons>
        <StyledText
          as="button"
          className="bg-violet-500"
          disabled={amount === 0}
          variant="button.primary"
          onClick={handleTransfer}
        >
          Transfer
        </StyledText>
        <StyledText
          as="button"
          variant="button.secondary"
          onClick={props.onClose}
        >
          Cancel
        </StyledText>
      </ModalWindow.Buttons>
    </ModalWindow>
  )
}
