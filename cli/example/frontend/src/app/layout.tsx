import { App } from '@/components'
import type { Metadata } from 'next'
import { Raleway } from 'next/font/google'
import Script from 'next/script'
import { twMerge } from 'tailwind-merge'
import './globals.css'

const bodyFont = Raleway({ subsets: ['latin'], variable: '--font-raleway' })

export const metadata: Metadata = {
  title: 'Cycles: Respect the Graph',
  description: 'The Open Clearing Protocol.',
  icons: [
    {
      rel: 'icon',
      type: 'image/png',
      sizes: '32x32',
      url: '/favicon.png',
    },
  ],
  openGraph: {
    type: 'website',
    url: 'https://example.com',
    title: 'Cycles: Respect the Graph',
    description: 'The Open Clearing Protocol.',
    siteName: 'Cycles',
    images: [
      {
        url: 'http://cycles.money/share-sheet-image.jpg',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    site: '@site',
    creator: '@creator',
    images: 'http://cycles.money/share-sheet-image.jpg',
  },
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <head>
        <Script src="https://kit.fontawesome.com/ddaf2d7713.js" />
      </head>
      <body
        className={twMerge(
          `
            overflow-x-hidden
            bg-appBgColor
            text-textColor
          `,
          bodyFont.className,
          bodyFont.variable,
        )}
      >
        <App>{children}</App>
      </body>
    </html>
  )
}
