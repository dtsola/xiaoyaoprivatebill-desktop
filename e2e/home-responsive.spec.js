import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

async function columns(page) {
  return page
    .locator('.features-grid')
    .evaluate((el) => getComputedStyle(el).gridTemplateColumns.split(' ').length)
}

test('首页随窗口宽度自适应：默认两列、宽屏四列', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))

  await page.setViewportSize({ width: 1280, height: 840 })
  await page.goto('/')
  await expect(page.locator('.welcome-title')).toBeVisible()
  expect(await columns(page)).toBe(2)

  // 内容区铺满可用宽度，且不超过 1200px 的设计上限
  const width = await page.locator('.welcome-section').evaluate((el) => el.clientWidth)
  expect(width).toBeGreaterThan(900)
  expect(width).toBeLessThanOrEqual(1200)

  await page.setViewportSize({ width: 1600, height: 900 })
  await expect.poll(() => columns(page)).toBe(4)

  await page.setViewportSize({ width: 1100, height: 840 })
  await expect.poll(() => columns(page)).toBe(2)

  // 任何宽度都不出现横向滚动条
  const overflow = await page.evaluate(
    () => document.scrollingElement.scrollWidth - window.innerWidth
  )
  expect(overflow).toBeLessThanOrEqual(0)

  expect(errors).toEqual([])
})
