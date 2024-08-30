'use client'

import { useState } from 'react'
import { useCosmWasmSigningClient, useExecuteContract } from 'graz'

import { LoadingSpinner } from '@/components/LoadingSpinner'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { StyledText } from '@/components/StyledText'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'
import { showError, showSuccess } from '@/lib/notifications'

export function WithdrawModalWindow(props: ModalWindowProps) {
  const [loading, setLoading] = useState(false)
  const { data: signingClient } = useCosmWasmSigningClient()
  const { executeContract } = useExecuteContract({
    contractAddress: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS!,
    onSuccess: (data) => {
      console.log(data)
      setLoading(false)
      showSuccess('Withdraw transaction sent successfully')
    },
    onError: (error: any) => {
      setLoading(false)
      showError(error.message)
    },
  })

  // Withdraw all funds from the wallet balance calling the Transfer contract
  function handleWithdraw() {
    setLoading(true)
    executeContract({
      signingClient: signingClient!,
      msg: contractMessageBuilders.withdraw(),
    })
  }

  return (
    <ModalWindow
      disableClosing={loading}
      {...props}
    >
      <LoadingSpinner isLoading={loading} />

      <ModalWindow.Title className="bg-amber-500">Withdraw</ModalWindow.Title>

      <ModalWindow.Body className="space-y-3">
        <p>This will return the entire remaining balance to your wallet.</p>
      </ModalWindow.Body>
      <ModalWindow.Buttons>
        <StyledText
          as="button"
          className="bg-amber-500"
          variant="button.primary"
          onClick={handleWithdraw}
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
    </ModalWindow>
  )
}
