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

  // 小遥Claw 标准 MCP 服务格式：http + url + Bearer Header
  const config = page.locator('.mcp-config-code')
  await expect(config).toBeVisible()
  await expect(config).toContainText('"type": "http"')
  await expect(config).toContainText('"url": "http://127.0.0.1:17650/mcp"')
  await expect(config).toContainText(`"Authorization": "Bearer ${'a'.repeat(64)}"`)

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

test('AI Agent 接入页展示小遥Claw 接入步骤与官网入口', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/agent')

  const guide = page.locator('.claw-guide')
  await expect(guide).toBeVisible()
  await expect(guide.locator('h3')).toHaveText('小遥Claw 接入方式')
  await expect(guide.locator('.claw-step')).toHaveCount(4)
  await expect(guide).toContainText('技能与 MCP')
  await expect(guide).toContainText('xiaoyaobill-mcp')
  await expect(guide).toContainText('连接正常 · 14 个工具')

  await guide.getByRole('button', { name: '打开小遥Claw 官网' }).click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_OPENED))
    .toBe('https://www.xiaoyaosai.com/xiaoyaoclaw/')

  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})

test('AI Agent 接入页展示小遥Claw 实操截图并支持放大', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/agent')

  const shots = page.locator('.claw-shot')
  await expect(shots).toHaveCount(4)
  await expect(page.locator('.claw-guide')).toContainText('添加 MCP 服务')
  await expect(page.locator('.claw-guide')).toContainText('连接正常 · 14 个工具')
  await expect(page.locator('.claw-guide')).toContainText('用自然语言发起分析')
  await expect(page.locator('.claw-guide')).toContainText('生成结构化账单分析')

  // 点击缩略图打开大图预览
  await shots.first().click()
  await expect(page.locator('.claw-preview')).toBeVisible()
  await expect(page.locator('.claw-preview img')).toHaveAttribute(
    'src',
    '/images/claw/01-add-mcp.png'
  )

  await page.getByRole('button', { name: '关闭预览' }).click()
  await expect(page.locator('.claw-preview')).toHaveCount(0)

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
