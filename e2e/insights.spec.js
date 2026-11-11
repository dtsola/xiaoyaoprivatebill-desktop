import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('洞察页加载 /analysis 全量数据并渲染', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/#/insights')
  await expect(page.locator('.page-title')).toHaveText('消费洞察')

  // 常客商家按金额降序渲染
  await expect(page.locator('.merchant-name').first()).toHaveText('家电店')
  await expect(page.locator('.merchant-name')).toHaveCount(4)

  // 智能标签
  await expect(page.locator('.tag')).toHaveCount(4)
  await expect(page.locator('.tag').first()).toHaveText('娱乐控')

  // 拿铁因子与隐形订阅
  await expect(page.locator('.latte-card')).toContainText('咖啡店')
  await expect(page.locator('.sub-card')).toContainText('视频会员')

  // 图表容器已完成 ECharts 初始化（canvas 存在）
  await expect(page.locator('.sankey-card canvas').first()).toBeVisible()

  expect(errors, `控制台错误:\n${errors.join('\n')}`).toEqual([])
})
