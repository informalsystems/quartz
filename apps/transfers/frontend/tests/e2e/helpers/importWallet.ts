import { Page } from '@playwright/test'

export const importWallet = async ({
  extensionUrl,
  mnemonic,
  name,
  page,
}: {
  extensionUrl: string
  mnemonic: string
  name: string
  page: Page
}) => {
  await page.goto(`${extensionUrl}/register.html`)

  const mnemonicWords = mnemonic.split(' ')

  await page.getByRole('button', { name: /import/i }).click()
  await page.getByRole('button', { name: /use/i }).click()
  await page.getByRole('button', { name: /24/ }).click()

  const seedInputs = await page.locator('input')

  for (let i = 0; i < mnemonicWords.length; i++) {
    await seedInputs.nth(i).fill(mnemonicWords[i])
  }

  await page.getByRole('button', { name: 'Import', exact: true }).click()
  await page.getByPlaceholder('e.g. Trading, NFT Vault,').fill(name)

  const inputs = await page.getByPlaceholder('At least 8 characters in length')

  for (let i = 0; i < (await inputs.count()); i++) {
    await inputs.nth(i).fill(process.env.TEST_WALLET_PASSWORD)
  }

  await page.getByRole('button', { name: /next/i }).click()
  await page.getByRole('button', { name: /save/i }).click()

  await page.close()
}
