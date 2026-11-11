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
  const storageCard = page.locator('.desktop-card').first()
  await expect(storageCard).toBeVisible()
  await expect(storageCard.locator('.mcp-endpoint')).toContainText('xiaoyaobill-desktop')
  await expect(storageCard).toContainText('2 个条目')
  await expect(storageCard).toContainText('1 个')
  await expect(storageCard).toContainText('2.0 KB')

  const toggle = page.locator('.desktop-card').last().locator('.mcp-btn')
  await expect(toggle).toHaveText('关闭窗口时最小化到托盘')
  await toggle.click()
  await expect(toggle).toHaveText('关闭窗口时直接退出')
  await expect(page.locator('.toast-success, .toast').first()).toContainText('直接退出')

  const openButtons = storageCard.locator('.mcp-btn', { hasText: '打开' })
  for (let index = 0; index < 3; index += 1) {
    await openButtons.nth(index).click()
  }
  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})

test('设置页标题与分组职责一致', async ({ page }) => {
  await page.goto('/#/settings')

  await expect(page.locator('.page-title')).toHaveText('设置')
  const sections = await page.locator('.section-title').allTextContents()
  expect(sections).toEqual(['账单文件', '数据管理', '存储与审计', '桌面'])
})
