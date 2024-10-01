import { BrowserContext, Page } from '@playwright/test'

import { routes } from '@/config/routes'

export const connectWallet = async ({
  context,
  page,
}: {
  context: BrowserContext
  page: Page
}) => {
  // Connect to Keplr wallet
  await page.goto(routes.landing)
  await page.getByRole('button', { name: /connect/i }).click()

  // Accept app suggested testnet info
  const addChainPage = await context.waitForEvent('page')

  await addChainPage.getByRole('button', { name: /approve/i }).click()
  await addChainPage.waitForEvent('close')
}
