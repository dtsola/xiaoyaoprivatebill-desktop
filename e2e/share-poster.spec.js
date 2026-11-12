import fs from 'node:fs'
import path from 'node:path'

import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

const read = (name) =>
  JSON.parse(fs.readFileSync(path.join('fixtures', 'expected', name), 'utf-8'))

const fixtures = {
  '/available_years': read('available_years.json'),
  '/available_dates': read('available_dates.json'),
  '/categories': read('categories.json'),
  '/category_available_dates': read('category_available_dates.json'),
  '/yearly_analysis': read('yearly_analysis_2024.json'),
  '/monthly_analysis': read('monthly_analysis_2024_12.json'),
  '/files': read('files_empty.json')
}

test('侧边栏入口：分享海报位于第二项并打开独立页面', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.addInitScript(installTauriMock, fixtures)
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.goto('/')
  await page.waitForTimeout(600)

  const navItems = page.locator('.nav-menu .nav-item')
  await expect(navItems.nth(1)).toContainText('分享海报')
  await expect(navItems.nth(1)).toHaveClass(/nav-item-share/)

  await navItems.nth(1).click()
  await expect(page).toHaveURL(/#\/share$/)
  await expect(page.locator('.page-title').first()).toHaveText('分享海报')
  await expect(page.locator('.poster-loading')).toHaveCount(0, { timeout: 10_000 })

  const canvasInfo = await page.evaluate(() => {
    const canvas = document.querySelector('.share-poster-content canvas')
    return { width: canvas?.width, height: canvas?.height }
  })
  expect(canvasInfo).toEqual({ width: 1080, height: 1440 })

  await page.screenshot({ path: 'artifacts/ui/share-poster/page.png', fullPage: true })

  await page.getByRole('button', { name: '保存图片' }).click()
  const saved = await page.evaluate(() => window.__TAURI_TEST_SAVED_POSTER)
  expect(saved?.pngBase64?.startsWith('iVBORw0KGgo')).toBe(true)

  // 保存 1080×1440 原图，便于人工检查海报细节
  fs.mkdirSync('artifacts/ui/share-poster', { recursive: true })
  fs.writeFileSync(
    'artifacts/ui/share-poster/poster.png',
    Buffer.from(saved.pngBase64, 'base64')
  )

  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})

test('分享海报：月度筛选、模板切换与单项金额开关', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.addInitScript(installTauriMock, fixtures)
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.goto('/#/share')
  await page.waitForTimeout(800)

  const annualImage = await page.evaluate(() =>
    document.querySelector('.share-poster-content canvas').toDataURL('image/png')
  )

  await expect(page.locator('.theme-card')).toHaveCount(3)
  await page.getByRole('tab', { name: '月度账单' }).click()
  await expect(page.getByLabel('选择月份')).toHaveValue('2024-12')
  await expect(page.locator('.poster-loading')).toHaveCount(0, { timeout: 10_000 })
  await page.waitForTimeout(300)

  // 切到月度后画布必须重绘为月度内容，而不是保留年度海报
  const monthlyImage = await page.evaluate(() =>
    document.querySelector('.share-poster-content canvas').toDataURL('image/png')
  )
  expect(monthlyImage).not.toBe(annualImage)

  await page.locator('.theme-card', { hasText: '墨夜' }).click()
  await expect(page.locator('.theme-card', { hasText: '墨夜' })).toHaveClass(/active/)

  // 月度默认隐藏收入与结余，点击开关可打开
  const incomeSwitch = page.locator('.switch-row', { hasText: '收入' })
  await expect(incomeSwitch).toHaveAttribute('aria-pressed', 'false')
  await incomeSwitch.click()
  await expect(incomeSwitch).toHaveAttribute('aria-pressed', 'true')

  await page.screenshot({ path: 'artifacts/ui/share-poster/page-monthly.png', fullPage: true })

  await page.getByRole('button', { name: '保存图片' }).click()
  const saved = await page.evaluate(() => window.__TAURI_TEST_SAVED_POSTER)
  expect(saved?.path).toMatch(/\.png$/)
  expect(saved?.pngBase64?.startsWith('iVBORw0KGgo')).toBe(true)
  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})

test('分享海报：当月数据未覆盖月末时显示截至日期', async ({ page }) => {
  const now = new Date()
  const year = now.getFullYear()
  const month = now.getMonth() + 1
  const monthKey = `${year}-${String(month).padStart(2, '0')}`
  const daysInMonth = new Date(year, month, 0).getDate()
  const dates = Array.from(
    { length: daysInMonth },
    (_, index) => `${monthKey}-${String(index + 1).padStart(2, '0')}`
  )

  const partialFixtures = {
    '/available_years': { success: true, years: [year] },
    '/available_dates': { success: true, years: [year], months: [monthKey] },
    '/files': read('files_empty.json'),
    '/yearly_analysis': read('yearly_analysis_2024.json'),
    '/monthly_analysis': {
      success: true,
      data: {
        stats: {
          total_expense: 1200,
          total_income: 0,
          balance: -1200,
          expense_count: 12,
          income_count: 0
        },
        daily_data: {
          expense: {
            dates,
            amounts: dates.map((_, index) => (index < 12 ? 100 : 0))
          },
          income: { dates, amounts: dates.map(() => 0) }
        },
        categories_source: {
          expense: [{ 交易分类: '餐饮美食', 来源: '微信', 金额: 1200 }]
        }
      }
    }
  }

  await page.addInitScript(installTauriMock, partialFixtures)
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.goto('/#/share')
  await page.waitForTimeout(600)
  await page.getByRole('tab', { name: '月度账单' }).click()
  await expect(page.locator('.poster-loading')).toHaveCount(0, { timeout: 10_000 })
  await page.waitForTimeout(300)

  await page.screenshot({
    path: 'artifacts/ui/share-poster/page-current-month.png',
    fullPage: true
  })
})
