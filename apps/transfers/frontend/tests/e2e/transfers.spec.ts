import test from './fixtures'

test.beforeEach(async ({ page }) => {
  await page.goto('/')
})

test.describe('Transfers', () => {
  test('app should render correctly', async ({ page }) => {
    await test
      .expect(page.getByRole('button', { name: /connect/i }))
      .toBeVisible()
  })
})
