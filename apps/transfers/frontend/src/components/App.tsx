'use client'

import { ReactNode, useEffect, useState } from 'react'

import { wallet } from '@/lib/wallet'
import { cosm } from '@/lib/cosm'
import { LoadingSpinner } from './LoadingSpinner'

// Root component
export function App({ children }: { children: ReactNode }) {
  const [isInit, setIsInit] = useState(false)

  useEffect(() => {
    // Inititalize Keplr and CosmWasm wrappers
    wallet
      .init()
      .then(cosm.init)
      .then(() => setIsInit(true))
  }, [])

  return (
    <>
      <LoadingSpinner isLoading={!isInit} />
      {isInit && children}
    </>
  )
}
