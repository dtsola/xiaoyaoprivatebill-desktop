import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('首页与侧边栏可加载', async ({ page }) => {
  const pageErrors = []
  page.on('pageerror', (error) => pageErrors.push(error.message))

  await page.goto('/#/')
  await expect(page.locator('.sidebar')).toBeVisible()
  await expect(page.locator('.welcome-title')).toHaveText('小遥账单助手')
  expect(pageErrors).toEqual([])
})

test('hash 路由：点击侧边栏更新 URL', async ({ page }) => {
  await page.goto('/#/')
  await page.getByRole('link', { name: '年度总览' }).click()

  await expect(page).toHaveURL(/#\/yearly$/)
  await expect(page.locator('.page-title')).toHaveText('年度总览')
})

test('IPC：年度页显示可用年份 2024', async ({ page }) => {
  await page.goto('/#/yearly')
  await expect(page.locator('.control-display')).toHaveText('2024年')
})

test('图标本地化：Lucide SVG 正常渲染且不加载外部资源', async ({ page }) => {
  const externalRequests = []
  page.on('request', (request) => {
    const url = request.url()
    if (url.startsWith('http') && !url.startsWith('http://localhost:41420')) {
      externalRequests.push(url)
    }
  })

  await page.goto('/#/')
  await expect(page.locator('.sidebar')).toBeVisible()

  expect(externalRequests.filter((url) => url.includes('bootcdn'))).toEqual([])

  // 侧边栏图标已换成 Lucide SVG：渲染出可见尺寸
  const iconBox = await page.locator('.nav-item svg').first().boundingBox()
  expect(iconBox.width).toBeGreaterThan(10)
  expect(iconBox.height).toBeGreaterThan(10)
})

test('10 个页面可切换且无未捕获异常', async ({ page }) => {
  const pageErrors = []
  page.on('pageerror', (error) => pageErrors.push(error.message))

  const pages = [
    ['/#/', '.welcome-title'],
    ['/#/yearly', '.page-title'],
    ['/#/monthly', '.page-title'],
    ['/#/category', '.page-title'],
    ['/#/time', '.page-title'],
    ['/#/insights', '.page-title'],
    ['/#/transactions', '.page-title'],
    ['/#/agent', '.page-title'],
    ['/#/settings', '.page-title'],
    ['/#/about-author', '.author-title']
  ]

  for (const [route, selector] of pages) {
    await page.goto(route)
    await expect(page.locator('.sidebar')).toBeVisible()
    await expect(page.locator(selector).first()).toBeVisible()
  }

  expect(pageErrors).toEqual([])
})
