import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('演示模式可进入并退出', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })
  page.on('dialog', (dialog) => dialog.accept())

  await page.goto('/')
  await page.locator('.demo-button').click()

  await expect(page.locator('.toast-success, .toast').first()).toContainText('已进入演示模式')
  await expect(page.locator('.demo-banner')).toBeVisible()
  await expect(page).toHaveURL(/#\/yearly/)

  await page.locator('.exit-demo-btn').click()
  await expect(page.locator('.demo-banner')).toHaveCount(0)
  await expect(page).toHaveURL(/\/$/)

  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})
