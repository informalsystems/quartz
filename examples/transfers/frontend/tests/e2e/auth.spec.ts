import { routes } from '@/config/routes'
import test from './fixtures'
import { connectWallet } from './helpers/connectWalet'
import { setSeedPhrase } from './helpers/setSeedPhrase'

const { dashboard, landing, seed } = routes

test.describe('Auth', () => {
  test('can go nowhere but landing page without a wallet', async ({ page }) => {
    await page.goto(seed)
    await page.goto(dashboard)
    await test
      .expect(page.getByRole('button', { name: /connect/i }))
      .toBeVisible()
  })

  test('can go nowhere but seed page without a seed phrase', async ({
    context,
    page,
  }) => {
    await connectWallet({ context, page })

    await page.goto(landing)
    await page.goto(dashboard)
    await test.expect(page.getByText(/recovery seed phrase/i)).toBeVisible()
  })

  test('cannot go to anon pages once fully logged in', async ({
    context,
    page,
  }) => {
    await connectWallet({ context, page })
    await setSeedPhrase({ page })

    await page.goto(landing)
    await page.goto(seed)
    await test.expect(page.getByText(/balance:/i)).toBeVisible()
  })
})
