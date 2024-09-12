import { BrowserContext, Page } from '@playwright/test'

export const signTx = async ({
  context,
  page,
}: {
  context: BrowserContext
  page: Page
}) => {
  // Sign tx
  const signPage = await context.waitForEvent('page')

  await signPage.getByRole('button', { name: /approve/i }).click()
  await signPage.waitForEvent('close')
  await page.getByText(/successfully/i).waitFor({ state: 'visible' })
}
