import { test as baseTest, chromium } from '@playwright/test'
import path from 'path'

import { importWallet } from './helpers/importWallet'

let extensionUrl: string
const pathToExtension = path.join(
  __dirname,
  'extensions',
  `keplr-extension-manifest-v3-v${process.env.TEST_KEPLR_EXTENSION_VERSION}`,
)

// Tests fixtures
const test = baseTest.extend<{
  extensionUrl: string
}>({
  // Overwritten Playwright context to setup Keplr wallet before all tests
  context: async ({}, use) => {
    // Launch browser with Keplr installed
    const context = await chromium.launchPersistentContext('', {
      headless: false,
      args: [
        `--disable-extensions-except=${pathToExtension}`,
        `--load-extension=${pathToExtension}`,
      ],
    })

    const page = await context.waitForEvent('page')

    // Retrieve target URL to interact with Keplr extension
    const extensionId = /\/\/(.*?)\//.exec(page.url())![1]
    extensionUrl = `chrome-extension://${extensionId}`

    // Import a wallet to be used in tests
    await importWallet({
      extensionUrl,
      mnemonic: process.env.TEST_WALLET_MNEMONIC,
      name: 'main',
      page,
    })

    await use(context)
    await context.close()
  },
  extensionUrl: async ({}, use) => {
    await use(extensionUrl)
  },
})

export default test
