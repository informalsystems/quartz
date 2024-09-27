'use client'

import { ChangeEvent, useState } from 'react'
import { useRouter } from 'next/navigation'
import { EnglishMnemonic } from '@cosmjs/crypto'
import { isEmpty } from 'lodash'

import { saveMnemonic } from '@/lib/ephemeralKeypair'
import { ModalWindow, ModalWindowProps } from '@/components/ModalWindow'
import { StyledBox } from './StyledBox'
import { StyledText } from './StyledText'
import { useGlobalState } from '@/state/useGlobalState'

export function EnterSeedModal({
  isOpen,
  onClose,
  ...otherProps
}: ModalWindowProps) {
  const router = useRouter()
  const [mnemonic, setMnemonic] = useState('')
  const [error, setError] = useState('')

  const submitSeed = () => {
    try {
      useGlobalState.getState().setLoading(true)
      setError('')
      const englishMnemonic = new EnglishMnemonic(mnemonic)
      saveMnemonic(englishMnemonic.toString())
      router.replace('/dashboard')
    } catch (err: any) {
      useGlobalState.getState().setLoading(false)
      setError(err.message)
      if (err.message !== 'Invalid mnemonic format') {
        throw err
      }
    }
  }

  return (
    <ModalWindow
      isOpen={isOpen}
      onClose={onClose}
      {...otherProps}
    >
      <ModalWindow.Title>Enter recovery phrase</ModalWindow.Title>
      <ModalWindow.Body className="space-y-3">
        {!isEmpty(error) && (
          <div className="font-bold text-red-500">{error}</div>
        )}
        <StyledBox
          as="input"
          min={0}
          value={mnemonic}
          variant="input"
          onChange={(event: ChangeEvent<HTMLInputElement>) =>
            setMnemonic(event.target.value)
          }
        />
      </ModalWindow.Body>
      <ModalWindow.Buttons>
        <StyledText
          as="button"
          variant="button.primary"
          onClick={submitSeed}
        >
          Continue
        </StyledText>
        <StyledText
          as="button"
          variant="button.secondary"
          onClick={onClose}
        >
          Cancel
        </StyledText>
      </ModalWindow.Buttons>
    </ModalWindow>
  )
}
