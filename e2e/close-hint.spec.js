import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('首次关闭到托盘时给出一次提示', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/')
  await page.evaluate(() => window.__TAURI_TEST_EMIT('desktop://close-to-tray-hint', null))

  await expect(page.locator('.toast-success, .toast').first()).toContainText('最小化到托盘')
  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})
