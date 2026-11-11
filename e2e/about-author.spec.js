import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('关于页外链交给系统浏览器打开', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/about-author')

  await page.locator('.contact-item', { hasText: 'www.dtsola.com' }).click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://www.dtsola.com')

  await page.locator('.project-link').click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://github.com/dtsola/xiaoyaoprivatebill')

  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})
