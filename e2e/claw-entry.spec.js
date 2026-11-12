import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('侧边栏「小遥Claw」入口直接打开官网且不切换路由', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/yearly')

  const entry = page.locator('.nav-item-external', { hasText: '小遥Claw' })
  await expect(entry).toBeVisible()
  await expect(page).toHaveURL(/#\/yearly$/)

  await entry.click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://www.xiaoyaosai.com/xiaoyaoclaw/')

  // 外链入口不改变当前页面
  await expect(page).toHaveURL(/#\/yearly$/)

  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})
