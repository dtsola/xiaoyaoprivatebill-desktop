import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('设置页可开启、轮换 Token、关闭 MCP 服务', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/settings')
  await expect(page.locator('.mcp-status')).toHaveText('已停止')

  await page.locator('.mcp-btn.primary').click()
  await expect(page.locator('.mcp-status')).toHaveText('运行中')
  await expect(page.locator('.mcp-token')).toHaveText('a'.repeat(64))
  await expect(page.locator('.toast-success, .toast').first()).toContainText('MCP 服务已开启')

  await page.getByRole('button', { name: '重新生成 Token' }).click()
  await expect(page.locator('.mcp-token')).toHaveText('b'.repeat(64))
  await expect(page.locator('.toast-success, .toast').first()).toContainText('Token 已重新生成')

  await page.locator('.mcp-btn.primary').click()
  await expect(page.locator('.mcp-status')).toHaveText('已停止')
  await expect(page.locator('.mcp-token')).toHaveCount(0)

  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})
