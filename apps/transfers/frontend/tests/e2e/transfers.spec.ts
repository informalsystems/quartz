import { Page } from '@playwright/test'

import { routes } from '@/config/routes'
import test from './fixtures'
import { importWallet } from './helpers/importWallet'
import { getBalance } from './helpers/getBalance'
import { swapWallet } from './helpers/swapWallet'
import { signTx } from './helpers/signTx'

let page: Page

test.describe.configure({ mode: 'serial' })
test.beforeAll(async ({ context }) => {
  page = await context.newPage()
  // Connect to Keplr wallet
  await page.goto(routes.landing)
  await page.getByRole('button', { name: /connect/i }).click()

  // Accept app suggested testnet info
  const addChainPage = await context.waitForEvent('page')

  await addChainPage.getByRole('button', { name: /approve/i }).click()
  await addChainPage.waitForEvent('close')
  await page.getByRole('button', { name: /continue with/i }).click()
})
test.afterAll(async () => {
  await page.close()
})

let mainBalance: number

test.describe('Transfers', () => {
  test('can deposit a sum successfully', async ({ context }) => {
    // Initialize the balance
    mainBalance = Number(
      (await getBalance({ context, page }))!.replace('$', ''),
    )

    await page.getByRole('button', { name: /deposit/i }).click()
    await page.keyboard.type('20')
    await page
      .getByRole('button', { name: /deposit/i })
      .nth(1)
      .click()

    await signTx({ context, page })

    await page
      .getByRole('button', { name: /cancel/i, includeHidden: false })
      .click()

    // Check new balance
    await page.waitForTimeout(4000)

    mainBalance += 20

    await test
      .expect(await getBalance({ context, page }))
      .toEqual(`$${mainBalance}`)
  })

  test('can transfer to another wallet successfully', async ({
    context,
    extensionUrl,
  }) => {
    const popupPage = await context.newPage()

    // Import a secondary wallet to transfer to
    await importWallet({
      extensionUrl,
      mnemonic: process.env.TEST_SECONDARY_WALLET_MNEMONIC!,
      page: popupPage,
      name: 'secondary',
    })

    await popupPage.close()

    // Initialize the secondary account balance after importing
    const secondaryBalance = Number(
      (await getBalance({ context, page }))!.replace('$', ''),
    )

    // Swap back to main wallet
    await swapWallet({ context, extensionUrl, name: 'main' })

    // Transfer to the secondary wallet
    await page.getByRole('button', { name: /transfer/i }).click()
    await page.keyboard.type(process.env.TEST_SECONDARY_WALLET_ADDRESS!)
    await page.getByPlaceholder('0.00').fill('10')
    await page
      .getByRole('button', { name: /transfer/i })
      .nth(1)
      .click()

    await signTx({ context, page })

    await page
      .getByRole('button', { name: /cancel/i, includeHidden: false })
      .click()

    // Check new balance
    await page.waitForTimeout(4000)
    mainBalance -= 10
    await test
      .expect(await getBalance({ context, page }))
      .toEqual(`$${mainBalance}`)

    // Swap to secondary to check if the transfer was received
    await swapWallet({ context, extensionUrl, name: 'secondary' })

    await test
      .expect(await getBalance({ context, page }))
      .toEqual(`$${secondaryBalance + 10}`)

    // Set balance to 0 again for cleaning purposes
    await page.getByRole('button', { name: /withdraw/i }).click()
    await page
      .getByRole('button', { name: /withdraw/i })
      .nth(1)
      .click()

    await signTx({ context, page })

    await page
      .getByRole('button', { name: /cancel/i, includeHidden: false })
      .click()

    // Back to main wallet
    await swapWallet({ context, extensionUrl, name: 'main' })
  })

  test('can withdraw deposited sum successfully', async ({ context }) => {
    await page.getByRole('button', { name: /withdraw/i }).click()
    await page
      .getByRole('button', { name: /withdraw/i })
      .nth(1)
      .click()

    await signTx({ context, page })

    await page
      .getByRole('button', { name: /cancel/i, includeHidden: false })
      .click()

    // Check new balance
    await page.waitForTimeout(4000)
    await test.expect(await getBalance({ context, page })).toEqual('$0')
  })
})
