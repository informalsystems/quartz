'use client'

import { ChangeEvent, useState } from 'react'
import { useCosmWasmSigningClient, useExecuteContract } from 'graz'
import { isEmpty } from 'lodash'

import { LoadingSpinner } from '@/components/LoadingSpinner'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { StyledBox } from '@/components/StyledBox'
import { StyledText } from '@/components/StyledText'
import { contractMessageBuilders } from '@/lib/contractMessageBuilders'
import { tw } from '@/lib/tw'
import chain from '@/config/chain'
import { showError, showSuccess } from '@/lib/notifications'
import { Icon } from './Icon'

export function DepositModalWindow(props: ModalWindowProps) {
  const [amount, setAmount] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const { data: signingClient } = useCosmWasmSigningClient()
  const { executeContract } = useExecuteContract({
    contractAddress: process.env.NEXT_PUBLIC_TRANSFERS_CONTRACT_ADDRESS!,
    onSuccess: (data) => {
      console.log(data)
      setLoading(false)
      showSuccess('Deposit transaction sent successfully')
      setTimeout(() => props.onClose(), 2000) // Close after 2 seconds
    },
    onError: (error: any) => {
      setLoading(false)
      showError(error.message)
    },
  })

  // Deposit the specified amount calling the Transfer contract
  function handleDeposit() {
    setError('')

    if (amount <= 0) {
      setError('Amount should be greater than zero.')

      return
    }

    setLoading(true)
    executeContract({
      signingClient: signingClient!,
      msg: contractMessageBuilders.deposit(),
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

      <ModalWindow.Title className="bg-emerald-500">
        <Icon name="piggy-bank" /> Deposit
      </ModalWindow.Title>

      <ModalWindow.Body className="space-y-3">
        {!isEmpty(error) && (
          <div className="font-bold text-red-500">{error}</div>
        )}

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
          onClick={handleDeposit}
        >
          Deposit
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
