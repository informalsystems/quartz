'use client'

import { useEffect, useState } from 'react'
import { SnackbarProvider } from 'notistack'

import chain from '@/config/chain'
import Middleware from './Middleware'
import GrazWrapper from './GrazWrapper'
import { LoadingWrapper } from './LoadingWrapper'

// Method to skip first render because Graz initial wallet status is 'disconnected' and NOT 'reconnecting'
// With this, we let the middleware initialize with the 'reconnecting' status
const SkipFirstRender = ({ children }: React.PropsWithChildren) => {
  const [isNotFirstRender, setIsNotFirstRender] = useState(false)

  useEffect(() => {
    setIsNotFirstRender(true)
  }, [])

  return isNotFirstRender && children
}

// Global App stuff definition
export default function App({ children }: React.PropsWithChildren) {
  return (
    <GrazWrapper chains={[chain]}>
      <SkipFirstRender>
        <Middleware>{children}</Middleware>
      </SkipFirstRender>
      <LoadingWrapper />
      <SnackbarProvider
        anchorOrigin={{ horizontal: 'right', vertical: 'bottom' }}
        preventDuplicate
      />
    </GrazWrapper>
  )
}
