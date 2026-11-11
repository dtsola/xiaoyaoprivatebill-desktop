import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('侧边栏贴窗口顶部，拖拽区只覆盖内容区并提供窗口按钮', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/')

  // 侧边栏从窗口最顶部开始（包含标题栏高度）
  const sidebar = await page.locator('.sidebar').boundingBox()
  expect(sidebar.y).toBe(0)
  expect(sidebar.height).toBe(840)

  // 拖拽区在侧边栏右侧，右侧让出 96px 给窗口按钮
  const strip = page.locator('.drag-strip')
  await expect(strip).toHaveAttribute('data-tauri-drag-region', '')
  const box = await strip.boundingBox()
  expect(box.height).toBe(36)
  expect(box.width).toBe(1280 - 240 - 96)
  expect(box.x).toBe(240)

  const btn = await page.locator('.window-btn').first().boundingBox()
  expect(btn.width).toBe(26)
  expect(btn.height).toBe(26)

  // 滚动只发生在内容区：窗口本身不滚动，滚动条不会顶到拖拽区
  const scroll = await page.evaluate(() => ({
    windowScroll: document.scrollingElement.scrollHeight - window.innerHeight,
    contentOverflow: getComputedStyle(document.querySelector('.content')).overflowY
  }))
  expect(scroll.windowScroll).toBeLessThanOrEqual(1)
  expect(scroll.contentOverflow).toBe('auto')

  // 内容区让出侧边栏宽度
  const contentMargin = await page
    .locator('.content')
    .evaluate((el) => getComputedStyle(el).marginLeft)
  expect(contentMargin).toBe('240px')

  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})

test('最小化、最大化/还原、关闭按钮调用窗口 API', async ({ page }) => {
  await page.goto('/')

  await page.locator('.window-btn[aria-label="最小化"]').click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_INVOKES))
    .toContain('plugin:window|minimize')

  const maximize = page.locator('.window-btn[aria-label="最大化"]')
  await maximize.click()
  await expect(page.locator('.window-btn[aria-label="还原"]')).toBeVisible()

  await page.locator('.window-btn[aria-label="还原"]').click()
  await expect(page.locator('.window-btn[aria-label="最大化"]')).toBeVisible()

  await page.locator('.window-btn[aria-label="关闭"]').click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_INVOKES))
    .toContain('plugin:window|close')
})

test('侧边栏 LOGO 已替换为新版本', async ({ page }) => {
  await page.goto('/')

  const logo = page.locator('.logo-icon')
  await expect(logo).toHaveAttribute('src', '/images/logo_128.png')
  await expect
    .poll(() => logo.evaluate((img) => (img.complete ? img.naturalWidth : 0)))
    .toBe(128)

  // 9 个页面的侧边栏统一头像尺寸：44×44，避免压过文字
  const box = await logo.boundingBox()
  expect(box.width).toBe(44)
  expect(box.height).toBe(44)
})

test('侧边栏为浅灰底色，选中项用白色胶囊区分', async ({ page }) => {
  await page.goto('/')

  const sidebarBg = await page
    .locator('.sidebar')
    .evaluate((el) => getComputedStyle(el).backgroundColor)
  expect(sidebarBg).toBe('rgb(242, 242, 244)')

  const activeBg = await page
    .locator('.nav-item.active')
    .first()
    .evaluate((el) => getComputedStyle(el).backgroundColor)
  expect(activeBg).toBe('rgb(255, 255, 255)')
})

test('滚动时页面头部紧贴标题栏，不出现间隙', async ({ page }) => {
  await page.goto('/#/settings')

  await page.locator('.content').evaluate((el) => {
    el.scrollTop = 400
  })
  await expect
    .poll(() =>
      page.evaluate(() => {
        const content = document.querySelector('.content').getBoundingClientRect()
        const header = document.querySelector('.page-header').getBoundingClientRect()
        return Math.round(header.top - content.top)
      })
    )
    .toBe(0)
})
