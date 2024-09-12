import Link from 'next/link'

import { StyledText } from '@/components/StyledText'

export default function NotFound() {
  return (
    <main className="flex min-h-screen flex-col items-center gap-4 p-24">
      <p>Page not found ❌</p>
      <StyledText
        as={Link}
        href="/"
        variant="button.secondary"
      >
        Go Home
      </StyledText>
    </main>
  )
}
