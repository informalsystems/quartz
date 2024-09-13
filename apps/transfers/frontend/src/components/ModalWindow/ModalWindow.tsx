'use client'

import { ComponentProps, useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { twMerge } from 'tailwind-merge'
import { classNames } from './classNames'

export interface ModalWindowProps extends ComponentProps<'div'> {
  disableClosing?: boolean
  isOpen: boolean
  onClose: () => void
}

export function ModalWindow({
  children,
  className,
  disableClosing,
  isOpen,
  onClose,
  ...otherProps
}: ModalWindowProps) {
  const [isClient, setIsClient] = useState(false)

  const [modalState, setModalState] = useState<
    'opening' | 'open' | 'closing' | 'closed'
  >('closed')

  const windowContentsContainerRef = useRef<HTMLDivElement>(null)

  function handleTransitionEnd() {
    if (modalState === 'closing') {
      setModalState('closed')
      onClose()
    }
    if (modalState === 'opening') {
      setModalState('open')
    }
  }

  function handleClose() {
    setModalState('closing')
  }

  function focusFirstElement() {
    const windowContentsContainer = windowContentsContainerRef.current

    if (!windowContentsContainer) {
      return
    }

    const firstFocusableElement = windowContentsContainer.querySelector(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
    ) as HTMLElement

    if (firstFocusableElement) {
      firstFocusableElement.focus()
    }
  }

  useEffect(() => {
    setIsClient(true)
  }, [])

  useEffect(() => {
    if (isOpen) {
      setModalState('opening')
      focusFirstElement()
    } else {
      setModalState('closing')
    }
  }, [isOpen])

  useEffect(() => {
    if (!disableClosing && isOpen) {
      const handleEscape = (event: KeyboardEvent) => {
        if (event.key === 'Escape') {
          handleClose()
        }
      }

      window.addEventListener('keydown', handleEscape)

      return () => window.removeEventListener('keydown', handleEscape)
    }
  }, [disableClosing, isOpen, onClose])

  if (!isClient) {
    return null
  }

  return createPortal(
    <>
      <div
        className={classNames.backdrop({ modalState })}
        {...(!disableClosing && { onClick: handleClose })}
      />
      <div
        className={twMerge(classNames.container({ modalState }), className)}
        ref={windowContentsContainerRef}
        onTransitionEnd={handleTransitionEnd}
        {...otherProps}
      >
        {isOpen && children}
      </div>
    </>,
    document.body,
  )
}

ModalWindow.Title = function ModalWindowTitle({
  children,
  className,
}: ComponentProps<'header'>) {
  return <div className={twMerge(classNames.header, className)}>{children}</div>
}

ModalWindow.Body = function ModalWindowBody({
  children,
  className,
}: ComponentProps<'main'>) {
  return <div className={twMerge(classNames.body, className)}>{children}</div>
}

ModalWindow.Buttons = function ModalWindowBody({
  children,
  className,
}: ComponentProps<'div'>) {
  return (
    <div className={twMerge(classNames.buttons, className)}>{children}</div>
  )
}
