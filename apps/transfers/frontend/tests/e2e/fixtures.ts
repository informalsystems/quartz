import {
  test as baseTest,
  chromium,
  expect,
  BrowserContext,
} from '@playwright/test'
import path from 'path'

// Tests fixtures
const test = baseTest.extend<{}, { _globalContext: BrowserContext }>({
  // Shared context for tests so Keplr initialization runs only once for all tests
  _globalContext: [
    async ({}, use) => {
      const mnemonicWords = process.env.TEST_KEPLR_MNEMONIC!.split(' ')
      const pathToExtension = path.join(
        __dirname,
        'extensions',
        `keplr-extension-manifest-v3-v${process.env.TEST_KEPLR_EXTENSION_VERSION}`,
      )
      // We launch browser with the extension
      const context = await chromium.launchPersistentContext('', {
        headless: false,
        args: [
          `--disable-extensions-except=${pathToExtension}`,
          `--load-extension=${pathToExtension}`,
        ],
      })
      const page = await context.waitForEvent('page')
      const extensionId = /\/\/(.*?)\//.exec(page.url())![1]

      // Keplr import wallet flow
      await page.waitForURL(new RegExp(`${extensionId}/register.html`))
      await expect(page.getByText('Import an existing wallet')).toBeVisible()
      await page
        .getByRole('button', { name: 'Import an existing wallet' })
        .click()
      await expect(
        page.getByText('Use recovery phrase or private key'),
      ).toBeVisible()
      await page
        .getByRole('button', { name: 'Use recovery phrase or private key' })
        .click()

      await page.getByText('24 Words').click()
      const seedInputs = await page.locator('input')
      for (let i = 0; i < mnemonicWords.length; i++) {
        await seedInputs.nth(i).fill(mnemonicWords[i])
      }
      await page.getByRole('button', { name: 'Import', exact: true }).click()

      await page
        .getByPlaceholder('e.g. Trading, NFT Vault, Investment')
        .fill('Playwright Wallet')
      const inputs = await page.getByPlaceholder(
        'At least 8 characters in length',
      )
      for (let i = 0; i < (await inputs.count()); i++) {
        await inputs.nth(i).fill(process.env.TEST_KEPLR_PASSWORD!)
      }
      await page.getByRole('button', { name: 'Next' }).click()

      await expect(page.getByText('Select Chains')).toBeVisible()
      await page.getByRole('button', { name: 'Save' }).click()

      // Accept app suggested testnet info
      await page.goto('/')
      const addChainPage = await context.waitForEvent('page')
      await addChainPage.getByRole('button', { name: 'Approve' }).click()

      // Wait for App to load
      await test.expect(page.getByText('Balance:')).toBeVisible()

      await use(context)
      await context.close()
    },
    { scope: 'worker' },
  ],
  context: async ({ _globalContext }, use) => {
    await use(_globalContext)
  },
  page: async ({ context }, use) => {
    const page = await context.newPage()

    await use(page)
    await page.close()
  },
})

export default test
