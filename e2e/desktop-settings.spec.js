import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('设置页展示数据目录并可切换关闭行为', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/settings')
  await expect(page.locator('.desktop-card')).toBeVisible()
  await expect(page.locator('.desktop-card .mcp-endpoint')).toContainText('xiaoyaobill-desktop')
  await expect(page.locator('.desktop-card')).toContainText('2 个条目')
  await expect(page.locator('.desktop-card')).toContainText('1 个')
  await expect(page.locator('.desktop-card')).toContainText('2.0 KB')

  const toggle = page.locator('.desktop-row .mcp-btn').last()
  await expect(toggle).toHaveText('关闭窗口时最小化到托盘')
  await toggle.click()
  await expect(toggle).toHaveText('关闭窗口时直接退出')
  await expect(page.locator('.toast-success, .toast').first()).toContainText('直接退出')

  const openButtons = page.locator('.desktop-row .mcp-btn', { hasText: '打开' })
  for (let index = 0; index < 3; index += 1) {
    await openButtons.nth(index).click()
  }
  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})
