import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(installTauriMock)
})

async function widthOf(page, selector) {
  return page.locator(selector).evaluate((el) => Math.round(el.getBoundingClientRect().width))
}

test('侧边栏可收起为图标栏，并记住状态', async ({ page }) => {
  await page.goto('/')

  await expect(page.locator('.sidebar')).toHaveCSS('width', '240px')
  await expect(page.locator('.nav-item', { hasText: '年度总览' })).toBeVisible()

  await page.getByRole('button', { name: '收起侧边栏' }).click()

  await expect(page.locator('.sidebar')).toHaveCSS('width', '64px')
  expect(await widthOf(page, '.content')).toBeGreaterThan(1000)
  // 文字标签隐藏，只留图标
  await expect(page.locator('.nav-item span').first()).toBeHidden()
  await expect(page.locator('.logo span')).toBeHidden()
  // 底部筛选菜单同步收成图标按钮
  expect(await widthOf(page, '.floating-menu')).toBe(64)
  await expect(page.locator('.floating-menu .filter-btn span').first()).toBeHidden()
  expect(await page.evaluate(() => localStorage.getItem('xiaoyaobill.sidebar'))).toBe(
    'collapsed'
  )

  // 刷新后保持收起
  await page.reload()
  await expect(page.locator('.sidebar')).toHaveCSS('width', '64px')

  // 展开后恢复并写回偏好
  await page.getByRole('button', { name: '展开侧边栏' }).click()
  await expect(page.locator('.sidebar')).toHaveCSS('width', '240px')
  expect(await page.evaluate(() => localStorage.getItem('xiaoyaobill.sidebar'))).toBe(
    'expanded'
  )
})

test('窄窗口自动收起，本次会话展开不写入偏好', async ({ page }) => {
  await page.setViewportSize({ width: 1050, height: 800 })
  await page.goto('/')

  await expect(page.locator('.sidebar')).toHaveCSS('width', '64px')

  await page.getByRole('button', { name: '展开侧边栏' }).click()
  await expect(page.locator('.sidebar')).toHaveCSS('width', '240px')
  expect(await page.evaluate(() => localStorage.getItem('xiaoyaobill.sidebar'))).toBeNull()
})
