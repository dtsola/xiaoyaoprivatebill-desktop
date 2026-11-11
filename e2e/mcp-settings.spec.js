import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('AI Agent 接入页可开启、轮换 Token、关闭 MCP 服务', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/agent')
  await expect(page.locator('.page-title')).toHaveText('AI Agent 接入')
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

test('托盘切换后 AI Agent 接入页同步显示运行状态', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))

  await page.goto('/#/agent')
  await page.evaluate(() =>
    window.__TAURI_TEST_EMIT('desktop://mcp-status', {
      enabled: true,
      running: true,
      port: 17650,
      endpoint: 'http://127.0.0.1:17650/mcp',
      last_error: null
    })
  )

  await expect(page.locator('.mcp-status')).toHaveText('运行中')
  await expect(page.locator('.mcp-token')).toHaveText('a'.repeat(64))
  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})

test('侧边栏有独立菜单，设置页不再包含 MCP 区块', async ({ page }) => {
  await page.goto('/#/settings')

  const menu = page.locator('.nav-item', { hasText: 'AI Agent 接入' })
  await expect(menu).toBeVisible()
  await expect(page.locator('.mcp-section')).toHaveCount(0)

  await menu.click()
  await expect(page).toHaveURL(/#\/agent$/)
  await expect(page.locator('.page-title')).toHaveText('AI Agent 接入')
  await expect(page.locator('.mcp-section')).toBeVisible()
  // 独立页面只保留 MCP 区块，导入与桌面分组不再渲染
  await expect(page.locator('.split-layout')).toHaveCount(0)
  await expect(page.locator('.desktop-section')).toHaveCount(0)
})
