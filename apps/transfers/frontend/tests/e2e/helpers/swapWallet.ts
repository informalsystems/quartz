import { BrowserContext } from '@playwright/test'

export const swapWallet = async ({
  context,
  extensionUrl,
  name,
}: {
  context: BrowserContext
  extensionUrl: string
  name: string
}) => {
  const page = await context.newPage()

  await page.goto(`${extensionUrl}/popup.html`)
  await page.locator('div[cursor="pointer"] > svg').nth(1).click()
  await page.getByText(name).click()

  await page.close()
}
