'use client'

import { useEffect } from 'react'
import { usePathname, useRouter } from 'next/navigation'
import { useAccount } from 'graz'

import { getMnemonic } from '@/lib/ephemeralKeypair'
import { useGlobalState } from '@/state/useGlobalState'

enum SetupPhases {
  // User is not connected to Keplr Wallet
  WALLET_CONNECTION,
  // No mnemonic in local storage
  MNEMONIC_CREATION,
  // User is connected and has mnemonic
  FINISHED_SETUP,
}

const loginRoutes = ['/', '/set-seed']
const setupRoutesMapping: Record<SetupPhases, string> = {
  [SetupPhases.WALLET_CONNECTION]: '/',
  [SetupPhases.MNEMONIC_CREATION]: '/set-seed',
  [SetupPhases.FINISHED_SETUP]: '',
}

// App routing middleware
// NOTE: Cannot use Nextjs middleware file because browser info is required
export default function Middleware({
  children,
}: {
  children: React.ReactNode
}) {
  const router = useRouter()
  const pathname = usePathname()
  const { status, isDisconnected, isReconnecting } = useAccount()
  const mnemonic = getMnemonic()
  const isAnonPage = loginRoutes.includes(pathname)
  const targetRoute =
    setupRoutesMapping[
      ((): SetupPhases => {
        if (isDisconnected) {
          return SetupPhases.WALLET_CONNECTION
        } else if (!mnemonic) {
          return SetupPhases.MNEMONIC_CREATION
        } else {
          return SetupPhases.FINISHED_SETUP
        }
      })()
    ]

  useEffect(() => {
    if (isReconnecting) {
      return
    }

    let redirectTo = targetRoute

    if (!redirectTo && isAnonPage) {
      redirectTo = '/dashboard'
    }

    // Redirect if the path is not the correct one
    if (redirectTo && redirectTo !== pathname) {
      useGlobalState.getState().setLoading(true)
      router.replace(redirectTo)
    } else {
      useGlobalState.getState().setLoading(false)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status, pathname])

  return (
    !isReconnecting &&
    (targetRoute === pathname || (!isAnonPage && !targetRoute)) &&
    children
  )
}
