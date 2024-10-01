'use client'

import dynamic from 'next/dynamic'

// Dynamic import for app so no Graz code prerenders in server
// This avoids errors on first render
const App = dynamic(() => import('./App'), { ssr: false })

export default function Root({ children }: React.PropsWithChildren) {
  return <App>{children}</App>
}
