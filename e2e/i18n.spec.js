import fs from 'node:fs'
import path from 'node:path'

import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

const read = (name) =>
  JSON.parse(fs.readFileSync(path.join('fixtures', 'expected', name), 'utf-8'))

const fixtures = {
  '/available_years': read('available_years.json'),
  '/available_dates': read('available_dates.json'),
  '/yearly_analysis': read('yearly_analysis_2024.json'),
  '/files': read('files_empty.json')
}

async function switchToEnglish(page) {
  await page.goto('/#/settings')
  await page.waitForTimeout(800)
  await page.locator('.language-select').selectOption('en-US')
  await expect(page.locator('.nav-menu .nav-item').first()).toContainText('Home')
}

test('切换英文后导航与首页文案生效', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.addInitScript(installTauriMock, fixtures)
  await page.setViewportSize({ width: 1280, height: 900 })
  await switchToEnglish(page)

  const invokes = await page.evaluate(() => window.__TAURI_TEST_INVOKES)
  expect(invokes).toContain('set_language')

  await page.locator('.nav-menu .nav-item').first().click()
  await expect(page.locator('.welcome-title')).toHaveText('Xiaoyao Bill')
  await expect(page.locator('.feature-title').first()).toHaveText(
    'Multi-dimensional analysis'
  )
  await expect(page.locator('.upload-title')).toHaveText('Start analyzing your bills')

  fs.mkdirSync('artifacts/ui/i18n', { recursive: true })
  await page.screenshot({ path: 'artifacts/ui/i18n/home-en.png', fullPage: true })
  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})

test('英文界面下分享海报文案与文件名', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.addInitScript(installTauriMock, fixtures)
  await page.setViewportSize({ width: 1280, height: 900 })
  await switchToEnglish(page)

  await page.goto('/#/share')
  await expect(page.locator('.page-title').first()).toHaveText('Share Poster')
  await expect(page.getByRole('tab', { name: 'Monthly' })).toBeVisible()
  await expect(page.getByRole('tab', { name: 'Yearly' })).toBeVisible()
  await expect(page.locator('.theme-card', { hasText: 'Sky' })).toBeVisible()
  await expect(page.locator('.switch-row', { hasText: 'Total spending' })).toBeVisible()
  await expect(page.locator('.poster-loading')).toHaveCount(0, { timeout: 10_000 })

  await page.getByRole('button', { name: 'Save image' }).click()
  await expect
    .poll(() =>
      page.evaluate(
        () => window.__TAURI_TEST_SAVE_ARGS?.options?.defaultPath ?? null
      )
    )
    .toBe('XiaoyaoBill-2024-yearly.png')

  fs.mkdirSync('artifacts/ui/i18n', { recursive: true })
  await page.screenshot({ path: 'artifacts/ui/i18n/poster-en.png', fullPage: true })
  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})
