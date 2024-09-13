'use client'

import { useRouter } from 'next/navigation'
import { useSuggestChainAndConnect } from 'graz'

import chain from '@/config/chain'
import { StyledText } from '@/components/StyledText'
import { useGlobalState } from '@/state/useGlobalState'

export default function Landing() {
  const router = useRouter()
  const { suggestAndConnect } = useSuggestChainAndConnect({
    onSuccess: () => router.replace('/set-seed'),
  })

  const connectWallet = () => {
    useGlobalState.getState().setLoading(true)
    suggestAndConnect({ chainInfo: chain })
  }

  return (
    <main className="flex min-h-screen flex-col items-center gap-4 p-24">
      <p>Connect your Keplr wallet to log in</p>
      <StyledText
        as="button"
        variant="button.primary"
        onClick={connectWallet}
      >
        Connect Keplr
      </StyledText>
    </main>
  )
}
