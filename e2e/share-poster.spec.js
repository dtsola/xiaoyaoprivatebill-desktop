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

test('年度总览：生成、隐藏金额并保存分享海报', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.addInitScript(installTauriMock, fixtures)
  await page.setViewportSize({ width: 1280, height: 900 })
  await page.goto('/#/yearly')
  await page.waitForTimeout(1200)

  await page.getByRole('button', { name: '分享海报' }).click()
  const dialog = page.locator('.share-dialog')
  await expect(dialog).toBeVisible()
  await expect(dialog.getByText('分享海报')).toBeVisible()
  await expect(page.locator('.poster-loading')).toHaveCount(0, { timeout: 10_000 })

  const canvasInfo = await page.evaluate(() => {
    const canvas = document.querySelector('.share-dialog canvas')
    return {
      width: canvas?.width,
      height: canvas?.height,
      dataUrlLength: canvas?.toDataURL('image/png').length ?? 0
    }
  })
  expect(canvasInfo.width).toBe(1080)
  expect(canvasInfo.height).toBe(1440)
  expect(canvasInfo.dataUrlLength).toBeGreaterThan(10_000)

  await page.screenshot({ path: 'artifacts/ui/share-poster/preview.png' })

  await dialog.getByRole('button', { name: '保存图片' }).click()
  const saved = await page.evaluate(() => window.__TAURI_TEST_SAVED_POSTER)
  expect(saved?.path).toMatch(/\.png$/)
  expect(saved?.pngBase64?.startsWith('iVBORw0KGgo')).toBe(true)
  expect(saved?.pngBase64?.length).toBeGreaterThan(10_000)

  // 保存 1080×1440 原图，便于人工检查海报细节
  fs.mkdirSync('artifacts/ui/share-poster', { recursive: true })
  fs.writeFileSync(
    'artifacts/ui/share-poster/poster.png',
    Buffer.from(saved.pngBase64, 'base64')
  )

  await dialog.getByRole('button', { name: '隐藏金额' }).click()
  await expect(dialog.getByRole('button', { name: '显示金额' })).toBeVisible()
  await page.screenshot({ path: 'artifacts/ui/share-poster/preview-hidden.png' })

  await page.keyboard.press('Escape')
  await expect(dialog).toHaveCount(0)
  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})

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

  await expect(page.locator('.theme-card')).toHaveCount(3)
  await page.getByRole('tab', { name: '月度账单' }).click()
  await expect(page.getByLabel('选择月份')).toHaveValue('2024-12')
  await expect(page.locator('.poster-loading')).toHaveCount(0, { timeout: 10_000 })

  await page.locator('.theme-card', { hasText: '墨夜' }).click()
  await expect(page.locator('.theme-card', { hasText: '墨夜' })).toHaveClass(/active/)

  const incomeSwitch = page.locator('.switch-row', { hasText: '收入' })
  await incomeSwitch.click()
  await expect(incomeSwitch).toHaveAttribute('aria-pressed', 'false')

  await page.screenshot({ path: 'artifacts/ui/share-poster/page-monthly.png', fullPage: true })

  await page.getByRole('button', { name: '保存图片' }).click()
  const saved = await page.evaluate(() => window.__TAURI_TEST_SAVED_POSTER)
  expect(saved?.path).toMatch(/\.png$/)
  expect(saved?.pngBase64?.startsWith('iVBORw0KGgo')).toBe(true)
  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})
