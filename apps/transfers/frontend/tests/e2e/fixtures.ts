import { test as baseTest, chromium, BrowserContext } from '@playwright/test'
import path from 'path'

import { importWallet } from './helpers/importWallet'

let extensionUrl: string

// Tests fixtures
const test = baseTest.extend<
  { extensionUrl: string },
  { _globalContext: BrowserContext }
>({
  // Shared context for tests so Keplr initialization runs only once for all tests
  _globalContext: [
    async ({}, use) => {
      const mnemonicWords = process.env.TEST_WALLET_MNEMONIC!.split(' ')
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
      extensionUrl = `chrome-extension://${extensionId}`

      await importWallet({
        extensionUrl,
        mnemonic: process.env.TEST_WALLET_MNEMONIC!,
        page,
        name: 'main',
      })

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
  extensionUrl: async ({}, use) => {
    await use(extensionUrl)
  },
})

export default test
