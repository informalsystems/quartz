'use client'

import { useEffect, useState } from 'react'

import chain from '@/config/chain'
import { LoadingWrapper } from './LoadingWrapper'
import Middleware from './Middleware'
import GrazWrapper from './GrazWrapper'

// Method to skip first render because Graz initial wallet status is 'disconnected' and NOT 'reconnecting'
// With this, we let the middleware initialize with the 'reconnecting' status
const SkipFirstRender = ({ children }: { children: React.ReactNode }) => {
  const [isNotFirstRender, setIsNotFirstRender] = useState(false)

  useEffect(() => {
    setIsNotFirstRender(true)
  }, [])

  return isNotFirstRender && children
}

// Global App stuff definition
export default function App({ children }: { children: React.ReactNode }) {
  return (
    <GrazWrapper chains={[chain]}>
      <SkipFirstRender>
        <Middleware>{children}</Middleware>
      </SkipFirstRender>
      <LoadingWrapper />
    </GrazWrapper>
  )
}
