'use client'

import { Icon } from '@/components/Icon'
import { StyledText } from '@/components/StyledText'
import { FormActionResponse } from '@/lib/types'
import { uniqueId } from 'lodash'
import { ComponentProps, useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import { twMerge } from 'tailwind-merge'
import { classNames } from './classNames'

interface NotificationsProps extends Omit<ComponentProps<'ul'>, 'children'> {
  formActionResponse: FormActionResponse | undefined
}

type NotificationState = 'entering' | 'entered' | 'exiting' | 'exited'

export function Notifications({
  className,
  formActionResponse,
  ...otherProps
}: NotificationsProps) {
  const [storedMessages, setStoredMessages] = useState<
    { id: number; message: string }[]
  >([])

  const [isClient, setIsClient] = useState(false)

  const { success, messages } = formActionResponse || {
    success: false,
    messages: [],
  }

  useEffect(() => {
    if (messages) {
      setStoredMessages((currentStoredMessages) => [
        ...currentStoredMessages,
        ...messages.map((message) => ({
          id: Number(uniqueId()),
          message,
        })),
      ])
    }
  }, [messages])

  useEffect(() => {
    setIsClient(true)
  }, [])

  function handleDismissMessage(id: number) {
    setStoredMessages((currentStoredMessages) =>
      currentStoredMessages.filter((message) => message.id !== id),
    )
  }

  if (!isClient) {
    return null
  }

  return createPortal(
    <>
      <div
        className={classNames.backdrop({
          hasMessages: storedMessages.length > 0,
          success,
        })}
      />
      {messages && (
        <ul
          className={twMerge(classNames.container, className)}
          {...otherProps}
        >
          {storedMessages.map(({ id, message }, index) => (
            <Notifications.Notification
              index={index}
              key={id}
              message={message}
              success={success}
              onDismiss={handleDismissMessage.bind(null, id)}
            />
          ))}
        </ul>
      )}
    </>,
    document.body,
  )
}

Notifications.Notification = function Notification({
  index,
  message,
  success,
  onDismiss,
  ...otherProps
}: ComponentProps<'li'> & {
  index: number
  message: string
  success: boolean
  onDismiss: () => void
}) {
  const [state, setState] = useState<NotificationState>('entering')

  function handleAnimationEnd() {
    if (state === 'entering') {
      setState('entered')
    }

    if (state === 'exiting') {
      setState('exited')
      onDismiss()
    }
  }

  return (
    state !== 'exited' && (
      <li
        className={classNames.notificationContainer({ state })}
        style={{
          animationDelay: `${index * 0.1}s`,
        }}
        onAnimationEnd={handleAnimationEnd}
        {...otherProps}
      >
        <div className={classNames.notificationSurface({ success })}>
          <Icon name={success ? 'circle-check' : 'circle-exclamation'} />
          <div>{message}</div>
          <div>
            <StyledText
              as="button"
              variant="button.tool"
              onClick={() => setState('exiting')}
            >
              <Icon name="xmark" />
            </StyledText>
          </div>
        </div>
      </li>
    )
  )
}
