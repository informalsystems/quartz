import test from './fixtures'

test.beforeEach(async ({ page }) => {
  await page.goto('/')
})

test.describe('Transfers', () => {
  test('app should render correctly', async ({ page }) => {
    await test.expect(page.getByText('Balance:')).toBeVisible()
  })

  test('balance should be 0 at first', async ({ page }) => {
    await test.expect(page.getByText('$0')).toBeVisible()
  })
})
