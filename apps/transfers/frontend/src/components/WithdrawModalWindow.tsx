'use client'
import { useActionState } from 'react'

import { LoadingSpinner } from '@/components/LoadingSpinner'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { Notifications } from '@/components/Notifications'
import { StyledText } from '@/components/StyledText'
import { cosm } from '@/lib/cosm'
import { FormActionResponse } from '@/lib/types'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'

// Withdraw all funds from the wallet balance calling the Transfer contract
async function handleWithdraw(): Promise<FormActionResponse> {
  try {
    const result = await cosm.executeTransferContract({
      messageBuilder: contractMessageBuilders.withdraw,
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

export function WithdrawModalWindow({
  isOpen,
  onClose,
  ...otherProps
}: ModalWindowProps) {
  const [formActionResponse, formAction, isLoading] = useActionState(
    handleWithdraw,
    null,
  )

  return (
    <ModalWindow
      isOpen={isOpen}
      onClose={onClose}
      {...otherProps}
    >
      <LoadingSpinner isLoading={isLoading} />

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
            onClick={onClose}
          >
            Cancel
          </StyledText>
        </ModalWindow.Buttons>
      </form>
    </ModalWindow>
  )
}
