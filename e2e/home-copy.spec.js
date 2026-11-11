import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

// 桌面版隐私模型：数据落在本机数据目录，不随会话删除。
test('首页隐私文案符合桌面版模型', async ({ page }) => {
  await page.goto('/#/')

  const privacyCard = page.locator('.feature-card').filter({ hasText: '隐私保护' })
  await expect(privacyCard).toContainText('数据仅保存在本机')
  await expect(privacyCard).toContainText('不发起任何网络请求')

  const notice = page.locator('.privacy-notice')
  await expect(notice).toContainText('CSV/XLSX')
  await expect(notice).toContainText('可随时删除')
})

test('首页不再出现会话结束自动删除的旧文案', async ({ page }) => {
  await page.goto('/#/')
  await expect(page.locator('.sidebar')).toBeVisible()

  const body = await page.locator('body').innerText()
  expect(body).not.toContain('会话结束后自动删除')
  expect(body).not.toContain('数据仅在浏览器中分析')
})
