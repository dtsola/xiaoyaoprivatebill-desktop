import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    // 模拟系统文件对话框返回的绝对路径
    window.__TAURI_TEST_DIALOG = ['C:/bills/alipay_2024.csv']
  })
  await page.addInitScript(installTauriMock)
})

test('点击上传区调用系统对话框并显示文件', async ({ page }) => {
  await page.goto('/#/settings')
  await expect(page.locator('.page-title')).toHaveText('文件管理')

  await page.locator('.file-upload').first().click()

  await expect(page.locator('.toast-success, .toast').first()).toContainText('文件上传成功')
  await expect(page.locator('.file-name', { hasText: 'alipay_2024.csv' })).toBeVisible()
})

test('上传后文件出现在支付宝列表', async ({ page }) => {
  await page.goto('/#/settings')
  await page.locator('.file-upload').first().click()

  const alipayList = page.locator('.file-list').first()
  await expect(alipayList.locator('.file-name')).toHaveText(['alipay_2024.csv'])
})

test('取消对话框不发起上传', async ({ page }) => {
  await page.addInitScript(() => {
    window.__TAURI_TEST_DIALOG = null
  })
  await page.goto('/#/settings')
  await page.locator('.file-upload').first().click()

  await expect(page.locator('.file-item')).toHaveCount(0)
})
