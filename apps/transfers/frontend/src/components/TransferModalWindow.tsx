'use client'
import { PublicKey, encrypt } from 'eciesjs'
import { ChangeEvent, useActionState, useState } from 'react'
import invariant from 'tiny-invariant'

import { LoadingSpinner } from '@/components/LoadingSpinner'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { Notifications } from '@/components/Notifications'
import { StyledBox } from '@/components/StyledBox'
import { StyledText } from '@/components/StyledText'
import { cosm } from '@/lib/cosm'
import { FormActionResponse } from '@/lib/types'
import { wallet } from '@/lib/wallet'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'

// Encrypt the transfer data using the enclave public key
function encryptMsg(data: {
  sender: string
  receiver: string
  amount: string
}): string {
  invariant(
    process.env.NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY,
    'NEXT_PUBLIC_ENCLAVE_PUBLIC_KEY must be defined',
  )

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

// Transfer an amount between wallets calling the Transfer contract with an encrypted request
async function handleTransfer(
  _: FormActionResponse,
  formData: FormData,
): Promise<FormActionResponse> {
  const receiver = String(formData.get('receiver'))
  const amount = String(formData.get('amount'))

  try {
    const encryptedMsg = encryptMsg({
      sender: wallet.getAccount().address,
      receiver,
      amount,
    })

    const result = await cosm.executeTransferContract({
      messageBuilder: () => contractMessageBuilders.transfer(encryptedMsg),
    })
    console.log(result)

    return {
      success: true,
      messages: ['woo!'],
    }
  } catch (error) {
    console.error(error)

    return {
      success: false,
      messages: ['Something went wrong'],
    }
  }
}

export function TransferModalWindow({
  isOpen,
  onClose,
  ...otherProps
}: ModalWindowProps) {
  const [amount, setAmount] = useState(0)
  const [receiver, setRecipient] = useState('')
  const [formActionResponse, formAction, isLoading] = useActionState(
    handleTransfer,
    null,
  )

  return (
    <ModalWindow
      isOpen={isOpen}
      onClose={onClose}
      {...otherProps}
    >
      <LoadingSpinner isLoading={isLoading} />

      <ModalWindow.Title>Transfer</ModalWindow.Title>

      <form action={formAction}>
        <ModalWindow.Body className="space-y-3">
          <Notifications formActionResponse={formActionResponse} />

          <StyledBox
            as="input"
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
            disabled={amount === 0}
            variant="button.primary"
          >
            Transfer
          </StyledText>
          <StyledText
            variant="button.secondary"
            onClick={onClose}
          >
            Cancel
          </StyledText>
        </ModalWindow.Buttons>
      </form>
    </ModalWindow>
  )
}
