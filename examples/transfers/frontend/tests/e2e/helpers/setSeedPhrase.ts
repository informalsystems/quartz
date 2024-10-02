import { Page } from '@playwright/test'

export const setSeedPhrase = async ({
  page,
  seedPhrase,
}: {
  page: Page
  seedPhrase?: string
}) => {
  if (!seedPhrase) {
    await page.getByRole('button', { name: /continue with/i }).click()
  } else {
    await page.getByRole('button', { name: /enter my own/i }).click()
    await page.locator('input').fill(seedPhrase)
    await page.getByRole('button', { name: 'Continue', exact: true }).click()
  }
}
