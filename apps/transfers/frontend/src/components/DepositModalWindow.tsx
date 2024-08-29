'use client'

import { ChangeEvent, useActionState, useState } from 'react'
import { useCosmWasmSigningClient, useExecuteContract } from 'graz'

import { LoadingSpinner } from '@/components/LoadingSpinner'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { Notifications } from '@/components/Notifications'
import { StyledBox } from '@/components/StyledBox'
import { StyledText } from '@/components/StyledText'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'
import { tw } from '@/lib/tw'
import { FormActionResponse } from '@/lib/types'
import chain from '@/config/chain'

export function DepositModalWindow({
  isOpen,
  onClose,
  ...otherProps
}: ModalWindowProps) {
  const [amount, setAmount] = useState(0)
  const [formActionResponse, formAction, isLoading] = useActionState(
    handleDeposit,
    null,
  )
  const { data: signingClient } = useCosmWasmSigningClient()
  const { executeContract } = useExecuteContract({
    contractAddress: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS!,
    onSuccess: (data) => {
      console.log(data)
    },
  })

  // Deposit the specified amount calling the Transfer contract
  async function handleDeposit(
    _: FormActionResponse,
    formData: FormData,
  ): Promise<FormActionResponse> {
    try {
      executeContract({
        signingClient: signingClient!,
        msg: contractMessageBuilders.deposit(),
        funds: [
          {
            denom: chain.currencies[0].coinMinimalDenom,
            amount: String(formData.get('amount')),
          },
        ],
      })

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

  return (
    <ModalWindow
      isOpen={isOpen}
      onClose={onClose}
      {...otherProps}
    >
      <LoadingSpinner isLoading={isLoading} />

      <ModalWindow.Title className="bg-emerald-500">Deposit</ModalWindow.Title>

      <form action={formAction}>
        <ModalWindow.Body className="space-y-3">
          <Notifications formActionResponse={formActionResponse} />

          <StyledBox
            as="input"
            className={tw`
              focus:!border-emerald-500
              focus:!outline-emerald-500
              focus:!ring-emerald-500
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
            className="bg-emerald-500"
            disabled={amount === 0}
            variant="button.primary"
          >
            Deposit
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
