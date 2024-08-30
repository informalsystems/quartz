'use client'

import { useActionState, useState } from 'react'
import { useCosmWasmSigningClient, useExecuteContract } from 'graz'

import { LoadingSpinner } from '@/components/LoadingSpinner'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { Notifications } from '@/components/Notifications'
import { StyledText } from '@/components/StyledText'
import { FormActionResponse } from '@/lib/types'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'

export function WithdrawModalWindow(props: ModalWindowProps) {
  const [loading, setLoading] = useState(false)
  const [formActionResponse, formAction] = useActionState(handleWithdraw, null)
  const { data: signingClient } = useCosmWasmSigningClient()
  const { executeContract } = useExecuteContract({
    contractAddress: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS!,
    onSuccess: (data) => {
      console.log(data)
      setLoading(false)
    },
    onError: () => setLoading(false),
  })

  // Withdraw all funds from the wallet balance calling the Transfer contract
  async function handleWithdraw(): Promise<FormActionResponse> {
    try {
      setLoading(true)
      executeContract({
        signingClient: signingClient!,
        msg: contractMessageBuilders.withdraw(),
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
      disableClosing={loading}
      {...props}
    >
      <LoadingSpinner isLoading={loading} />

      <ModalWindow.Title className="bg-amber-500">Withdraw</ModalWindow.Title>

      <form action={formAction}>
        <ModalWindow.Body className="space-y-3">
          <Notifications formActionResponse={formActionResponse} />
          <p>This will return the entire remaining balance to your wallet.</p>
        </ModalWindow.Body>
        <ModalWindow.Buttons>
          <StyledText
            as="button"
            className="bg-amber-500"
            variant="button.primary"
          >
            Withdraw
          </StyledText>
          <StyledText
            variant="button.secondary"
            onClick={props.onClose}
          >
            Cancel
          </StyledText>
        </ModalWindow.Buttons>
      </form>
    </ModalWindow>
  )
}
