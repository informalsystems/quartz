import { BrowserContext, Page } from '@playwright/test'

export const getBalance = async ({
  context,
  page,
}: {
  context: BrowserContext
  page: Page
}) => {
  // Check new balance
  await page.getByRole('button', { name: /get/i }).click()

  // Sign tx
  const signGetTxPage = await context.waitForEvent('page')

  await signGetTxPage.getByRole('button', { name: /approve/i }).click()
  await signGetTxPage.waitForEvent('close')

  // Wait for the success alert to appear so we know balance updated
  await page.getByText(/\$/i).waitFor({ state: 'visible' })

  return page.getByText(/\$/i).textContent()
}
