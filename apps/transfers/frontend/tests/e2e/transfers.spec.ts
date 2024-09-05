import test from './fixtures'
import { importWallet } from './helpers/importWallet'
import { getBalance } from './helpers/getBalance'
import { swapWallet } from './helpers/swapWallet'
import { signTx } from './helpers/signTx'
import { connectWallet } from './helpers/connectWalet'
import { setSeedPhrase } from './helpers/setSeedPhrase'

test.describe.configure({ mode: 'serial' })
test.beforeEach(async ({ context, page }) => {
  await connectWallet({ context, page })
  await setSeedPhrase({ page, seedPhrase: process.env.TEST_WALLET_MNEMONIC! })
})

let mainBalance: number

test.describe('Transfers', () => {
  test('can deposit a sum successfully', async ({ context, page }) => {
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
    page,
  }) => {
    // Import a secondary wallet to transfer to
    await importWallet({
      extensionUrl,
      mnemonic: process.env.TEST_SECONDARY_WALLET_MNEMONIC!,
      page: await context.newPage(),
      name: 'secondary',
    })

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

  test('can withdraw deposited sum successfully', async ({ context, page }) => {
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
