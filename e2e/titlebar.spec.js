import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

test('自定义标题栏隐藏原生 title，并提供拖拽区与窗口按钮', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.goto('/')

  const titlebar = page.locator('.titlebar')
  await expect(titlebar).toBeVisible()
  // 整条标题栏是拖拽区，原生标题栏已被去掉
  await expect(titlebar).toHaveAttribute('data-tauri-drag-region', '')

  const box = await titlebar.boundingBox()
  expect(box.height).toBe(34)
  expect(box.width).toBe(1280)

  // 主内容与侧边栏都让位到标题栏下方
  const sidebar = await page.locator('.sidebar').boundingBox()
  expect(sidebar.y).toBeGreaterThanOrEqual(34)
  // 主内容用 54px 顶部内边距让位（标题栏 34px + 原有 20px）
  const contentPaddingTop = await page
    .locator('.content')
    .evaluate((el) => getComputedStyle(el).paddingTop)
  expect(contentPaddingTop).toBe('54px')

  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})

test('最小化、最大化/还原、关闭按钮调用窗口 API', async ({ page }) => {
  await page.goto('/')

  await page.locator('.titlebar-btn[aria-label="最小化"]').click()
  await expect
    .poll(() => page.evaluate(() => window.__TAURI_TEST_INVOKES))
    .toContain('plugin:window|minimize')

  const maximize = page.locator('.titlebar-btn[aria-label="最大化"]')
  await maximize.click()
  await expect(page.locator('.titlebar-btn[aria-label="还原"]')).toBeVisible()

  await page.locator('.titlebar-btn[aria-label="还原"]').click()
  await expect(page.locator('.titlebar-btn[aria-label="最大化"]')).toBeVisible()

  await page.locator('.titlebar-btn[aria-label="关闭"]').click()
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
})
